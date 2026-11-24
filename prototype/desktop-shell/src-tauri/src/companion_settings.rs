//! Device-local companion preferences and named character profiles.
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::{Path, PathBuf}, sync::Mutex};

const SCHEMA_VERSION: u32 = 1;
const MAX_PROFILES: usize = 20;
const MAX_INSTRUCTIONS_BYTES: usize = 16_384;
const BUILT_INS: [&str; 6] = ["best-friend", "girlfriend", "boyfriend", "mentor", "professional", "custom"];

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompanionProfile {
    pub id: String,
    pub name: String,
    pub instructions: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompanionSettings {
    pub schema_version: u32,
    pub selected_personality: String,
    pub custom_instructions: String,
    pub profiles: Vec<CompanionProfile>,
}

impl Default for CompanionSettings {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            selected_personality: "best-friend".to_owned(),
            custom_instructions: String::new(),
            profiles: Vec::new(),
        }
    }
}

impl CompanionSettings {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != SCHEMA_VERSION {
            return Err("Unsupported companion settings version.".to_owned());
        }
        if self.custom_instructions.len() > MAX_INSTRUCTIONS_BYTES {
            return Err("Custom companion instructions exceed the 16 KiB limit.".to_owned());
        }
        if self.profiles.len() > MAX_PROFILES {
            return Err("A maximum of 20 custom characters is allowed.".to_owned());
        }
        let mut ids = HashSet::new();
        let mut names = HashSet::new();
        for profile in &self.profiles {
            if profile.id.is_empty() || profile.id.len() > 64
                || !profile.id.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_') {
                return Err("A custom character has an invalid identifier.".to_owned());
            }
            if !ids.insert(profile.id.clone()) {
                return Err("Custom character identifiers must be unique.".to_owned());
            }
            let name = profile.name.trim();
            if name.is_empty() || name.chars().count() > 48 || name.chars().any(char::is_control) {
                return Err("Character names must contain 1 to 48 printable characters.".to_owned());
            }
            if !names.insert(name.to_lowercase()) {
                return Err("Custom character names must be unique.".to_owned());
            }
            if profile.instructions.trim().is_empty() || profile.instructions.len() > MAX_INSTRUCTIONS_BYTES {
                return Err("Each custom character needs instructions no longer than 16 KiB.".to_owned());
            }
        }
        let selected_is_valid = BUILT_INS.contains(&self.selected_personality.as_str())
            || self.selected_personality.strip_prefix("profile:")
                .is_some_and(|id| ids.contains(id));
        if !selected_is_valid {
            return Err("The selected companion does not exist.".to_owned());
        }
        Ok(())
    }
}

/// Stores only user-authored companion preferences; no chat messages or secrets.
pub struct CompanionSettingsStore {
    path: PathBuf,
    settings: Mutex<CompanionSettings>,
}

impl CompanionSettingsStore {
    pub fn open(path: &Path) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("Could not create companion settings directory: {error}"))?;
        }
        let settings = match std::fs::read_to_string(path) {
            Ok(contents) => {
                let parsed: CompanionSettings = serde_json::from_str(&contents)
                    .map_err(|error| format!("Companion settings could not be read: {error}"))?;
                parsed.validate()?;
                parsed
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => CompanionSettings::default(),
            Err(error) => return Err(format!("Could not read companion settings: {error}")),
        };
        Ok(Self { path: path.to_path_buf(), settings: Mutex::new(settings) })
    }

    pub fn get(&self) -> Result<CompanionSettings, String> {
        self.settings.lock().map(|settings| settings.clone())
            .map_err(|_| "Companion settings are unavailable.".to_owned())
    }

    pub fn save(&self, settings: CompanionSettings) -> Result<CompanionSettings, String> {
        settings.validate()?;
        let mut current = self.settings.lock()
            .map_err(|_| "Companion settings are unavailable.".to_owned())?;
        let contents = serde_json::to_vec_pretty(&settings)
            .map_err(|error| format!("Could not encode companion settings: {error}"))?;
        let temporary = self.path.with_extension("json.tmp");
        std::fs::write(&temporary, contents)
            .map_err(|error| format!("Could not write companion settings: {error}"))?;
        if self.path.exists() {
            std::fs::remove_file(&self.path)
                .map_err(|error| format!("Could not replace companion settings: {error}"))?;
        }
        std::fs::rename(&temporary, &self.path)
            .map_err(|error| format!("Could not finish saving companion settings: {error}"))?;
        *current = settings.clone();
        Ok(settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);

    fn test_path() -> PathBuf {
        std::env::temp_dir().join(format!("midsunday-companions-{}-{}.json",
            std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)))
    }

    #[test]
    fn defaults_are_valid_and_select_best_friend() {
        let settings = CompanionSettings::default();
        assert_eq!(settings.selected_personality, "best-friend");
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn saves_and_reloads_preferences_and_named_profiles() {
        let path = test_path();
        let store = CompanionSettingsStore::open(&path).unwrap();
        let settings = CompanionSettings {
            selected_personality: "profile:study-buddy".to_owned(),
            profiles: vec![CompanionProfile {
                id: "study-buddy".to_owned(),
                name: "Study Buddy".to_owned(),
                instructions: "Be patient and explain concepts with examples.".to_owned(),
            }],
            ..CompanionSettings::default()
        };
        store.save(settings.clone()).unwrap();
        drop(store);
        let reopened = CompanionSettingsStore::open(&path).unwrap();
        assert_eq!(reopened.get().unwrap(), settings);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn rejects_unknown_selection_duplicate_names_and_oversized_instructions() {
        let mut settings = CompanionSettings::default();
        settings.selected_personality = "unknown".to_owned();
        assert!(settings.validate().is_err());
        settings.selected_personality = "best-friend".to_owned();
        settings.profiles = vec![
            CompanionProfile { id: "one".to_owned(), name: "Same".to_owned(), instructions: "Helpful".to_owned() },
            CompanionProfile { id: "two".to_owned(), name: " same ".to_owned(), instructions: "Helpful".to_owned() },
        ];
        assert!(settings.validate().is_err());
        settings.profiles.truncate(1);
        settings.custom_instructions = "x".repeat(MAX_INSTRUCTIONS_BYTES + 1);
        assert!(settings.validate().is_err());
    }

    #[test]
    fn malformed_settings_are_not_silently_overwritten() {
        let path = test_path();
        std::fs::write(&path, "{broken").unwrap();
        assert!(CompanionSettingsStore::open(&path).is_err());
        let _ = std::fs::remove_file(path);
    }
}
