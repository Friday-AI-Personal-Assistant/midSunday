//! Read-only local file checks used by the first-launch setup guide.
use serde::Serialize;
use std::{fs, fs::File, io::Read, path::Path};

#[derive(Clone, Debug, Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SetupFileCheck {
    pub status: String,
    pub detail: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocalSetupReadiness {
    pub executable: SetupFileCheck,
    pub model: SetupFileCheck,
}

pub fn check_local_setup(executable_path: &str, model_path: &str) -> LocalSetupReadiness {
    LocalSetupReadiness {
        executable: check_executable(executable_path),
        model: check_gguf_model(model_path),
    }
}

fn check_executable(value: &str) -> SetupFileCheck {
    let value = value.trim();
    if value.is_empty() { return result("not_configured", "Choose an existing llama.cpp server executable."); }
    if value.len() > 4096 || value.contains('\0') { return result("invalid_path", "The server path is invalid or too long."); }
    let canonical = match Path::new(value).canonicalize() {
        Ok(path) => path,
        Err(_) => return result("missing", "The selected server file could not be found. Check its path."),
    };
    let metadata = match fs::metadata(&canonical) {
        Ok(metadata) => metadata,
        Err(_) => return result("unavailable", "The selected server file could not be inspected."),
    };
    if !metadata.is_file() { return result("not_a_file", "The selected server path is not a regular file."); }
    #[cfg(windows)]
    if !canonical.extension().and_then(|extension| extension.to_str()).is_some_and(|extension| extension.eq_ignore_ascii_case("exe")) {
        return result("wrong_file_type", "On Windows, select the llama-server.exe application file.");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o111 == 0 {
            return result("not_executable", "The file has no executable permission. Check its permissions before continuing.");
        }
    }
    result("ready", "An existing server file was found and its platform file checks passed. The binary has not been launched or compatibility-tested.")
}

fn check_gguf_model(value: &str) -> SetupFileCheck {
    let value = value.trim();
    if value.is_empty() { return result("not_configured", "Choose an existing GGUF model file."); }
    if value.len() > 4096 || value.contains('\0') { return result("invalid_path", "The model path is invalid or too long."); }
    let canonical = match Path::new(value).canonicalize() {
        Ok(path) => path,
        Err(_) => return result("missing", "The selected model file could not be found. Check its path."),
    };
    if !canonical.is_file() { return result("not_a_file", "The selected model path is not a regular file."); }
    if !canonical.extension().and_then(|extension| extension.to_str()).is_some_and(|extension| extension.eq_ignore_ascii_case("gguf")) {
        return result("wrong_file_type", "The selected model file must use the .gguf extension.");
    }
    let metadata = match fs::metadata(&canonical) {
        Ok(metadata) => metadata,
        Err(_) => return result("unavailable", "The selected model file could not be inspected."),
    };
    if metadata.len() <= 4 { return result("empty", "The GGUF file does not contain model data."); }
    let mut file = match File::open(&canonical) {
        Ok(file) => file,
        Err(_) => return result("unavailable", "The GGUF file could not be opened for a header check."),
    };
    let mut magic = [0_u8; 4];
    if file.read_exact(&mut magic).is_err() || &magic != b"GGUF" {
        return result("invalid_gguf", "The file does not contain a valid GGUF header.");
    }
    result("ready", "An existing GGUF file with a valid header was found. Full catalogue fingerprint verification still occurs before engine startup.")
}

fn result(status: &str, detail: &str) -> SetupFileCheck {
    SetupFileCheck { status: status.to_owned(), detail: detail.to_owned() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);

    fn temp_file(name: &str, contents: &[u8]) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("midsunday-setup-{}-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed), name));
        fs::write(&path, contents).unwrap();
        path
    }

    #[test]
    fn empty_paths_are_not_configured() {
        let result = check_local_setup("", "");
        assert_eq!(result.executable.status, "not_configured");
        assert_eq!(result.model.status, "not_configured");
    }

    #[test]
    fn model_check_requires_gguf_extension_and_magic() {
        let valid = temp_file("valid.gguf", b"GGUFmodel-data");
        assert_eq!(check_gguf_model(valid.to_str().unwrap()).status, "ready");
        let invalid = temp_file("invalid.gguf", b"NOPEmodel-data");
        assert_eq!(check_gguf_model(invalid.to_str().unwrap()).status, "invalid_gguf");
        let wrong_extension = temp_file("wrong.bin", b"GGUFmodel-data");
        assert_eq!(check_gguf_model(wrong_extension.to_str().unwrap()).status, "wrong_file_type");
        for path in [valid, invalid, wrong_extension] { let _ = fs::remove_file(path); }
    }

    #[test]
    fn missing_paths_are_reported_without_panicking() {
        let path = std::env::temp_dir().join("midsunday-file-that-does-not-exist.gguf");
        assert_eq!(check_gguf_model(path.to_str().unwrap()).status, "missing");
        assert_eq!(check_executable(path.to_str().unwrap()).status, "missing");
    }

    #[test]
    fn executable_check_never_launches_the_selected_file() {
        #[cfg(windows)]
        let path = temp_file("server.exe", b"not-a-real-executable");
        #[cfg(not(windows))]
        let path = temp_file("server", b"not-a-real-executable");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&path, permissions).unwrap();
        }
        let checked = check_executable(path.to_str().unwrap());
        assert_eq!(checked.status, "ready");
        assert!(checked.detail.contains("has not been launched"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn directories_are_not_accepted_as_executable_or_model_files() {
        let directory = std::env::temp_dir().join(format!(
            "midsunday-setup-directory-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&directory).unwrap();
        let checked = check_local_setup(directory.to_str().unwrap(), directory.to_str().unwrap());
        assert_eq!(checked.executable.status, "not_a_file");
        assert_eq!(checked.model.status, "not_a_file");
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn invalid_and_oversized_paths_are_rejected_before_filesystem_access() {
        let oversized = "x".repeat(4097);
        for invalid in ["\0", oversized.as_str()] {
            let checked = check_local_setup(invalid, invalid);
            assert_eq!(checked.executable.status, "invalid_path");
            assert_eq!(checked.model.status, "invalid_path");
        }
    }

    #[cfg(unix)]
    #[test]
    fn executable_without_execute_permission_is_not_ready() {
        let path = temp_file("not-executable", b"not-a-real-executable");
        assert_eq!(check_executable(path.to_str().unwrap()).status, "not_executable");
        let _ = fs::remove_file(path);
    }
}
