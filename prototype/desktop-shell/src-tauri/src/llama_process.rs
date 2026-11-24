//! Safe child-process supervisor for a locally installed llama.cpp server.
//!
//! This module never invokes a shell, never downloads binaries/models, and
//! only constructs a loopback-bound server command from validated inputs.
//! HTTP readiness and inference are intentionally handled by a later adapter.

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

        let executable = canonical_file(executable)
            .ok_or(ProcessStartError::ExecutableNotFound)?;
        let model = canonical_file(model).ok_or(ProcessStartError::ModelNotFound)?;

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

    /// Returns false when the child has already exited.
    pub fn is_running(&mut self) -> Result<bool, std::io::Error> {
        match self.child.as_mut() {
            Some(child) => Ok(child.try_wait()?.is_none()),
            None => Ok(false),
        }
    }

    /// Stop the managed process and wait for the operating system to reap it.
    pub fn stop(&mut self) -> Result<(), std::io::Error> {
        if let Some(mut child) = self.child.take() {
            match child.try_wait()? {
                Some(_) => {}
                None => {
                    child.kill()?;
                    let _ = child.wait()?;
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

fn canonical_file(path: &Path) -> Option<PathBuf> {
    let canonical = path.canonicalize().ok()?;
    canonical.is_file().then_some(canonical)
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
        assert!(canonical_file(Path::new("/definitely/not/a/real/model.gguf")).is_none());
    }
}
