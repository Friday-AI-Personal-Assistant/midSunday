//! Managed local inference session combining process, readiness, and generation.
use std::path::Path;
use std::time::Duration;

use crate::llama_client::{wait_for_ready, GenerationCancellation, LlamaClientError, LlamaHttpClient, ReadinessError};
use crate::llama_process::{LlamaServerProcess, ProcessStartError};
use crate::runtime::{GenerationRequest, RuntimeLifecycle, RuntimeStatus};

#[derive(Debug)]
pub enum RuntimeSessionError {
    InvalidTiming,
    ProcessStart(ProcessStartError),
    Client(LlamaClientError),
    Readiness(ReadinessError),
    Generation(LlamaClientError),
    Cancelled,
    ProcessStop(String),
    InvalidState,
}

/// Owns the child process and its loopback client as one lifecycle-managed unit.
///
/// The session is synchronous and bounded by the supplied HTTP/startup timeouts.
/// Dropping it also drops the process supervisor, which performs best-effort cleanup.
pub struct LlamaRuntimeSession {
    process: LlamaServerProcess,
    client: LlamaHttpClient,
    lifecycle: RuntimeLifecycle,
}

impl LlamaRuntimeSession {
    pub fn start(
        config: &crate::runtime::RuntimeConfig,
        executable: &Path,
        model: &Path,
        startup_timeout: Duration,
        poll_interval: Duration,
        request_timeout: Duration,
    ) -> Result<Self, RuntimeSessionError> {
        if startup_timeout.is_zero() || poll_interval.is_zero() || request_timeout.is_zero() {
            return Err(RuntimeSessionError::InvalidTiming);
        }

        let mut process = LlamaServerProcess::start(config, executable, model)
            .map_err(RuntimeSessionError::ProcessStart)?;
        let client = match LlamaHttpClient::new(config, request_timeout) {
            Ok(client) => client,
            Err(error) => {
                let _ = process.stop();
                return Err(RuntimeSessionError::Client(error));
            }
        };

        let mut lifecycle = RuntimeLifecycle::default();
        lifecycle.transition(RuntimeStatus::Stopped)
            .map_err(|_| RuntimeSessionError::InvalidState)?;
        lifecycle.transition(RuntimeStatus::Starting)
            .map_err(|_| RuntimeSessionError::InvalidState)?;

        if let Err(error) = wait_for_ready(&client, startup_timeout, poll_interval, || {
            process.is_running()
        }) {
            let _ = process.stop();
            let _ = lifecycle.transition(RuntimeStatus::Failed);
            return Err(RuntimeSessionError::Readiness(error));
        }

        lifecycle.transition(RuntimeStatus::Ready)
            .map_err(|_| RuntimeSessionError::InvalidState)?;

        Ok(Self { process, client, lifecycle })
    }

    pub fn status(&self) -> RuntimeStatus {
        self.lifecycle.status()
    }

    /// Generate one bounded, non-streaming response from the local server.
    pub fn generate(&mut self, request: &GenerationRequest) -> Result<String, RuntimeSessionError> {
        request.validate().map_err(|_| {
            RuntimeSessionError::Generation(LlamaClientError::InvalidGenerationRequest)
        })?;
        self.lifecycle.transition(RuntimeStatus::Generating)
            .map_err(|_| RuntimeSessionError::InvalidState)?;

        match self.client.generate(request) {
            Ok(response) => {
                self.lifecycle.transition(RuntimeStatus::Ready)
                    .map_err(|_| RuntimeSessionError::InvalidState)?;
                Ok(response)
            }
            Err(error) => {
                let _ = self.lifecycle.transition(RuntimeStatus::Failed);
                Err(RuntimeSessionError::Generation(error))
            }
        }
    }

    /// Generate streaming text deltas. The cancellation handle may be cloned
    /// before this call and triggered from another thread while generation runs.
    pub fn generate_streaming<F>(
        &mut self,
        request: &GenerationRequest,
        cancellation: &GenerationCancellation,
        on_delta: F,
    ) -> Result<(), RuntimeSessionError>
    where
        F: FnMut(&str),
    {
        request.validate().map_err(|_| {
            RuntimeSessionError::Generation(LlamaClientError::InvalidGenerationRequest)
        })?;
        self.lifecycle.transition(RuntimeStatus::Generating)
            .map_err(|_| RuntimeSessionError::InvalidState)?;

        match self.client.generate_streaming(request, cancellation, on_delta) {
            Ok(()) => {
                self.lifecycle.transition(RuntimeStatus::Ready)
                    .map_err(|_| RuntimeSessionError::InvalidState)?;
                Ok(())
            }
            Err(LlamaClientError::Cancelled) => {
                self.lifecycle.transition(RuntimeStatus::Ready)
                    .map_err(|_| RuntimeSessionError::InvalidState)?;
                Err(RuntimeSessionError::Cancelled)
            }
            Err(error) => {
                let _ = self.lifecycle.transition(RuntimeStatus::Failed);
                Err(RuntimeSessionError::Generation(error))
            }
        }
    }

    pub fn stop(&mut self) -> Result<(), RuntimeSessionError> {
        if self.lifecycle.status() == RuntimeStatus::Stopped {
            return Ok(());
        }

        if let Err(error) = self.process.stop() {
            let _ = self.lifecycle.transition(RuntimeStatus::Failed);
            return Err(RuntimeSessionError::ProcessStop(error.to_string()));
        }

        self.lifecycle.transition(RuntimeStatus::Stopped)
            .map_err(|_| RuntimeSessionError::InvalidState)
    }
}

