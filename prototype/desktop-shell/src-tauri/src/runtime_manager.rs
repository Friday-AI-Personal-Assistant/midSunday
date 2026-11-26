//! Thread-safe coordinator for the local inference session.
//!
//! Runtime work is serialized independently from status reads. A long startup
//! or generation therefore cannot prevent the UI from reading status or
//! signalling cancellation. This module never downloads assets or uses a
//! network endpoint other than the validated loopback runtime.

use std::{path::Path, sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex}, thread::{self, JoinHandle}, time::{Duration, SystemTime, UNIX_EPOCH}};

use serde::Serialize;

use crate::llama_client::GenerationCancellation;
use crate::llama_runtime::{LlamaRuntimeSession, RecoveryPolicy, RuntimeSessionError, RuntimeStartupMemoryReport};
use crate::runtime::{GenerationRequest, RuntimeConfig, RuntimeStatus};
use crate::system_info::{collect_known_process_memory, ProcessMemorySnapshot};

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStatusSnapshot {
    pub status: String,
    pub detail: Option<String>,
    pub startup_memory: Option<RuntimeStartupMemoryReport>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeGenerationMemoryReport {
    pub status: String,
    pub before: ProcessMemorySnapshot,
    pub after: ProcessMemorySnapshot,
    pub resident_delta_bytes: Option<i64>,
    pub virtual_delta_bytes: Option<i64>,
    pub captured_at_unix_seconds: u64,
    pub peak_status: String,
    pub peak_resident_bytes: Option<u64>,
    pub peak_virtual_bytes: Option<u64>,
    /// Includes the before/after boundary readings and successful interval samples.
    pub peak_sample_count: u64,
    /// Successful interval samples taken while generation was in progress.
    pub samples_during_generation: u64,
    pub sampling_interval_ms: u64,
}

#[derive(Clone, Debug, Default)]
struct PeakMemoryAccumulator {
    resident_bytes: Option<u64>,
    virtual_bytes: Option<u64>,
    sample_count: u64,
    interval_sample_count: u64,
}

impl PeakMemoryAccumulator {
    fn observe(&mut self, snapshot: &ProcessMemorySnapshot, interval_sample: bool) {
        if snapshot.status != "detected" {
            return;
        }
        if let Some(value) = snapshot.resident_bytes {
            self.resident_bytes = Some(self.resident_bytes.map_or(value, |peak| peak.max(value)));
        }
        if let Some(value) = snapshot.virtual_bytes {
            self.virtual_bytes = Some(self.virtual_bytes.map_or(value, |peak| peak.max(value)));
        }
        self.sample_count = self.sample_count.saturating_add(1);
        if interval_sample {
            self.interval_sample_count = self.interval_sample_count.saturating_add(1);
        }
    }
}

const GENERATION_MEMORY_SAMPLE_INTERVAL: Duration = Duration::from_millis(250);

fn memory_delta_bytes(before: Option<u64>, after: Option<u64>) -> Option<i64> {
    let (before, after) = (before?, after?);
    if after >= before {
        i64::try_from(after - before).ok()
    } else {
        i64::try_from(before - after).ok().map(|delta| -delta)
    }
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
                    startup_memory: None,
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

    /// Samples only the managed llama-server process.
    pub fn process_memory_snapshot(&self) -> Result<ProcessMemorySnapshot, String> {
        let mut slot = self.inner.session.lock()
            .map_err(|_| "Runtime session is unavailable.".to_owned())?;
        match slot.as_mut() {
            Some(session) => Ok(session.process_memory_snapshot()),
            None => Ok(ProcessMemorySnapshot {
                status: "not_running".to_owned(), resident_bytes: None, virtual_bytes: None,
            }),
        }
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
                let startup_memory = session.startup_memory_report().clone();
                *self.inner.session.lock()
                    .map_err(|_| "Runtime session is unavailable.".to_owned())? = Some(session);
                {
                    let mut snapshot = self.inner.status.lock()
                        .map_err(|_| "Runtime status is unavailable.".to_owned())?;
                    snapshot.startup_memory = Some(startup_memory);
                }
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
    ) -> Result<RuntimeGenerationMemoryReport, String>
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

        let before_memory = session.process_memory_snapshot();
        let sampling_target = session.process_memory_sampling_target();
        let peak = Arc::new(Mutex::new(PeakMemoryAccumulator::default()));
        if let Ok(mut values) = peak.lock() {
            values.observe(&before_memory, false);
        }

        // The monitor receives only the PID of the managed child and queries
        // that PID directly. It never enumerates unrelated processes.
        let stop_sampling = Arc::new(AtomicBool::new(false));
        let sampler: Option<JoinHandle<()>> = sampling_target.map(|pid| {
            let stop = Arc::clone(&stop_sampling);
            let peak_values = Arc::clone(&peak);
            thread::spawn(move || {
                while !stop.load(Ordering::Relaxed) {
                    thread::sleep(GENERATION_MEMORY_SAMPLE_INTERVAL);
                    if stop.load(Ordering::Relaxed) {
                        break;
                    }
                    let snapshot = collect_known_process_memory(pid);
                    if let Ok(mut values) = peak_values.lock() {
                        values.observe(&snapshot, true);
                    }
                }
            })
        });

        let result = session.generate_streaming(&request, &cancellation, |delta| {
            on_delta(delta);
        });

        stop_sampling.store(true, Ordering::Relaxed);
        if let Some(handle) = sampler {
            let _ = handle.join();
        }
        let after_memory = session.process_memory_snapshot();
        if let Ok(mut values) = peak.lock() {
            values.observe(&after_memory, false);
        }
        let peak_values = peak.lock().map(|values| values.clone()).unwrap_or_default();
        let report = RuntimeGenerationMemoryReport {
            status: if before_memory.status == "detected" && after_memory.status == "detected" {
                "complete".to_owned()
            } else {
                "partial".to_owned()
            },
            resident_delta_bytes: memory_delta_bytes(before_memory.resident_bytes, after_memory.resident_bytes),
            virtual_delta_bytes: memory_delta_bytes(before_memory.virtual_bytes, after_memory.virtual_bytes),
            peak_status: if peak_values.resident_bytes.is_some() || peak_values.virtual_bytes.is_some() {
                "detected".to_owned()
            } else if sampling_target.is_some() {
                "unavailable".to_owned()
            } else {
                "not_running".to_owned()
            },
            peak_resident_bytes: peak_values.resident_bytes,
            peak_virtual_bytes: peak_values.virtual_bytes,
            peak_sample_count: peak_values.sample_count,
            samples_during_generation: peak_values.interval_sample_count,
            sampling_interval_ms: GENERATION_MEMORY_SAMPLE_INTERVAL.as_millis() as u64,
            before: before_memory,
            after: after_memory,
            captured_at_unix_seconds: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        if let Ok(mut active) = self.inner.active_cancellation.lock() {
            *active = None;
        }

        match result {
            Ok(()) => {
                self.set_status(session.status(), None)?;
                Ok(report)
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
    fn peak_accumulator_tracks_maximum_memory_and_interval_samples() {
        let mut peak = PeakMemoryAccumulator::default();
        let sample = |resident, virtual_bytes| ProcessMemorySnapshot {
            status: "detected".to_owned(),
            resident_bytes: resident,
            virtual_bytes,
        };
        peak.observe(&sample(Some(100), Some(500)), false);
        peak.observe(&sample(Some(250), Some(450)), true);
        peak.observe(&sample(Some(200), Some(700)), true);
        peak.observe(&ProcessMemorySnapshot {
            status: "unavailable".to_owned(), resident_bytes: Some(999), virtual_bytes: Some(999),
        }, true);
        assert_eq!(peak.resident_bytes, Some(250));
        assert_eq!(peak.virtual_bytes, Some(700));
        assert_eq!(peak.sample_count, 3);
        assert_eq!(peak.interval_sample_count, 2);
    }

    #[test]
    fn peak_accumulator_handles_missing_values_and_empty_samples() {
        let mut peak = PeakMemoryAccumulator::default();
        peak.observe(&ProcessMemorySnapshot {
            status: "detected".to_owned(), resident_bytes: None, virtual_bytes: Some(42),
        }, true);
        assert_eq!(peak.resident_bytes, None);
        assert_eq!(peak.virtual_bytes, Some(42));
        assert_eq!(peak.sample_count, 1);
    }

    #[test]
    fn memory_delta_reports_positive_negative_zero_and_missing_samples() {
        assert_eq!(memory_delta_bytes(Some(100), Some(150)), Some(50));
        assert_eq!(memory_delta_bytes(Some(150), Some(100)), Some(-50));
        assert_eq!(memory_delta_bytes(Some(100), Some(100)), Some(0));
        assert_eq!(memory_delta_bytes(None, Some(100)), None);
        assert_eq!(memory_delta_bytes(Some(100), None), None);
    }

    #[test]
    fn memory_delta_rejects_values_outside_signed_range() {
        assert_eq!(memory_delta_bytes(Some(0), Some(u64::MAX)), None);
    }

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
            system_prompt: String::new(),
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
