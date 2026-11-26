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


/// Minimal process boundary so recovery can be tested without launching a real model.
trait ManagedServerProcess: Send {
    fn is_running(&mut self) -> std::io::Result<bool>;
    fn process_id(&self) -> Option<u32>;
    fn stop(&mut self) -> std::io::Result<()>;
}

impl ManagedServerProcess for LlamaServerProcess {
    fn is_running(&mut self) -> std::io::Result<bool> {
        LlamaServerProcess::is_running(self)
    }

    fn process_id(&self) -> Option<u32> {
        LlamaServerProcess::process_id(self)
    }

    fn stop(&mut self) -> std::io::Result<()> {
        LlamaServerProcess::stop(self)
    }
}

/// Owns the child process and its loopback client as one lifecycle-managed unit.
///
/// The session is synchronous and bounded by the supplied HTTP/startup timeouts.
/// Dropping it also drops the process supervisor, which performs best-effort cleanup.
pub struct LlamaRuntimeSession {
    process: Box<dyn ManagedServerProcess>,
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

        let mut process: Box<dyn ManagedServerProcess> = Box::new(
            LlamaServerProcess::start(config, executable, model)
                .map_err(RuntimeSessionError::ProcessStart)?,
        );
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

    /// Returns a PID only for the live managed child; it is never exposed to the UI.
    pub fn process_memory_sampling_target(&mut self) -> Option<u32> {
        match self.process.is_running() {
            Ok(true) => self.process.process_id(),
            _ => None,
        }
    }

    /// Samples memory for the managed child only; the PID remains internal.
    pub fn process_memory_snapshot(&mut self) -> crate::system_info::ProcessMemorySnapshot {
        match self.process.is_running() {
            Ok(true) => match self.process.process_id() {
                Some(pid) => crate::system_info::collect_known_process_memory(pid),
                None => crate::system_info::ProcessMemorySnapshot {
                    status: "unavailable".to_owned(), resident_bytes: None, virtual_bytes: None,
                },
            },
            Ok(false) => crate::system_info::ProcessMemorySnapshot {
                status: "not_running".to_owned(), resident_bytes: None, virtual_bytes: None,
            },
            Err(_) => crate::system_info::ProcessMemorySnapshot {
                status: "unavailable".to_owned(), resident_bytes: None, virtual_bytes: None,
            },
        }
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
        self.recover_with(
            startup_timeout,
            poll_interval,
            policy,
            |config, executable, model| {
                LlamaServerProcess::start(config, executable, model)
                    .map(|process| Box::new(process) as Box<dyn ManagedServerProcess>)
            },
            executable,
            model,
        )
    }

    fn recover_with<F>(
        &mut self,
        startup_timeout: Duration,
        poll_interval: Duration,
        policy: RecoveryPolicy,
        mut start_process: F,
        executable: &Path,
        model: &Path,
    ) -> Result<(), RuntimeSessionError>
    where
        F: FnMut(
            &crate::runtime::RuntimeConfig,
            &Path,
            &Path,
        ) -> Result<Box<dyn ManagedServerProcess>, ProcessStartError>,
    {
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

            let mut process = match start_process(&self.config, executable, model) {
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

#[cfg(test)]
mod recovery_integration_tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, atomic::{AtomicBool, AtomicUsize, Ordering}};
    use std::thread;

    struct FakeProcess {
        running: Arc<AtomicBool>,
        stop_count: Arc<AtomicUsize>,
    }

    impl ManagedServerProcess for FakeProcess {
        fn is_running(&mut self) -> std::io::Result<bool> {
            Ok(self.running.load(Ordering::SeqCst))
        }

        fn process_id(&self) -> Option<u32> {
            // The fake process does not own an operating-system child.
            None
        }

        fn stop(&mut self) -> std::io::Result<()> {
            self.stop_count.fetch_add(1, Ordering::SeqCst);
            self.running.store(false, Ordering::SeqCst);
            Ok(())
        }
    }

    fn ready_server() -> (u16, thread::JoinHandle<()>) {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0_u8; 1024];
            let _ = stream.read(&mut request);
            let body = r#"{"status":"ok"}"#;
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            ).unwrap();
            stream.flush().unwrap();
        });
        (port, handle)
    }

    fn failed_session(port: u16) -> (LlamaRuntimeSession, Arc<AtomicUsize>) {
        let config = crate::runtime::RuntimeConfig {
            host: "127.0.0.1".to_owned(),
            port,
            ..crate::runtime::RuntimeConfig::default()
        };
        let client = LlamaHttpClient::new(&config, Duration::from_secs(1)).unwrap();
        let mut lifecycle = RuntimeLifecycle::default();
        lifecycle.transition(RuntimeStatus::Stopped).unwrap();
        lifecycle.transition(RuntimeStatus::Starting).unwrap();
        lifecycle.transition(RuntimeStatus::Failed).unwrap();
        let original_running = Arc::new(AtomicBool::new(true));
        let stop_count = Arc::new(AtomicUsize::new(0));
        let process = FakeProcess {
            running: original_running,
            stop_count: stop_count.clone(),
        };
        (
            LlamaRuntimeSession {
                process: Box::new(process),
                client,
                lifecycle,
                config,
            },
            stop_count,
        )
    }

    fn no_delay_policy(attempts: u8) -> RecoveryPolicy {
        RecoveryPolicy { max_attempts: attempts, retry_delay: Duration::ZERO }
    }

    #[test]
    fn recovery_replaces_failed_process_and_returns_to_ready() {
        let (port, server) = ready_server();
        let (mut session, old_stop_count) = failed_session(port);
        let new_running = Arc::new(AtomicBool::new(true));
        let new_stop_count = Arc::new(AtomicUsize::new(0));
        let result = session.recover_with(
            Duration::from_secs(1),
            Duration::from_millis(5),
            no_delay_policy(1),
            {
                let running = new_running.clone();
                let stops = new_stop_count.clone();
                move |_, _, _| Ok(Box::new(FakeProcess {
                    running: running.clone(),
                    stop_count: stops.clone(),
                }) as Box<dyn ManagedServerProcess>)
            },
            Path::new("fake-server"),
            Path::new("fake-model"),
        );
        assert!(result.is_ok());
        assert_eq!(session.status(), RuntimeStatus::Ready);
        assert_eq!(old_stop_count.load(Ordering::SeqCst), 1);
        assert_eq!(new_stop_count.load(Ordering::SeqCst), 0);
        server.join().unwrap();
    }

    #[test]
    fn recovery_retries_startup_failure_then_succeeds() {
        let (port, server) = ready_server();
        let (mut session, _) = failed_session(port);
        let attempts = Arc::new(AtomicUsize::new(0));
        let running = Arc::new(AtomicBool::new(true));
        let stops = Arc::new(AtomicUsize::new(0));
        let result = session.recover_with(
            Duration::from_secs(1),
            Duration::from_millis(5),
            no_delay_policy(2),
            {
                let attempts = attempts.clone();
                let running = running.clone();
                let stops = stops.clone();
                move |_, _, _| {
                    if attempts.fetch_add(1, Ordering::SeqCst) == 0 {
                        Err(ProcessStartError::ExecutableNotFound)
                    } else {
                        Ok(Box::new(FakeProcess {
                            running: running.clone(),
                            stop_count: stops.clone(),
                        }) as Box<dyn ManagedServerProcess>)
                    }
                }
            },
            Path::new("fake-server"),
            Path::new("fake-model"),
        );
        assert!(result.is_ok());
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
        assert_eq!(session.status(), RuntimeStatus::Ready);
        server.join().unwrap();
    }

    #[test]
    fn recovery_exhaustion_reports_attempt_count_and_leaves_failed_state() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let (mut session, _) = failed_session(port);
        let attempts = Arc::new(AtomicUsize::new(0));
        let result = session.recover_with(
            Duration::from_millis(20),
            Duration::from_millis(5),
            no_delay_policy(3),
            {
                let attempts = attempts.clone();
                move |_, _, _| {
                    attempts.fetch_add(1, Ordering::SeqCst);
                    Err(ProcessStartError::ExecutableNotFound)
                }
            },
            Path::new("missing-server"),
            Path::new("missing-model"),
        );
        assert!(matches!(
            result,
            Err(RuntimeSessionError::RecoveryExhausted { attempts: 3, .. })
        ));
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
        assert_eq!(session.status(), RuntimeStatus::Failed);
    }

    #[test]
    fn recovery_stops_each_replacement_that_never_becomes_ready() {
        // Reserve and release a loopback port so every health request is refused.
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let (mut session, old_stop_count) = failed_session(port);
        let attempts = Arc::new(AtomicUsize::new(0));
        let replacement_stop_count = Arc::new(AtomicUsize::new(0));
        let result = session.recover_with(
            Duration::from_millis(30),
            Duration::from_millis(5),
            no_delay_policy(2),
            {
                let attempts = attempts.clone();
                let stops = replacement_stop_count.clone();
                move |_, _, _| {
                    attempts.fetch_add(1, Ordering::SeqCst);
                    Ok(Box::new(FakeProcess {
                        running: Arc::new(AtomicBool::new(true)),
                        stop_count: stops.clone(),
                    }) as Box<dyn ManagedServerProcess>)
                }
            },
            Path::new("fake-server"),
            Path::new("fake-model"),
        );

        assert!(matches!(
            result,
            Err(RuntimeSessionError::RecoveryExhausted { attempts: 2, .. })
        ));
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
        assert_eq!(old_stop_count.load(Ordering::SeqCst), 1);
        assert_eq!(replacement_stop_count.load(Ordering::SeqCst), 2);
        assert_eq!(session.status(), RuntimeStatus::Failed);
    }

    #[test]
    fn recovery_is_rejected_while_session_is_ready() {
        let (mut session, _) = failed_session(1);
        session.lifecycle.transition(RuntimeStatus::Stopped).unwrap();
        session.lifecycle.transition(RuntimeStatus::Starting).unwrap();
        session.lifecycle.transition(RuntimeStatus::Ready).unwrap();
        let result = session.recover_with(
            Duration::from_secs(1),
            Duration::from_millis(5),
            no_delay_policy(1),
            |_, _, _| Err(ProcessStartError::ExecutableNotFound),
            Path::new("fake-server"),
            Path::new("fake-model"),
        );
        assert!(matches!(result, Err(RuntimeSessionError::InvalidState)));
        assert_eq!(session.status(), RuntimeStatus::Ready);
    }
}

