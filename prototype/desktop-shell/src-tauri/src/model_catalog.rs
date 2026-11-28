//! User-managed inventory of existing local GGUF model files.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fmt::Write as _, fs::{self, File}, io::{BufReader, Read, Write}, path::{Path, PathBuf}, sync::{Arc, Mutex}, time::{SystemTime, UNIX_EPOCH}};
const MAX_LABEL_BYTES: usize = 512;
const HASH_BUFFER_BYTES: usize = 1024 * 1024;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocalModel {
    pub id: String, pub display_name: String, pub file_path: String,
    pub size_bytes: u64, pub sha256: String, pub source: String,
    pub license: String, pub added_at: u64,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ModelVerification {
    pub valid: bool,
    pub status: String,
    pub detail: String,
}
#[derive(Clone)]
pub struct ModelCatalog { path: PathBuf, models: Arc<Mutex<Vec<LocalModel>>> }

impl ModelCatalog {
    pub fn open(path: &Path) -> Result<Self, String> {
        if let Some(parent) = path.parent() { fs::create_dir_all(parent).map_err(|e| format!("Could not create model catalogue directory: {e}"))?; }
        let models = match fs::read_to_string(path) {
            Ok(contents) => { let parsed: Vec<LocalModel> = serde_json::from_str(&contents).map_err(|e| format!("Local model catalogue could not be read: {e}"))?; validate_catalogue(&parsed)?; parsed }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(e) => return Err(format!("Could not read local model catalogue: {e}")),
        };
        Ok(Self { path: path.to_path_buf(), models: Arc::new(Mutex::new(models)) })
    }
    pub fn list(&self) -> Result<Vec<LocalModel>, String> {
        self.models.lock().map(|m| m.clone()).map_err(|_| "Local model catalogue is unavailable.".to_owned())
    }
    pub fn register(&self, file_path: &str, display_name: &str, source: &str, license: &str) -> Result<LocalModel, String> {
        let display_name = clean_label(display_name, "Model name", 120)?;
        let source = clean_label(source, "Source", MAX_LABEL_BYTES)?;
        let license = clean_label(license, "Licence", MAX_LABEL_BYTES)?;
        let canonical = Path::new(file_path).canonicalize().map_err(|e| format!("Could not locate the selected model file: {e}"))?;
        if !canonical.is_file() { return Err("The selected model path is not a file.".to_owned()); }
        if !canonical.extension().and_then(|v| v.to_str()).is_some_and(|v| v.eq_ignore_ascii_case("gguf")) { return Err("Only existing .gguf model files can be registered.".to_owned()); }
        let metadata = fs::metadata(&canonical).map_err(|e| format!("Could not inspect the selected model file: {e}"))?;
        validate_gguf_header(&canonical)?;
        if metadata.len() <= 4 { return Err("The selected GGUF file contains no model data.".to_owned()); }
        let sha256 = hash_file(&canonical)?;
        let added_at = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_| "System clock is earlier than the Unix epoch.".to_owned())?.as_secs();
        let model = LocalModel { id: sha256.clone(), display_name, file_path: canonical.to_string_lossy().into_owned(), size_bytes: metadata.len(), sha256, source, license, added_at };
        let mut models = self.models.lock().map_err(|_| "Local model catalogue is unavailable.".to_owned())?;
        let mut next = models.clone();
        if let Some(existing) = next.iter_mut().find(|entry| entry.id == model.id) { *existing = model.clone(); } else { next.push(model.clone()); }
        self.persist(&next)?;
        *models = next;
        Ok(model)
    }
    pub fn verify(&self, id: &str) -> Result<ModelVerification, String> {
        let model = self.models.lock().map_err(|_| "Local model catalogue is unavailable.".to_owned())?
            .iter().find(|model| model.id == id).cloned()
            .ok_or_else(|| "The selected model is no longer in the local library.".to_owned())?;
        Ok(verify_record(&model))
    }

    /// Validates a selected GGUF before launch. Registered files must still match
    /// their recorded size and SHA-256 fingerprint; unregistered files must have
    /// a valid GGUF header but are not assigned catalogue metadata.
    pub fn verify_path(&self, file_path: &str) -> Result<ModelVerification, String> {
        let canonical = Path::new(file_path).canonicalize()
            .map_err(|_| "The selected GGUF model file could not be found. Check its path.".to_owned())?;
        validate_gguf_header(&canonical)?;
        let models = self.models.lock().map_err(|_| "Local model catalogue is unavailable.".to_owned())?;
        if let Some(model) = models.iter().find(|model| Path::new(&model.file_path) == canonical) {
            return Ok(verify_record(model));
        }
        Ok(ModelVerification {
            valid: true,
            status: "unregistered".to_owned(),
            detail: "GGUF header is valid. This file is not registered, so no saved fingerprint can be checked.".to_owned(),
        })
    }

    /// Removes catalogue metadata only; the original GGUF file is never deleted.
    pub fn remove(&self, id: &str) -> Result<bool, String> {
        if id.len() != 64 || !id.bytes().all(|b| b.is_ascii_hexdigit()) { return Err("The model identifier is invalid.".to_owned()); }
        let mut models = self.models.lock().map_err(|_| "Local model catalogue is unavailable.".to_owned())?;
        let mut next = models.clone();
        let before = next.len(); next.retain(|model| model.id != id);
        if next.len() == before { return Ok(false); }
        self.persist(&next)?; *models = next; Ok(true)
    }
    fn persist(&self, models: &[LocalModel]) -> Result<(), String> {
        let contents = serde_json::to_vec_pretty(models).map_err(|e| format!("Could not encode local model catalogue: {e}"))?;
        let parent = self.path.parent().filter(|parent| !parent.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
        let mut temporary = tempfile::NamedTempFile::new_in(parent)
            .map_err(|e| format!("Could not prepare a temporary model catalogue: {e}"))?;
        temporary.write_all(&contents)
            .and_then(|_| temporary.as_file().sync_all())
            .map_err(|e| format!("Could not write local model catalogue: {e}"))?;
        temporary.persist(&self.path)
            .map_err(|e| format!("Could not atomically replace local model catalogue: {}", e.error))?;
        Ok(())
    }
}
fn validate_gguf_header(path: &Path) -> Result<(), String> {
    let mut file = File::open(path).map_err(|e| format!("Could not open the GGUF model file: {e}"))?;
    let mut magic = [0_u8; 4];
    file.read_exact(&mut magic).map_err(|_| "The selected file is too small to be a valid GGUF model.".to_owned())?;
    if &magic != b"GGUF" {
        return Err("The selected file does not have a valid GGUF header.".to_owned());
    }
    Ok(())
}

fn verify_record(model: &LocalModel) -> ModelVerification {
    let fail = |status: &str, detail: String| ModelVerification { valid: false, status: status.to_owned(), detail };
    let path = Path::new(&model.file_path);
    let canonical = match path.canonicalize() {
        Ok(path) => path,
        Err(_) => return fail("missing", "The original model file could not be found. It may have been moved or deleted.".to_owned()),
    };
    let metadata = match fs::metadata(&canonical) {
        Ok(metadata) => metadata,
        Err(error) => return fail("unavailable", format!("Could not inspect the model file: {error}")),
    };
    if metadata.len() != model.size_bytes {
        return fail("sizeMismatch", format!("File size changed: expected {} bytes, found {} bytes.", model.size_bytes, metadata.len()));
    }
    if let Err(error) = validate_gguf_header(&canonical) {
        return fail("invalidFormat", error);
    }
    match hash_file(&canonical) {
        Ok(hash) if hash == model.sha256 => ModelVerification {
            valid: true, status: "verified".to_owned(),
            detail: "The GGUF header, file size, and SHA-256 fingerprint match the saved catalogue record.".to_owned(),
        },
        Ok(_) => fail("hashMismatch", "The file contents no longer match the saved SHA-256 fingerprint.".to_owned()),
        Err(error) => fail("unavailable", error),
    }
}

fn clean_label(value: &str, label: &str, max: usize) -> Result<String, String> {
    let value = value.trim();
    if value.len() > max || value.contains('\0') { return Err(format!("{label} is too long or contains invalid characters.")); }
    Ok(value.to_owned())
}
fn hash_file(path: &Path) -> Result<String, String> {
    let file = File::open(path).map_err(|e| format!("Could not open model file for fingerprinting: {e}"))?;
    let mut reader = BufReader::with_capacity(HASH_BUFFER_BYTES, file);
    let mut hasher = Sha256::new(); let mut buffer = vec![0; HASH_BUFFER_BYTES];
    loop { let count = reader.read(&mut buffer).map_err(|e| format!("Could not read model file for fingerprinting: {e}"))?; if count == 0 { break; } hasher.update(&buffer[..count]); }
    let mut output = String::with_capacity(64);
    for byte in hasher.finalize() { write!(&mut output, "{byte:02x}").expect("String formatting cannot fail"); }
    Ok(output)
}
fn validate_catalogue(models: &[LocalModel]) -> Result<(), String> {
    if models.len() > 10_000 { return Err("The local model catalogue contains too many entries.".to_owned()); }
    let mut ids = std::collections::HashSet::new();
    for model in models {
        if model.id.len() != 64 || model.id != model.sha256 || !model.id.bytes().all(|b| b.is_ascii_hexdigit()) || !ids.insert(model.id.as_str()) { return Err("The local model catalogue contains an invalid or duplicate fingerprint.".to_owned()); }
        clean_label(&model.display_name, "Model name", 120)?; clean_label(&model.source, "Source", MAX_LABEL_BYTES)?; clean_label(&model.license, "Licence", MAX_LABEL_BYTES)?;
        if model.file_path.is_empty() || model.file_path.len() > 4096 || model.file_path.contains('\0') || model.size_bytes == 0 { return Err("The local model catalogue contains invalid file metadata.".to_owned()); }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    fn dir() -> PathBuf { let p = std::env::temp_dir().join(format!("midsunday-model-catalog-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed))); fs::create_dir_all(&p).unwrap(); p }
    #[test] fn registers_fingerprints_and_reloads_catalogue() {
        let d=dir(); let model_path=d.join("tiny.gguf"); fs::write(&model_path,b"GGUF-local test model bytes").unwrap();
        let path=d.join("models.json"); let c=ModelCatalog::open(&path).unwrap();
        let model=c.register(model_path.to_str().unwrap(),"Tiny model","Local test","Test-only").unwrap();
        assert_eq!(model.sha256.len(),64); assert_eq!(model.id,model.sha256); assert_eq!(model.size_bytes,27);
        drop(c); let reopened=ModelCatalog::open(&path).unwrap(); assert_eq!(reopened.list().unwrap(),vec![model]); let _=fs::remove_dir_all(d);
    }
    #[test] fn duplicate_content_is_one_catalogue_entry() {
        let d=dir(); let a=d.join("a.gguf"); let b=d.join("b.gguf"); fs::write(&a,b"GGUF-same model").unwrap(); fs::write(&b,b"GGUF-same model").unwrap();
        let c=ModelCatalog::open(&d.join("models.json")).unwrap(); c.register(a.to_str().unwrap(),"First","","").unwrap(); c.register(b.to_str().unwrap(),"Second","","").unwrap();
        assert_eq!(c.list().unwrap().len(),1); assert_eq!(c.list().unwrap()[0].display_name,"Second"); let _=fs::remove_dir_all(d);
    }
    #[test] fn rejects_wrong_extension_and_empty_files() {
        let d=dir(); let wrong=d.join("model.bin"); let empty=d.join("empty.gguf"); fs::write(&wrong,b"NOPE-not gguf").unwrap(); fs::write(&empty,b"").unwrap();
        let c=ModelCatalog::open(&d.join("models.json")).unwrap(); assert!(c.register(wrong.to_str().unwrap(),"Wrong","","").is_err()); assert!(c.register(empty.to_str().unwrap(),"Empty","","").is_err()); let _=fs::remove_dir_all(d);
    }
    #[test] fn removal_never_deletes_the_original_model_file() {
        let d=dir(); let p=d.join("keep.gguf"); fs::write(&p,b"GGUF-keep me").unwrap(); let c=ModelCatalog::open(&d.join("models.json")).unwrap();
        let m=c.register(p.to_str().unwrap(),"Keep","","").unwrap(); assert!(c.remove(&m.id).unwrap()); assert!(p.exists()); assert!(c.list().unwrap().is_empty()); let _=fs::remove_dir_all(d);
    }
    #[test] fn registration_rejects_files_without_gguf_magic() {
        let d=dir(); let p=d.join("fake.gguf"); fs::write(&p,b"not a model").unwrap();
        let c=ModelCatalog::open(&d.join("models.json")).unwrap();
        assert!(c.register(p.to_str().unwrap(),"Fake","","").is_err());
        let _=fs::remove_dir_all(d);
    }
    #[test] fn verification_detects_modified_and_missing_files() {
        let d=dir(); let p=d.join("valid.gguf"); fs::write(&p,b"GGUF-original model").unwrap();
        let c=ModelCatalog::open(&d.join("models.json")).unwrap();
        let model=c.register(p.to_str().unwrap(),"Valid","","").unwrap();
        assert!(c.verify(&model.id).unwrap().valid);
        fs::write(&p,b"GGUF-replaced model").unwrap();
        let changed=c.verify(&model.id).unwrap();
        assert!(!changed.valid);
        assert!(matches!(changed.status.as_str(),"sizeMismatch"|"hashMismatch"));
        fs::remove_file(&p).unwrap();
        assert_eq!(c.verify(&model.id).unwrap().status,"missing");
        let _=fs::remove_dir_all(d);
    }
    #[test] fn cloned_catalogue_handles_share_the_same_local_inventory() {
        let d=dir(); let p=d.join("shared.gguf"); fs::write(&p,b"GGUF-shared model bytes").unwrap();
        let c=ModelCatalog::open(&d.join("models.json")).unwrap();
        let worker_handle=c.clone();
        let model=c.register(p.to_str().unwrap(),"Shared","","").unwrap();
        assert_eq!(worker_handle.list().unwrap(),vec![model]);
        let _=fs::remove_dir_all(d);
    }
    #[test] fn unregistered_gguf_can_be_preflighted_without_catalogue_hash() {
        let d=dir(); let p=d.join("unregistered.gguf"); fs::write(&p,b"GGUF-unregistered").unwrap();
        let c=ModelCatalog::open(&d.join("models.json")).unwrap();
        let result=c.verify_path(p.to_str().unwrap()).unwrap();
        assert!(result.valid); assert_eq!(result.status,"unregistered");
        let _=fs::remove_dir_all(d);
    }
    #[test] fn successive_catalogue_updates_atomically_replace_and_reload_the_file() {
        let d=dir();
        let first_path=d.join("first.gguf");
        let second_path=d.join("second.gguf");
        fs::write(&first_path,b"GGUF-first model").unwrap();
        fs::write(&second_path,b"GGUF-second model").unwrap();
        let catalogue_path=d.join("models.json");
        let catalogue=ModelCatalog::open(&catalogue_path).unwrap();
        catalogue.register(first_path.to_str().unwrap(),"First","","").unwrap();
        catalogue.register(second_path.to_str().unwrap(),"Second","","").unwrap();
        let expected=catalogue.list().unwrap();
        assert_eq!(expected.len(),2);
        drop(catalogue);
        let reopened=ModelCatalog::open(&catalogue_path).unwrap();
        assert_eq!(reopened.list().unwrap(),expected);
        let _=fs::remove_dir_all(d);
    }
    #[test] fn malformed_catalogue_is_not_silently_overwritten() {
        let d=dir(); let p=d.join("models.json"); fs::write(&p,"{broken").unwrap(); assert!(ModelCatalog::open(&p).is_err()); let _=fs::remove_dir_all(d);
    }
}
