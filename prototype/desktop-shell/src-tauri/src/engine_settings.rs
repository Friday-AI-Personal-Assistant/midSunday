//! User-controlled local inference settings persisted on this device.
use serde::{Deserialize, Serialize};
use std::{path::{Path, PathBuf}, sync::Mutex};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EngineSettings {
    pub executable_path: String,
    pub model_path: String,
    pub context_tokens: u32,
    pub cpu_threads: Option<u16>,
}

impl Default for EngineSettings {
    fn default() -> Self {
        Self { executable_path: String::new(), model_path: String::new(), context_tokens: 2048, cpu_threads: None }
    }
}

impl EngineSettings {
    pub fn validate(&self) -> Result<(), String> {
        for (label, value) in [("llama-server path", &self.executable_path), ("GGUF model path", &self.model_path)] {
            if value.len() > 4096 || value.contains('\0') {
                return Err(format!("The {label} is invalid or too long."));
            }
        }
        if !(512..=32768).contains(&self.context_tokens) {
            return Err("Context length must be between 512 and 32,768 tokens.".to_owned());
        }
        if self.cpu_threads.is_some_and(|threads| !(1..=256).contains(&threads)) {
            return Err("CPU threads must be between 1 and 256, or left on automatic.".to_owned());
        }
        Ok(())
    }
}

/// Stores only local file paths and inference tuning values; no prompts or secrets.
pub struct EngineSettingsStore {
    path: PathBuf,
    settings: Mutex<EngineSettings>,
}

impl EngineSettingsStore {
    pub fn open(path: &Path) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| format!("Could not create settings directory: {error}"))?;
        }
        let settings = match std::fs::read_to_string(path) {
            Ok(contents) => {
                let parsed: EngineSettings = serde_json::from_str(&contents)
                    .map_err(|error| format!("Local engine settings could not be read: {error}"))?;
                parsed.validate()?;
                parsed
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => EngineSettings::default(),
            Err(error) => return Err(format!("Could not read local engine settings: {error}")),
        };
        Ok(Self { path: path.to_path_buf(), settings: Mutex::new(settings) })
    }

    pub fn get(&self) -> Result<EngineSettings, String> {
        self.settings.lock().map(|settings| settings.clone())
            .map_err(|_| "Local engine settings are unavailable.".to_owned())
    }

    pub fn save(&self, settings: EngineSettings) -> Result<(), String> {
        settings.validate()?;
        let mut current = self.settings.lock().map_err(|_| "Local engine settings are unavailable.".to_owned())?;
        let contents = serde_json::to_vec_pretty(&settings)
            .map_err(|error| format!("Could not encode local engine settings: {error}"))?;
        let temporary = self.path.with_extension("json.tmp");
        std::fs::write(&temporary, contents)
            .map_err(|error| format!("Could not write local engine settings: {error}"))?;
        if self.path.exists() {
            std::fs::remove_file(&self.path)
                .map_err(|error| format!("Could not replace local engine settings: {error}"))?;
        }
        std::fs::rename(&temporary, &self.path)
            .map_err(|error| format!("Could not finish saving local engine settings: {error}"))?;
        *current = settings;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);

    fn test_path() -> PathBuf {
        std::env::temp_dir().join(format!("midsunday-engine-settings-{}-{}.json", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)))
    }

    #[test]
    fn defaults_are_safe_and_valid() {
        let settings = EngineSettings::default();
        assert_eq!(settings.context_tokens, 2048);
        assert_eq!(settings.cpu_threads, None);
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn rejects_unsafe_resource_values() {
        let mut settings = EngineSettings::default();
        settings.context_tokens = 0;
        assert!(settings.validate().is_err());
        settings.context_tokens = 2048;
        settings.cpu_threads = Some(0);
        assert!(settings.validate().is_err());
        settings.cpu_threads = Some(257);
        assert!(settings.validate().is_err());
    }

    #[test]
    fn saves_and_reloads_settings_between_store_instances() {
        let path = test_path();
        let store = EngineSettingsStore::open(&path).unwrap();
        let settings = EngineSettings {
            executable_path: "/tools/llama-server".to_owned(),
            model_path: "/models/example.gguf".to_owned(),
            context_tokens: 4096,
            cpu_threads: Some(6),
        };
        store.save(settings.clone()).unwrap();
        drop(store);
        let reopened = EngineSettingsStore::open(&path).unwrap();
        assert_eq!(reopened.get().unwrap(), settings);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn malformed_settings_are_not_silently_overwritten() {
        let path = test_path();
        std::fs::write(&path, "{broken").unwrap();
        assert!(EngineSettingsStore::open(&path).is_err());
        let _ = std::fs::remove_file(path);
    }
}
