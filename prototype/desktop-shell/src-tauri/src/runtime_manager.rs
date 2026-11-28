//! Thread-safe coordinator for the local inference session.
//!
//! Runtime work is serialized independently from status reads. A long startup
//! or generation therefore cannot prevent the UI from reading status or
//! signalling cancellation. This module never downloads assets or uses a
//! network endpoint other than the validated loopback runtime.

use std::{path::Path, sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex}, thread::{self, JoinHandle}, time::{Duration, Instant, SystemTime, UNIX_EPOCH}};

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
    pub session: Option<RuntimeSessionSnapshot>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSessionSnapshot {
    pub model_file_size_bytes: Option<u64>,
    pub context_tokens: u32,
    pub cpu_threads: Option<u16>,
    pub uptime_seconds: u64,
}

struct RuntimeSessionDetails {
    model_file_size_bytes: Option<u64>,
    context_tokens: u32,
    cpu_threads: Option<u16>,
    started_at: Instant,
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
    session_details: Mutex<Option<RuntimeSessionDetails>>,
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
                    session: None,
                }),
                session_details: Mutex::new(None),
                active_cancellation: Mutex::new(None),
            }),
        }
    }
}

impl RuntimeManager {
    /// Acquires the runtime's exclusive operation gate for a database restore.
    ///
    /// This is deliberately non-blocking: restore must be refused rather than
    /// queued behind an active start, recovery, or generation operation. The
    /// returned guard must be held until the restore has fully completed or
    /// rolled back. A ready runtime is also rejected; the user must stop the
    /// local engine before a restore can begin. Runtime operations use this
    /// same mutex, so a new operation cannot start while the guard is held.
    pub fn try_acquire_restore_gate(&self) -> Result<std::sync::MutexGuard<'_, ()>, String> {
        let guard = self.inner.operation.try_lock().map_err(|error| match error {
            std::sync::TryLockError::WouldBlock =>
                "The local AI runtime is busy starting, generating, stopping, or recovering. Wait for it to finish and try again.".to_owned(),
            std::sync::TryLockError::Poisoned(_) =>
                "The local AI runtime operation lock is unavailable. Restart midSunday before attempting a restore.".to_owned(),
        })?;

        let status = self.inner.status.lock()
            .map_err(|_| "The local AI runtime status is unavailable. Restart midSunday before attempting a restore.".to_owned())?;
        if matches!(status.status.as_str(), "starting" | "ready" | "generating") {
            return Err("Stop the local AI engine before restoring a database backup. No database data was changed.".to_owned());
        }

        Ok(guard)
    }

    pub fn snapshot(&self) -> Result<RuntimeStatusSnapshot, String> {
        let mut snapshot = self.inner.status.lock()
            .map(|status| status.clone())
            .map_err(|_| "Runtime status is unavailable.".to_owned())?;

        // Poll only the one child process owned by this runtime while idle.
        // Generation holds the session mutex for the full streaming request, so
        // trying to inspect it here would block the UI's status refresh. A
        // generation-time child failure is instead surfaced by the generation
        // result; the next idle refresh checks the child again if it remains ready.
        let child_exited = if snapshot.status == "ready" {
            let mut session_slot = self.inner.session.lock()
                .map_err(|_| "Runtime session is unavailable.".to_owned())?;
            session_slot.as_mut()
                .map(|session| session.process_memory_snapshot().status == "not_running")
                .unwrap_or(false)
        } else {
            false
        };

        if child_exited {
            if let Ok(mut details) = self.inner.session_details.lock() {
                *details = None;
            }
            let mut current = self.inner.status.lock()
                .map_err(|_| "Runtime status is unavailable.".to_owned())?;
            if matches!(current.status.as_str(), "ready" | "generating") {
                current.status = "failed".to_owned();
                current.detail = Some("The local engine process exited unexpectedly. Check the configured server file and model, then start the engine again.".to_owned());
            }
            snapshot = current.clone();
            snapshot.session = None;
            return Ok(snapshot);
        }

        let details = self.inner.session_details.lock()
            .map_err(|_| "Runtime session details are unavailable.".to_owned())?;
        snapshot.session = details.as_ref().map(|details| RuntimeSessionSnapshot {
            model_file_size_bytes: details.model_file_size_bytes,
            context_tokens: details.context_tokens,
            cpu_threads: details.cpu_threads,
            uptime_seconds: u64::try_from(details.started_at.elapsed().as_secs()).unwrap_or(u64::MAX),
        });
        Ok(snapshot)
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
        let slot = self.inner.session.lock()
            .map_err(|_| "Runtime session is unavailable.".to_owned())?;
        if slot.is_some() {
            return Err("A runtime session already exists; stop it or use recovery first.".to_owned());
        }
        drop(slot);

        self.set_status(RuntimeStatus::Starting, None)?;
        if let Ok(mut details) = self.inner.session_details.lock() { *details = None; }
        let model_file_size_bytes = std::fs::metadata(model).ok().map(|metadata| metadata.len());
        match LlamaRuntimeSession::start(
            &config, Path::new(executable), Path::new(model),
            startup_timeout, poll_interval, request_timeout,
        ) {
            Ok(session) => {
                let startup_memory = session.startup_memory_report().clone();
                *self.inner.session.lock()
                    .map_err(|_| "Runtime session is unavailable.".to_owned())? = Some(session);
                *self.inner.session_details.lock()
                    .map_err(|_| "Runtime session details are unavailable.".to_owned())? = Some(RuntimeSessionDetails {
                        model_file_size_bytes,
                        context_tokens: config.context_tokens,
                        cpu_threads: config.cpu_threads,
                        started_at: Instant::now(),
                    });
                {
                    let mut snapshot = self.inner.status.lock()
                        .map_err(|_| "Runtime status is unavailable.".to_owned())?;
                    snapshot.startup_memory = Some(startup_memory);
                }
                self.set_status(RuntimeStatus::Ready, None)
            }
            Err(error) => {
                if let Ok(mut details) = self.inner.session_details.lock() { *details = None; }
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
                let status = session.status();
                if status == RuntimeStatus::Failed {
                    if let Ok(mut details) = self.inner.session_details.lock() { *details = None; }
                }
                self.set_status(status, Some(detail.clone()))?;
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
            *self.inner.session_details.lock()
                .map_err(|_| "Runtime session details are unavailable.".to_owned())? = None;
            return Ok(());
        };

        match session.stop() {
            Ok(()) => {
                *slot = None;
                *self.inner.session_details.lock()
                    .map_err(|_| "Runtime session details are unavailable.".to_owned())? = None;
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
    fn restore_gate_allows_only_inactive_runtime_states() {
        for status in ["notInstalled", "stopped", "failed"] {
            let manager = RuntimeManager::default();
            {
                let mut snapshot = manager.inner.status.lock().unwrap();
                snapshot.status = status.to_owned();
            }
            let guard = manager.try_acquire_restore_gate()
                .unwrap_or_else(|error| panic!("restore gate should allow {status}: {error}"));
            drop(guard);
        }
    }

    #[test]
    fn restore_gate_rejects_starting_ready_and_generating_runtime_states() {
        for status in ["starting", "ready", "generating"] {
            let manager = RuntimeManager::default();
            {
                let mut snapshot = manager.inner.status.lock().unwrap();
                snapshot.status = status.to_owned();
            }
            let error = manager.try_acquire_restore_gate().unwrap_err();
            assert!(error.contains("Stop the local AI engine"), "unexpected error for {status}: {error}");
        }
    }

    #[test]
    fn restore_gate_fails_fast_when_another_runtime_operation_is_active() {
        let manager = RuntimeManager::default();
        let _operation = manager.inner.operation.lock().unwrap();
        let error = manager.try_acquire_restore_gate().unwrap_err();
        assert!(error.contains("runtime is busy"));
    }

    #[test]
    fn restore_gate_prevents_new_runtime_operations_until_released() {
        let manager = RuntimeManager::default();
        let guard = manager.try_acquire_restore_gate().unwrap();
        assert!(manager.inner.operation.try_lock().is_err());
        drop(guard);
        assert!(manager.inner.operation.try_lock().is_ok());
    }

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
    fn snapshot_reports_only_non_sensitive_active_session_resource_details() {
        let manager = RuntimeManager::default();
        *manager.inner.session_details.lock().unwrap() = Some(RuntimeSessionDetails {
            model_file_size_bytes: Some(4 * 1024 * 1024 * 1024),
            context_tokens: 4096,
            cpu_threads: Some(6),
            started_at: Instant::now() - Duration::from_secs(12),
        });
        let snapshot = manager.snapshot().unwrap();
        let session = snapshot.session.unwrap();
        assert_eq!(session.model_file_size_bytes, Some(4 * 1024 * 1024 * 1024));
        assert_eq!(session.context_tokens, 4096);
        assert_eq!(session.cpu_threads, Some(6));
        assert!(session.uptime_seconds >= 12);
        // The snapshot intentionally contains no local path or process identifier.
    }

    #[test]
    fn unexpected_process_exit_is_reported_as_recoverable_failure() {
        let manager = RuntimeManager::default();
        manager.set_status(RuntimeStatus::Ready, None).unwrap();
        let mut snapshot = manager.snapshot().unwrap();
        assert_eq!(snapshot.status, "ready");
        // Without a managed session there is no child to inspect, so status is not
        // incorrectly changed merely because process telemetry is unavailable.
        assert_eq!(snapshot.detail, None);
    }

    #[test]
    fn snapshot_hides_session_details_after_the_session_is_cleared() {
        let manager = RuntimeManager::default();
        *manager.inner.session_details.lock().unwrap() = Some(RuntimeSessionDetails {
            model_file_size_bytes: None,
            context_tokens: 2048,
            cpu_threads: None,
            started_at: Instant::now(),
        });
        *manager.inner.session_details.lock().unwrap() = None;
        assert!(manager.snapshot().unwrap().session.is_none());
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
    fn generating_status_snapshot_does_not_wait_for_the_session_lock() {
        let manager = RuntimeManager::default();
        manager.set_status(RuntimeStatus::Generating, None).unwrap();
        let session_guard = manager.inner.session.lock().unwrap();
        let worker_manager = manager.clone();
        let (snapshot_tx, snapshot_rx) = mpsc::channel();

        let worker = thread::spawn(move || {
            snapshot_tx.send(worker_manager.snapshot()).unwrap();
        });

        let result = snapshot_rx.recv_timeout(Duration::from_millis(200));
        drop(session_guard);
        worker.join().unwrap();

        let snapshot = result.expect("generating status must remain readable during inference").unwrap();
        assert_eq!(snapshot.status, "generating");
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
