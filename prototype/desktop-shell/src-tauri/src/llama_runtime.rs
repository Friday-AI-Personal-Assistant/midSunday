//! Managed local inference session combining process, readiness, and generation.
use std::path::Path;
use std::time::Duration;

use crate::llama_client::{wait_for_ready, GenerationCancellation, LlamaClientError, LlamaHttpClient, ReadinessError};
use crate::llama_process::{LlamaServerProcess, ProcessStartError};
use crate::runtime::{GenerationRequest, RuntimeLifecycle, RuntimeStatus};


/// Bounded policy for an explicit runtime recovery attempt.
/// The maximum is deliberately small to prevent restart loops.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecoveryPolicy {
    pub max_attempts: u8,
    pub retry_delay: Duration,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecoveryPolicyError {
    InvalidAttemptCount,
}

impl Default for RecoveryPolicy {
    fn default() -> Self {
        Self { max_attempts: 2, retry_delay: Duration::from_secs(1) }
    }
}

impl RecoveryPolicy {
    pub fn validate(&self) -> Result<(), RecoveryPolicyError> {
        if !(1..=3).contains(&self.max_attempts) {
            return Err(RecoveryPolicyError::InvalidAttemptCount);
        }
        Ok(())
    }
}

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
    RecoveryPolicy(RecoveryPolicyError),
    RecoveryExhausted { attempts: u8, last_error: String },
}

/// Owns the child process and its loopback client as one lifecycle-managed unit.
///
/// The session is synchronous and bounded by the supplied HTTP/startup timeouts.
/// Dropping it also drops the process supervisor, which performs best-effort cleanup.
pub struct LlamaRuntimeSession {
    process: LlamaServerProcess,
    client: LlamaHttpClient,
    lifecycle: RuntimeLifecycle,
    config: crate::runtime::RuntimeConfig,
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

        Ok(Self { process, client, lifecycle, config: config.clone() })
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


    /// Explicitly recover a failed/stopped session with a bounded number of
    /// fresh process launches. This method never runs an unbounded restart loop.
    pub fn recover(
        &mut self,
        executable: &Path,
        model: &Path,
        startup_timeout: Duration,
        poll_interval: Duration,
        policy: RecoveryPolicy,
    ) -> Result<(), RuntimeSessionError> {
        policy.validate().map_err(RuntimeSessionError::RecoveryPolicy)?;
        if startup_timeout.is_zero() || poll_interval.is_zero() {
            return Err(RuntimeSessionError::InvalidTiming);
        }
        match self.lifecycle.status() {
            RuntimeStatus::Failed | RuntimeStatus::Stopped => {}
            _ => return Err(RuntimeSessionError::InvalidState),
        }

        self.process.stop().map_err(|error| {
            let _ = self.lifecycle.transition(RuntimeStatus::Failed);
            RuntimeSessionError::ProcessStop(error.to_string())
        })?;
        if self.lifecycle.status() == RuntimeStatus::Failed {
            self.lifecycle.transition(RuntimeStatus::Stopped)
                .map_err(|_| RuntimeSessionError::InvalidState)?;
        }

        let mut last_error = String::from("recovery did not start");
        for attempt in 1..=policy.max_attempts {
            self.lifecycle.transition(RuntimeStatus::Starting)
                .map_err(|_| RuntimeSessionError::InvalidState)?;

            let mut process = match LlamaServerProcess::start(&self.config, executable, model) {
                Ok(process) => process,
                Err(error) => {
                    last_error = format!("process start failed: {error:?}");
                    self.lifecycle.transition(RuntimeStatus::Failed)
                        .map_err(|_| RuntimeSessionError::InvalidState)?;
                    if attempt < policy.max_attempts {
                        self.lifecycle.transition(RuntimeStatus::Stopped)
                            .map_err(|_| RuntimeSessionError::InvalidState)?;
                        if !policy.retry_delay.is_zero() {
                            std::thread::sleep(policy.retry_delay);
                        }
                    }
                    continue;
                }
            };

            match wait_for_ready(&self.client, startup_timeout, poll_interval, || {
                process.is_running()
            }) {
                Ok(()) => {
                    self.process = process;
                    self.lifecycle.transition(RuntimeStatus::Ready)
                        .map_err(|_| RuntimeSessionError::InvalidState)?;
                    return Ok(());
                }
                Err(error) => {
                    last_error = format!("server did not become ready: {error:?}");
                    let _ = process.stop();
                    self.lifecycle.transition(RuntimeStatus::Failed)
                        .map_err(|_| RuntimeSessionError::InvalidState)?;
                    if attempt < policy.max_attempts {
                        self.lifecycle.transition(RuntimeStatus::Stopped)
                            .map_err(|_| RuntimeSessionError::InvalidState)?;
                        if !policy.retry_delay.is_zero() {
                            std::thread::sleep(policy.retry_delay);
                        }
                    }
                }
            }
        }

        Err(RuntimeSessionError::RecoveryExhausted {
            attempts: policy.max_attempts,
            last_error,
        })
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


#[cfg(test)]
mod recovery_policy_tests {
    use super::*;

    #[test]
    fn default_recovery_policy_is_bounded_and_valid() {
        let policy = RecoveryPolicy::default();
        assert_eq!(policy.max_attempts, 2);
        assert_eq!(policy.validate(), Ok(()));
    }

    #[test]
    fn recovery_policy_rejects_zero_attempts() {
        let policy = RecoveryPolicy { max_attempts: 0, retry_delay: Duration::ZERO };
        assert_eq!(policy.validate(), Err(RecoveryPolicyError::InvalidAttemptCount));
    }

    #[test]
    fn recovery_policy_rejects_more_than_three_attempts() {
        let policy = RecoveryPolicy { max_attempts: 4, retry_delay: Duration::ZERO };
        assert_eq!(policy.validate(), Err(RecoveryPolicyError::InvalidAttemptCount));
    }
}
