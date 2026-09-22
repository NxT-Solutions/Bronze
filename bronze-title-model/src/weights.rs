use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub const FILENAME: &str = "SmolLM2-360M-Instruct-Q4_K_M.gguf";
pub const SHA256_HEX: &str = "2fa3f013dcdd7b99f9b237717fa0b12d75bbb89984cc1274be1471a465bac9c2";
pub const HF_BASE_REPO: &str = "HuggingFaceTB/SmolLM2-360M-Instruct";
pub const HF_GGUF_REPO: &str = "bartowski/SmolLM2-360M-Instruct-GGUF";
pub const HF_REVISION: &str = "ab928a97ee49f3a015f35194879f68211291d6ca";

static OVERRIDE: OnceLock<PathBuf> = OnceLock::new();
static VERIFIED: OnceLock<(PathBuf, WeightsSource)> = OnceLock::new();

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WeightsError {
    Missing,
    HashMismatch,
    Unreadable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WeightsSource {
    Override,
    Env,
    Vendor,
    Workspace,
    Exe,
    Resources,
}

impl WeightsSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Override => "override",
            Self::Env => "env",
            Self::Vendor => "vendor",
            Self::Workspace => "workspace",
            Self::Exe => "exe",
            Self::Resources => "resources",
        }
    }
}

pub fn set_weights_path(path: PathBuf) {
    let _ = OVERRIDE.set(path);
}

pub fn vendor_weights_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("vendor")
        .join(FILENAME)
}

pub fn any_candidate_file() -> bool {
    candidate_paths()
        .into_iter()
        .any(|(path, _)| path.is_file())
}

pub fn weights_present() -> Result<(), WeightsError> {
    if VERIFIED.get().is_some() || any_candidate_file() {
        Ok(())
    } else {
        Err(WeightsError::Missing)
    }
}

pub fn candidate_paths() -> Vec<(PathBuf, WeightsSource)> {
    let mut out = Vec::new();
    push_unique(&mut out, OVERRIDE.get().cloned(), WeightsSource::Override);
    if let Ok(path) = std::env::var("BRONZE_TITLE_WEIGHTS") {
        if !path.is_empty() {
            push_unique(&mut out, Some(PathBuf::from(path)), WeightsSource::Env);
        }
    }
    push_unique(&mut out, Some(vendor_weights_path()), WeightsSource::Vendor);
    push_unique(
        &mut out,
        walk_vendor_from_manifest(),
        WeightsSource::Workspace,
    );
    if let Ok(cwd) = std::env::current_dir() {
        push_unique(&mut out, walk_vendor(cwd), WeightsSource::Workspace);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            push_unique(
                &mut out,
                Some(dir.join("models").join(FILENAME)),
                WeightsSource::Exe,
            );
            push_unique(
                &mut out,
                Some(dir.join("../Resources/models").join(FILENAME)),
                WeightsSource::Resources,
            );
            push_unique(
                &mut out,
                walk_vendor(dir.to_path_buf()),
                WeightsSource::Workspace,
            );
        }
    }
    out
}

pub fn verified_weights_path() -> Result<PathBuf, WeightsError> {
    verified_weights().map(|(path, _)| path)
}

pub fn verified_weights() -> Result<(PathBuf, WeightsSource), WeightsError> {
    if let Some(found) = VERIFIED.get() {
        return Ok(found.clone());
    }
    let mut saw_mismatch = false;
    let mut saw_unreadable = false;
    for (path, source) in candidate_paths() {
        match verify_weights(&path) {
            Ok(()) => {
                if VERIFIED.set((path.clone(), source)).is_ok() {
                    crate::emit_diag(&format!("weights resolved source={}", source.as_str()));
                    crate::emit_diag("hash ok");
                }
                return Ok((path, source));
            }
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

fn walk_vendor_from_manifest() -> Option<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|root| walk_vendor(root.to_path_buf()))
}

fn walk_vendor(mut dir: PathBuf) -> Option<PathBuf> {
    for _ in 0..10 {
        if dir
            .file_name()
            .is_some_and(|name| name == "bronze-title-model")
        {
            let direct = dir.join("vendor").join(FILENAME);
            if direct.is_file() {
                return Some(direct);
            }
        }
        let nested = dir.join("bronze-title-model").join("vendor").join(FILENAME);
        if nested.is_file() {
            return Some(nested);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

fn push_unique(
    out: &mut Vec<(PathBuf, WeightsSource)>,
    path: Option<PathBuf>,
    source: WeightsSource,
) {
    let Some(path) = path else {
        return;
    };
    if out.iter().any(|(existing, _)| existing == &path) {
        return;
    }
    out.push((path, source));
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
    fn vendor_path_is_title_model_crate() {
        let path = vendor_weights_path();
        let text = path.to_string_lossy();
        assert!(text.contains("bronze-title-model"));
        assert!(text.ends_with(FILENAME));
        assert!(!text.contains("src-tauri/vendor"));
        let sources = candidate_paths();
        assert!(sources
            .iter()
            .any(|(candidate, source)| *source == WeightsSource::Vendor && candidate == &path));
    }

    #[test]
    fn workspace_walk_finds_crate_vendor_when_present() {
        let from_manifest = walk_vendor_from_manifest();
        let vendor = vendor_weights_path();
        if vendor.is_file() {
            assert_eq!(from_manifest.as_ref(), Some(&vendor));
            assert_eq!(verify_weights(&vendor), Ok(()));
        } else {
            assert_eq!(verify_weights(&vendor), Err(WeightsError::Missing));
        }
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
        assert!(FILENAME.contains("360M"));
        assert!(!FILENAME.contains("135M"));
    }
}
