use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub const FILENAME: &str = "SmolLM2-135M-Instruct-Q4_K_M.gguf";
pub const SHA256_HEX: &str = "2e8040ceae7815abe0dcb3540b9995eaa1fa0d2ca9e797d0a635ae4433c68c2d";
pub const HF_BASE_REPO: &str = "HuggingFaceTB/SmolLM2-135M-Instruct";
pub const HF_GGUF_REPO: &str = "bartowski/SmolLM2-135M-Instruct-GGUF";
pub const HF_REVISION: &str = "09816acd5d99df7be770d85ea30822623dab342c";

static OVERRIDE: OnceLock<PathBuf> = OnceLock::new();

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WeightsError {
    Missing,
    HashMismatch,
    Unreadable,
}

pub fn set_weights_path(path: PathBuf) {
    let _ = OVERRIDE.set(path);
}

pub fn vendor_weights_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("vendor")
        .join(FILENAME)
}

pub fn candidate_paths() -> Vec<PathBuf> {
    if let Some(path) = OVERRIDE.get() {
        return vec![path.clone()];
    }
    if let Ok(path) = std::env::var("BRONZE_TITLE_WEIGHTS") {
        if !path.is_empty() {
            return vec![PathBuf::from(path)];
        }
    }
    let mut out = vec![vendor_weights_path()];
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            out.push(dir.join("models").join(FILENAME));
            out.push(dir.join("../Resources/models").join(FILENAME));
        }
    }
    out
}

pub fn verified_weights_path() -> Result<PathBuf, WeightsError> {
    let mut saw_mismatch = false;
    let mut saw_unreadable = false;
    for path in candidate_paths() {
        match verify_weights(&path) {
            Ok(()) => return Ok(path),
            Err(WeightsError::Missing) => {}
            Err(WeightsError::HashMismatch) => saw_mismatch = true,
            Err(WeightsError::Unreadable) => saw_unreadable = true,
        }
    }
    if saw_mismatch {
        Err(WeightsError::HashMismatch)
    } else if saw_unreadable {
        Err(WeightsError::Unreadable)
    } else {
        Err(WeightsError::Missing)
    }
}

pub fn verify_weights(path: &Path) -> Result<(), WeightsError> {
    if !path.is_file() {
        return Err(WeightsError::Missing);
    }
    let mut file = File::open(path).map_err(|_| WeightsError::Unreadable)?;
    let mut hasher = Sha256::new();
    let mut buf = [0_u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf).map_err(|_| WeightsError::Unreadable)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let actual = hex_lower(&hasher.finalize());
    if actual == SHA256_HEX {
        Ok(())
    } else {
        Err(WeightsError::HashMismatch)
    }
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod weights_tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn missing_file_is_missing() {
        let path = std::env::temp_dir().join("bronze-missing-title-weights.gguf");
        let _ = std::fs::remove_file(&path);
        assert_eq!(verify_weights(&path), Err(WeightsError::Missing));
    }

    #[test]
    fn bad_hash_is_mismatch() {
        let path = std::env::temp_dir().join("bronze-bad-title-weights.gguf");
        let mut file = File::create(&path).expect("create");
        file.write_all(b"not-a-gguf").expect("write");
        drop(file);
        assert_eq!(verify_weights(&path), Err(WeightsError::HashMismatch));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn manifest_pins_expected_file() {
        let manifest = include_str!("../vendor/MANIFEST");
        assert!(manifest.contains(FILENAME));
        assert!(manifest.contains(SHA256_HEX));
        assert!(manifest.contains(HF_BASE_REPO));
        assert!(manifest.contains(HF_GGUF_REPO));
        assert!(manifest.contains(HF_REVISION));
        assert!(manifest.contains("Apache-2.0"));
    }
}
