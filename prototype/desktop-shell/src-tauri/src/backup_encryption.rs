use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::{aead::{Aead, KeyInit, Payload}, XChaCha20Poly1305, XNonce};
use rand::{rngs::OsRng, RngCore};
use std::{fs::File, io::{Read, Write}, path::Path};

const MAGIC: &[u8; 8] = b"MSUNENC1";
const SALT_BYTES: usize = 16;
const NONCE_BYTES: usize = 24;
const KEY_BYTES: usize = 32;
const CHUNK_BYTES: usize = 1024 * 1024;
const TAG_BYTES: usize = 16;
const MIN_PASSWORD_BYTES: usize = 12;
const MAX_PASSWORD_BYTES: usize = 1024;
const ARGON_MEMORY_KIB: u32 = 64 * 1024;
const ARGON_ITERATIONS: u32 = 3;

fn validate_password(password: &str) -> Result<(), String> {
    if password.as_bytes().len() < MIN_PASSWORD_BYTES {
        return Err("Use a backup password containing at least 12 bytes.".to_owned());
    }
    if password.as_bytes().len() > MAX_PASSWORD_BYTES {
        return Err("The backup password is too long.".to_owned());
    }
    Ok(())
}

fn derive_key(password: &str, salt: &[u8; SALT_BYTES]) -> Result<[u8; KEY_BYTES], String> {
    validate_password(password)?;
    let params = Params::new(ARGON_MEMORY_KIB, ARGON_ITERATIONS, 1, Some(KEY_BYTES))
        .map_err(|_| "The encrypted backup key settings are unavailable.".to_owned())?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = [0u8; KEY_BYTES];
    argon.hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|_| "Could not derive the encrypted backup key.".to_owned())?;
    Ok(key)
}

fn chunk_nonce(base: &[u8; NONCE_BYTES], index: u64) -> [u8; NONCE_BYTES] {
    let mut nonce = *base;
    for (offset, byte) in index.to_le_bytes().iter().enumerate() {
        nonce[NONCE_BYTES - 8 + offset] ^= byte;
    }
    nonce
}

fn associated_data(index: u64, length: u32, final_chunk: bool) -> Vec<u8> {
    let mut aad = Vec::with_capacity(MAGIC.len() + 8 + 4 + 1);
    aad.extend_from_slice(MAGIC);
    aad.extend_from_slice(&index.to_le_bytes());
    aad.extend_from_slice(&length.to_le_bytes());
    aad.push(u8::from(final_chunk));
    aad
}

fn destination_parent(destination: &Path) -> &Path {
    destination.parent().filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

/// Encrypts a file in independently authenticated 1 MiB chunks.
/// The authenticated final marker makes truncation and reordering detectable.
pub fn encrypt_file(source: &Path, destination: &Path, password: &str) -> Result<(), String> {
    validate_password(password)?;
    let mut input = File::open(source)
        .map_err(|_| "The database snapshot could not be opened for encryption.".to_owned())?;
    let mut output = tempfile::NamedTempFile::new_in(destination_parent(destination))
        .map_err(|_| "The encrypted backup temporary file could not be created.".to_owned())?;

    let result = (|| -> Result<(), String> {
        let mut salt = [0u8; SALT_BYTES];
        let mut base_nonce = [0u8; NONCE_BYTES];
        OsRng.fill_bytes(&mut salt);
        OsRng.fill_bytes(&mut base_nonce);
        let key = derive_key(password, &salt)?;
        let cipher = XChaCha20Poly1305::new((&key).into());

        output.write_all(MAGIC).and_then(|_| output.write_all(&salt)).and_then(|_| output.write_all(&base_nonce))
            .map_err(|_| "The encrypted backup header could not be written.".to_owned())?;

        let mut buffer = vec![0u8; CHUNK_BYTES];
        let mut index = 0u64;
        loop {
            let read = input.read(&mut buffer)
                .map_err(|_| "The database snapshot could not be read during encryption.".to_owned())?;
            if read == 0 { break; }
            let length = u32::try_from(read).map_err(|_| "The database snapshot chunk is too large.".to_owned())?;
            let nonce = chunk_nonce(&base_nonce, index);
            let aad = associated_data(index, length, false);
            let encrypted = cipher.encrypt(XNonce::from_slice(&nonce), Payload { msg: &buffer[..read], aad: &aad })
                .map_err(|_| "The database snapshot could not be encrypted.".to_owned())?;
            output.write_all(&length.to_le_bytes()).and_then(|_| output.write_all(&encrypted))
                .map_err(|_| "The encrypted backup could not be written. Check available disk space.".to_owned())?;
            index = index.checked_add(1).ok_or_else(|| "The encrypted backup is too large.".to_owned())?;
        }

        let nonce = chunk_nonce(&base_nonce, index);
        let aad = associated_data(index, 0, true);
        let final_tag = cipher.encrypt(XNonce::from_slice(&nonce), Payload { msg: &[], aad: &aad })
            .map_err(|_| "The encrypted backup could not be finalized.".to_owned())?;
        output.write_all(&0u32.to_le_bytes()).and_then(|_| output.write_all(&final_tag))
            .and_then(|_| output.flush()).and_then(|_| output.as_file().sync_all())
            .map_err(|_| "The encrypted backup could not be finalized on disk.".to_owned())?;
        Ok(())
    })();

    result?;
    output.persist_noclobber(destination)
        .map_err(|_| "The encrypted backup could not be finalized without replacing an existing file.".to_owned())?;
    Ok(())
}

/// Decrypts only to a new file. Wrong passwords, altered chunks, truncation,
/// reordering, and trailing bytes are rejected; partial plaintext is removed.
pub fn decrypt_file(source: &Path, destination: &Path, password: &str) -> Result<(), String> {
    validate_password(password)?;
    let mut input = File::open(source)
        .map_err(|_| "The encrypted backup could not be opened.".to_owned())?;
    let mut output = tempfile::NamedTempFile::new_in(destination_parent(destination))
        .map_err(|_| "A temporary verification file could not be created.".to_owned())?;

    let result = (|| -> Result<(), String> {
        let mut magic = [0u8; 8];
        let mut salt = [0u8; SALT_BYTES];
        let mut base_nonce = [0u8; NONCE_BYTES];
        input.read_exact(&mut magic).and_then(|_| input.read_exact(&mut salt)).and_then(|_| input.read_exact(&mut base_nonce))
            .map_err(|_| "This is not a supported encrypted midSunday backup.".to_owned())?;
        if &magic != MAGIC {
            return Err("This is not a supported encrypted midSunday backup.".to_owned());
        }
        let key = derive_key(password, &salt)?;
        let cipher = XChaCha20Poly1305::new((&key).into());
        let mut index = 0u64;

        loop {
            let mut length_bytes = [0u8; 4];
            input.read_exact(&mut length_bytes)
                .map_err(|_| "The encrypted backup is incomplete or damaged.".to_owned())?;
            let length = u32::from_le_bytes(length_bytes) as usize;
            if length > CHUNK_BYTES {
                return Err("The encrypted backup contains an invalid chunk size.".to_owned());
            }
            let final_chunk = length == 0;
            let encrypted_length = length.checked_add(TAG_BYTES)
                .ok_or_else(|| "The encrypted backup contains an invalid chunk size.".to_owned())?;
            let mut encrypted = vec![0u8; encrypted_length];
            input.read_exact(&mut encrypted)
                .map_err(|_| "The encrypted backup is incomplete or damaged.".to_owned())?;
            let nonce = chunk_nonce(&base_nonce, index);
            let aad = associated_data(index, length as u32, final_chunk);
            let plaintext = cipher.decrypt(XNonce::from_slice(&nonce), Payload { msg: &encrypted, aad: &aad })
                .map_err(|_| "The password is incorrect or the encrypted backup has been modified.".to_owned())?;

            if final_chunk {
                if !plaintext.is_empty() {
                    return Err("The encrypted backup final marker is invalid.".to_owned());
                }
                let mut trailing = [0u8; 1];
                if input.read(&mut trailing).map_err(|_| "The encrypted backup could not be checked.".to_owned())? != 0 {
                    return Err("The encrypted backup contains unexpected trailing data.".to_owned());
                }
                break;
            }
            if plaintext.len() != length {
                return Err("The encrypted backup contains an invalid data chunk.".to_owned());
            }
            output.write_all(&plaintext)
                .map_err(|_| "The temporary verification copy could not be written. Check available disk space.".to_owned())?;
            index = index.checked_add(1).ok_or_else(|| "The encrypted backup contains too many chunks.".to_owned())?;
        }

        output.flush().and_then(|_| output.as_file().sync_all())
            .map_err(|_| "The temporary verification copy could not be finalized.".to_owned())?;
        Ok(())
    })();

    result?;
    output.persist_noclobber(destination)
        .map_err(|_| "The verified plaintext could not be finalized without replacing an existing file.".to_owned())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf, sync::atomic::{AtomicU64, Ordering}};
    static NEXT: AtomicU64 = AtomicU64::new(1);

    fn paths() -> (PathBuf, PathBuf, PathBuf) {
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("midsunday-encryption-{}-{id}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        (root.join("plain.bin"), root.join("encrypted.msbackup"), root.join("restored.bin"))
    }

    #[test]
    fn encrypted_backup_round_trips_in_authenticated_chunks() {
        let (plain, encrypted, restored) = paths();
        let source = (0..(CHUNK_BYTES + 137)).map(|index| (index % 251) as u8).collect::<Vec<_>>();
        fs::write(&plain, &source).unwrap();
        encrypt_file(&plain, &encrypted, "a-long-test-backup-password").unwrap();
        decrypt_file(&encrypted, &restored, "a-long-test-backup-password").unwrap();
        assert_eq!(fs::read(&restored).unwrap(), source);
        let _ = fs::remove_dir_all(plain.parent().unwrap());
    }

    #[test]
    fn encrypted_backup_rejects_wrong_password_and_modified_ciphertext() {
        let (plain, encrypted, restored) = paths();
        fs::write(&plain, b"private backup payload").unwrap();
        encrypt_file(&plain, &encrypted, "a-long-test-backup-password").unwrap();
        assert!(decrypt_file(&encrypted, &restored, "a-different-backup-password").is_err());
        let mut bytes = fs::read(&encrypted).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 1;
        fs::write(&encrypted, bytes).unwrap();
        assert!(decrypt_file(&encrypted, &restored, "a-long-test-backup-password").is_err());
        let _ = fs::remove_dir_all(plain.parent().unwrap());
    }

    #[test]
    fn encrypted_backup_rejects_truncation_and_short_passwords() {
        let (plain, encrypted, restored) = paths();
        fs::write(&plain, b"private backup payload").unwrap();
        assert!(encrypt_file(&plain, &encrypted, "short").is_err());
        encrypt_file(&plain, &encrypted, "a-long-test-backup-password").unwrap();
        let mut bytes = fs::read(&encrypted).unwrap();
        bytes.truncate(bytes.len() - 4);
        fs::write(&encrypted, bytes).unwrap();
        assert!(decrypt_file(&encrypted, &restored, "a-long-test-backup-password").is_err());
        let _ = fs::remove_dir_all(plain.parent().unwrap());
    }

    #[test]
    fn encrypted_backup_supports_empty_and_exact_chunk_payloads() {
        for (length, suffix) in [(0usize, "empty"), (CHUNK_BYTES, "exact")] {
            let (plain, encrypted, restored) = paths();
            let source = (0..length).map(|index| (index % 239) as u8).collect::<Vec<_>>();
            fs::write(&plain, &source).unwrap();
            encrypt_file(&plain, &encrypted, "a-long-test-backup-password").unwrap();
            decrypt_file(&encrypted, &restored, "a-long-test-backup-password").unwrap();
            assert_eq!(fs::read(&restored).unwrap(), source, "{suffix} payload did not round-trip");
            let _ = fs::remove_dir_all(plain.parent().unwrap());
        }
    }

    #[test]
    fn encrypted_backup_rejects_header_changes_oversized_chunks_and_trailing_bytes() {
        let (plain, encrypted, restored) = paths();
        fs::write(&plain, b"authenticated payload").unwrap();
        encrypt_file(&plain, &encrypted, "a-long-test-backup-password").unwrap();
        let original = fs::read(&encrypted).unwrap();

        let mut changed_header = original.clone();
        changed_header[0] ^= 1;
        fs::write(&encrypted, &changed_header).unwrap();
        assert!(decrypt_file(&encrypted, &restored, "a-long-test-backup-password").is_err());
        assert!(!restored.exists());

        let mut oversized_chunk = original.clone();
        let invalid_length = (CHUNK_BYTES as u32 + 1).to_le_bytes();
        let length_start = MAGIC.len() + SALT_BYTES + NONCE_BYTES;
        oversized_chunk[length_start..length_start + 4].copy_from_slice(&invalid_length);
        fs::write(&encrypted, &oversized_chunk).unwrap();
        assert!(decrypt_file(&encrypted, &restored, "a-long-test-backup-password").is_err());
        assert!(!restored.exists());

        let mut trailing = original;
        trailing.push(0xA5);
        fs::write(&encrypted, trailing).unwrap();
        assert!(decrypt_file(&encrypted, &restored, "a-long-test-backup-password").is_err());
        assert!(!restored.exists());
        let _ = fs::remove_dir_all(plain.parent().unwrap());
    }

    #[test]
    fn encryption_and_decryption_never_overwrite_existing_destinations() {
        let (plain, encrypted, restored) = paths();
        fs::write(&plain, b"private backup payload").unwrap();
        fs::write(&encrypted, b"keep existing encrypted file").unwrap();
        assert!(encrypt_file(&plain, &encrypted, "a-long-test-backup-password").is_err());
        assert_eq!(fs::read(&encrypted).unwrap(), b"keep existing encrypted file");

        fs::write(&encrypted, b"not an encrypted backup").unwrap();
        fs::write(&restored, b"keep existing restored file").unwrap();
        assert!(decrypt_file(&encrypted, &restored, "a-long-test-backup-password").is_err());
        assert_eq!(fs::read(&restored).unwrap(), b"keep existing restored file");
        let _ = fs::remove_dir_all(plain.parent().unwrap());
    }

}