//! Thread-safe coordinator for the local inference session.
//!
//! Runtime work is serialized independently from status reads. A long startup
//! or generation therefore cannot prevent the UI from reading status or
//! signalling cancellation. This module never downloads assets or uses a
//! network endpoint other than the validated loopback runtime.

use std::{path::Path, sync::{Arc, Mutex}, time::Duration};

use serde::Serialize;

use crate::llama_client::GenerationCancellation;
use crate::llama_runtime::{LlamaRuntimeSession, RecoveryPolicy, RuntimeSessionError};
use crate::runtime::{GenerationRequest, RuntimeConfig, RuntimeStatus};

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStatusSnapshot {
    pub status: String,
    pub detail: Option<String>,
}

struct RuntimeManagerInner {
    operation: Mutex<()>,
    session: Mutex<Option<LlamaRuntimeSession>>,
    status: Mutex<RuntimeStatusSnapshot>,
    active_cancellation: Mutex<Option<GenerationCancellation>>,
}

#[derive(Clone)]
pub struct RuntimeManager {
    inner: Arc<RuntimeManagerInner>,
}

impl Default for RuntimeManager {
    fn default() -> Self {
        Self {
            inner: Arc::new(RuntimeManagerInner {
                operation: Mutex::new(()),
                session: Mutex::new(None),
                status: Mutex::new(RuntimeStatusSnapshot {
                    status: "notInstalled".to_owned(),
                    detail: None,
                }),
                active_cancellation: Mutex::new(None),
            }),
        }
    }
}

impl RuntimeManager {
    pub fn snapshot(&self) -> Result<RuntimeStatusSnapshot, String> {
        self.inner.status.lock()
            .map(|status| status.clone())
            .map_err(|_| "Runtime status is unavailable.".to_owned())
    }

    fn set_status(&self, status: RuntimeStatus, detail: Option<String>) -> Result<(), String> {
        let mut snapshot = self.inner.status.lock()
            .map_err(|_| "Runtime status is unavailable.".to_owned())?;
        snapshot.status = status_name(status).to_owned();
        snapshot.detail = detail;
        Ok(())
    }

    pub fn start(
        &self,
        config: RuntimeConfig,
        executable: &str,
        model: &str,
        startup_timeout: Duration,
        poll_interval: Duration,
        request_timeout: Duration,
    ) -> Result<(), String> {
        let _operation = self.inner.operation.lock()
            .map_err(|_| "Runtime operation lock is unavailable.".to_owned())?;
        let mut slot = self.inner.session.lock()
            .map_err(|_| "Runtime session is unavailable.".to_owned())?;
        if slot.is_some() {
            return Err("A runtime session already exists; stop it or use recovery first.".to_owned());
        }
        drop(slot);

        self.set_status(RuntimeStatus::Starting, None)?;
        match LlamaRuntimeSession::start(
            &config, Path::new(executable), Path::new(model),
            startup_timeout, poll_interval, request_timeout,
        ) {
            Ok(session) => {
                *self.inner.session.lock()
                    .map_err(|_| "Runtime session is unavailable.".to_owned())? = Some(session);
                self.set_status(RuntimeStatus::Ready, None)
            }
            Err(error) => {
                let detail = format!("{error:?}");
                self.set_status(RuntimeStatus::Failed, Some(detail.clone()))?;
                Err(detail)
            }
        }
    }

    /// Generate a complete response while forwarding each received text delta.
    pub fn generate(&self, request: GenerationRequest) -> Result<String, String> {
        let mut response = String::new();
        self.generate_with_callback(request, |delta| response.push_str(delta))?;
        Ok(response)
    }

    /// Stream text deltas to the caller while preserving the same lifecycle,
    /// serialization, and cancellation guarantees as complete-response calls.
    pub fn generate_with_callback<F>(
        &self,
        request: GenerationRequest,
        mut on_delta: F,
    ) -> Result<(), String>
    where
        F: FnMut(&str),
    {
        let _operation = self.inner.operation.lock()
            .map_err(|_| "Runtime operation lock is unavailable.".to_owned())?;
        let mut session = self.inner.session.lock()
            .map_err(|_| "Runtime session is unavailable.".to_owned())?;
        let session = session.as_mut()
            .ok_or_else(|| "No local runtime session is active.".to_owned())?;

        let cancellation = GenerationCancellation::default();
        *self.inner.active_cancellation.lock()
            .map_err(|_| "Generation cancellation is unavailable.".to_owned())? =
            Some(cancellation.clone());
        self.set_status(RuntimeStatus::Generating, None)?;

        let result = session.generate_streaming(&request, &cancellation, |delta| {
            on_delta(delta);
        });

        if let Ok(mut active) = self.inner.active_cancellation.lock() {
            *active = None;
        }

        match result {
            Ok(()) => {
                self.set_status(session.status(), None)
            }
            Err(RuntimeSessionError::Cancelled) => {
                self.set_status(session.status(), Some("Generation cancelled.".to_owned()))?;
                Err("Generation cancelled.".to_owned())
            }
            Err(error) => {
                let detail = format!("{error:?}");
                self.set_status(session.status(), Some(detail.clone()))?;
                Err(detail)
            }
        }
    }

    /// Signal an in-flight generation without waiting for its operation lock.
    pub fn cancel_generation(&self) -> Result<bool, String> {
        let active = self.inner.active_cancellation.lock()
            .map_err(|_| "Generation cancellation is unavailable.".to_owned())?;
        if let Some(cancellation) = active.as_ref() {
            cancellation.cancel();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn stop(&self) -> Result<(), String> {
        // Cancellation deliberately happens before waiting for the serialized
        // operation lock, allowing an active streaming request to unwind.
        let _ = self.cancel_generation()?;
        let _operation = self.inner.operation.lock()
            .map_err(|_| "Runtime operation lock is unavailable.".to_owned())?;
        let mut slot = self.inner.session.lock()
            .map_err(|_| "Runtime session is unavailable.".to_owned())?;

        let Some(session) = slot.as_mut() else {
            let current = self.snapshot()?;
            if current.status != "notInstalled" {
                self.set_status(RuntimeStatus::Stopped, None)?;
            }
            return Ok(());
        };

        match session.stop() {
            Ok(()) => {
                *slot = None;
                self.set_status(RuntimeStatus::Stopped, None)
            }
            Err(error) => {
                let detail = format!("{error:?}");
                self.set_status(session.status(), Some(detail.clone()))?;
                Err(detail)
            }
        }
    }

    pub fn recover(
        &self,
        executable: &str,
        model: &str,
        startup_timeout: Duration,
        poll_interval: Duration,
    ) -> Result<(), String> {
        let _operation = self.inner.operation.lock()
            .map_err(|_| "Runtime operation lock is unavailable.".to_owned())?;
        let mut slot = self.inner.session.lock()
            .map_err(|_| "Runtime session is unavailable.".to_owned())?;
        let session = slot.as_mut()
            .ok_or_else(|| "There is no existing session to recover.".to_owned())?;

        self.set_status(RuntimeStatus::Starting, None)?;
        match session.recover(
            Path::new(executable), Path::new(model), startup_timeout, poll_interval,
            RecoveryPolicy::default(),
        ) {
            Ok(()) => self.set_status(session.status(), None),
            Err(error) => {
                let detail = format!("{error:?}");
                self.set_status(session.status(), Some(detail.clone()))?;
                Err(detail)
            }
        }
    }
}

fn status_name(status: RuntimeStatus) -> &'static str {
    match status {
        RuntimeStatus::NotInstalled => "notInstalled",
        RuntimeStatus::Stopped => "stopped",
        RuntimeStatus::Starting => "starting",
        RuntimeStatus::Ready => "ready",
        RuntimeStatus::Generating => "generating",
        RuntimeStatus::Failed => "failed",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::thread;

    #[test]
    fn new_manager_reports_not_installed() {
        let snapshot = RuntimeManager::default().snapshot().unwrap();
        assert_eq!(snapshot.status, "notInstalled");
        assert_eq!(snapshot.detail, None);
    }

    #[test]
    fn generation_without_a_session_is_rejected() {
        let manager = RuntimeManager::default();
        let request = GenerationRequest {
            history: Vec::new(),
            prompt: "Hello".to_owned(), max_tokens: 32, temperature: 0.7, top_p: 0.9,
        };
        assert!(manager.generate(request).unwrap_err().contains("No local runtime"));
    }

    #[test]
    fn invalid_launch_configuration_is_reported_as_failed() {
        let manager = RuntimeManager::default();
        let config = RuntimeConfig {
            host: "0.0.0.0".to_owned(),
            ..RuntimeConfig::default()
        };
        assert!(manager.start(
            config, "missing-server", "missing-model",
            Duration::from_secs(1), Duration::from_millis(1), Duration::from_secs(1),
        ).is_err());
        assert_eq!(manager.snapshot().unwrap().status, "failed");
        assert!(manager.snapshot().unwrap().detail.is_some());
    }

    #[test]
    fn status_remains_readable_while_an_operation_is_busy() {
        let manager = RuntimeManager::default();
        let worker_manager = manager.clone();
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();

        let worker = thread::spawn(move || {
            let _operation = worker_manager.inner.operation.lock().unwrap();
            worker_manager.set_status(RuntimeStatus::Generating, None).unwrap();
            entered_tx.send(()).unwrap();
            release_rx.recv().unwrap();
        });

        entered_rx.recv().unwrap();
        assert_eq!(manager.snapshot().unwrap().status, "generating");
        release_tx.send(()).unwrap();
        worker.join().unwrap();
    }

    #[test]
    fn cancellation_signal_is_non_blocking_and_reports_if_generation_exists() {
        let manager = RuntimeManager::default();
        assert_eq!(manager.cancel_generation(), Ok(false));

        let token = GenerationCancellation::default();
        *manager.inner.active_cancellation.lock().unwrap() = Some(token.clone());
        assert_eq!(manager.cancel_generation(), Ok(true));
        assert!(token.is_cancelled());
    }
}
