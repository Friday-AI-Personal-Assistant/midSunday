//! Safe child-process supervisor for a locally installed llama.cpp server.
//!
//! This module never invokes a shell, never downloads binaries/models, and
//! only constructs a loopback-bound server command from validated inputs.
//! HTTP readiness and inference are handled by the managed runtime session.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use crate::runtime::{RuntimeConfig, RuntimeConfigError};

#[derive(Debug)]
pub enum ProcessStartError {
    InvalidConfiguration(RuntimeConfigError),
    ExecutableNotFound,
    ModelNotFound,
    ExecutableNotFile,
    ModelNotFile,
    SpawnFailed(std::io::Error),
}

pub struct LlamaServerProcess {
    child: Option<Child>,
}

impl LlamaServerProcess {
    /// Start a user-installed llama-server executable with a local model.
    /// Both paths must identify existing regular files.
    pub fn start(
        config: &RuntimeConfig,
        executable: &Path,
        model: &Path,
    ) -> Result<Self, ProcessStartError> {
        config
            .validate()
            .map_err(ProcessStartError::InvalidConfiguration)?;

        let executable = canonical_file(executable).map_err(|error| match error {
            CanonicalFileError::NotFound => ProcessStartError::ExecutableNotFound,
            CanonicalFileError::NotFile => ProcessStartError::ExecutableNotFile,
        })?;
        let model = canonical_file(model).map_err(|error| match error {
            CanonicalFileError::NotFound => ProcessStartError::ModelNotFound,
            CanonicalFileError::NotFile => ProcessStartError::ModelNotFile,
        })?;

        let args = build_server_arguments(config, &model);
        let child = Command::new(executable)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(ProcessStartError::SpawnFailed)?;

        Ok(Self { child: Some(child) })
    }

    /// Returns the PID of the child started by this supervisor.
    pub fn process_id(&self) -> Option<u32> {
        self.child.as_ref().map(|child| child.id())
    }

    /// Returns false when the child has exited, clearing the reaped handle.
    pub fn is_running(&mut self) -> Result<bool, std::io::Error> {
        let status = match self.child.as_mut() {
            Some(child) => child.try_wait()?,
            None => return Ok(false),
        };
        if status.is_some() {
            // try_wait has collected the exit status. Drop the now-reaped
            // handle so callers cannot mistake its stale PID for a live server.
            self.child = None;
            Ok(false)
        } else {
            Ok(true)
        }
    }

    /// Stop the managed process and wait for the operating system to reap it.
    pub fn stop(&mut self) -> Result<(), std::io::Error> {
        if let Some(mut child) = self.child.take() {
            match child.try_wait() {
                Ok(Some(_)) => {}
                Ok(None) => {
                    if let Err(kill_error) = child.kill() {
                        // The process may have exited between try_wait and kill.
                        // Re-check so that an already-exited child is still reaped.
                        match child.try_wait() {
                            Ok(Some(_)) => {}
                            Ok(None) => return Err(kill_error),
                            Err(status_error) => return Err(status_error),
                        }
                    } else {
                        let _ = child.wait()?;
                    }
                }
                Err(error) => {
                    // If status inspection fails, still attempt to terminate
                    // the child before returning the original inspection error.
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(error);
                }
            }
        }
        Ok(())
    }
}

impl Drop for LlamaServerProcess {
    fn drop(&mut self) {
        // Best-effort cleanup: a dropped app-side handle must not leave an
        // unmanaged inference server running in the background.
        let _ = self.stop();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CanonicalFileError {
    NotFound,
    NotFile,
}

fn canonical_file(path: &Path) -> Result<PathBuf, CanonicalFileError> {
    let canonical = path.canonicalize().map_err(|_| CanonicalFileError::NotFound)?;
    if !canonical.is_file() {
        return Err(CanonicalFileError::NotFile);
    }
    Ok(canonical)
}

/// Build a fixed argument list; user/model text is never interpreted by a shell.
fn build_server_arguments(config: &RuntimeConfig, model: &Path) -> Vec<OsString> {
    let mut args = vec![
        OsString::from("-m"),
        model.as_os_str().to_owned(),
        OsString::from("--host"),
        OsString::from(&config.host),
        OsString::from("--port"),
        OsString::from(config.port.to_string()),
        OsString::from("-c"),
        OsString::from(config.context_tokens.to_string()),
    ];

    if let Some(threads) = config.cpu_threads {
        args.push(OsString::from("--threads"));
        args.push(OsString::from(threads.to_string()));
    }

    args
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    // Also launched as a child test process by the supervisor tests.
    #[test]
    fn controlled_child_process_fixture() {
        match std::env::var("MIDSUNDAY_TEST_CHILD_MODE").as_deref() {
            Ok("sleep") => thread::sleep(Duration::from_secs(30)),
            Ok("exit") => {}
            _ => {}
        }
    }

    fn spawn_test_child(mode: &str) -> Child {
        Command::new(std::env::current_exe().expect("test executable path"))
            .arg("--exact")
            .arg("llama_process::tests::controlled_child_process_fixture")
            .arg("--nocapture")
            .env("MIDSUNDAY_TEST_CHILD_MODE", mode)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn controlled child")
    }

    #[test]
    fn supervisor_detects_a_child_that_exits_unexpectedly() {
        let child = spawn_test_child("exit");
        let mut process = LlamaServerProcess { child: Some(child) };
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            if !process.is_running().expect("inspect child") { break; }
            assert!(std::time::Instant::now() < deadline, "child did not exit in time");
            thread::sleep(Duration::from_millis(10));
        }
        assert!(!process.is_running().expect("inspect exited child again"));
        assert_eq!(process.process_id(), None, "an exited and reaped child must not retain a stale PID");
        process.stop().expect("stop after child was already reaped");
    }

    #[test]
    fn supervisor_stop_is_idempotent_and_reaps_child() {
        let child = spawn_test_child("sleep");
        let mut process = LlamaServerProcess { child: Some(child) };
        assert!(process.is_running().expect("child should be running"));
        process.stop().expect("stop child");
        assert!(!process.is_running().expect("stopped process"));
        process.stop().expect("second stop is harmless");
    }

    #[test]
    fn start_reports_directories_as_not_files_without_spawning() {
        let directory = tempfile::tempdir().unwrap();
        let executable_directory = directory.path().join("server-directory");
        let model_directory = directory.path().join("model-directory");
        std::fs::create_dir(&executable_directory).unwrap();
        std::fs::create_dir(&model_directory).unwrap();

        let config = RuntimeConfig::default();
        assert!(matches!(
            LlamaServerProcess::start(&config, &executable_directory, &model_directory),
            Err(ProcessStartError::ExecutableNotFile)
        ));

        let executable_file = directory.path().join("llama-server");
        std::fs::write(&executable_file, b"not launched by this test").unwrap();
        assert!(matches!(
            LlamaServerProcess::start(&config, &executable_file, &model_directory),
            Err(ProcessStartError::ModelNotFile)
        ));
    }

    #[test]
    fn arguments_bind_only_to_loopback_and_use_explicit_model_path() {
        let config = RuntimeConfig::default();
        let model = Path::new("/models/test model.gguf");
        let args = build_server_arguments(&config, model);
        let args: Vec<String> = args
            .into_iter()
            .map(|value| value.to_string_lossy().into_owned())
            .collect();

        assert_eq!(
            args,
            vec![
                "-m",
                "/models/test model.gguf",
                "--host",
                "127.0.0.1",
                "--port",
                "9931",
                "-c",
                "2048"
            ]
        );
    }

    #[test]
    fn optional_cpu_threads_are_passed_as_a_separate_argument() {
        let config = RuntimeConfig {
            cpu_threads: Some(4),
            ..RuntimeConfig::default()
        };
        let args = build_server_arguments(&config, Path::new("/models/model.gguf"));
        let args: Vec<String> = args
            .into_iter()
            .map(|value| value.to_string_lossy().into_owned())
            .collect();

        assert!(args.windows(2).any(|pair| pair == ["--threads", "4"]));
    }

    #[test]
    fn executable_and_model_paths_must_be_regular_files() {
        assert_eq!(
            canonical_file(Path::new("/definitely/not/a/real/model.gguf")),
            Err(CanonicalFileError::NotFound)
        );
    }
}
