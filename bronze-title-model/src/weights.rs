use crate::tiers::{spec_for_filename, TierSpec, TitleTier, GGUF_TIERS};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

pub const FILENAME: &str = "SmolLM2-360M-Instruct-Q4_K_M.gguf";
pub const SHA256_HEX: &str = "2fa3f013dcdd7b99f9b237717fa0b12d75bbb89984cc1274be1471a465bac9c2";
pub const HF_BASE_REPO: &str = "HuggingFaceTB/SmolLM2-360M-Instruct";
pub const HF_GGUF_REPO: &str = "bartowski/SmolLM2-360M-Instruct-GGUF";
pub const HF_REVISION: &str = "ab928a97ee49f3a015f35194879f68211291d6ca";

static OVERRIDE: OnceLock<PathBuf> = OnceLock::new();
static WEIGHTS_DIR: OnceLock<PathBuf> = OnceLock::new();
static HASH_CACHE: Mutex<Option<HashMap<PathBuf, Result<(), WeightsError>>>> = Mutex::new(None);

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
    BundleDir,
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
            Self::BundleDir => "bundle",
        }
    }
}

pub fn set_weights_path(path: PathBuf) {
    let _ = OVERRIDE.set(path);
}

pub fn set_weights_dir(dir: PathBuf) {
    let _ = WEIGHTS_DIR.set(dir);
}

pub fn vendor_weights_path() -> PathBuf {
    vendor_path_for(FILENAME)
}

pub fn vendor_path_for(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("vendor")
        .join(filename)
}

pub fn any_candidate_file() -> bool {
    !present_gguf_tiers().is_empty()
}

pub fn present_gguf_tiers() -> Vec<TitleTier> {
    GGUF_TIERS
        .iter()
        .filter(|spec| {
            candidate_paths_for(spec)
                .into_iter()
                .any(|(path, _)| path.is_file())
        })
        .map(|spec| spec.tier)
        .collect()
}

pub fn weights_present() -> Result<(), WeightsError> {
    weights_present_for(TitleTier::Smol360)
}

pub fn weights_present_for(tier: TitleTier) -> Result<(), WeightsError> {
    if tier == TitleTier::Extractive {
        return Ok(());
    }
    let spec = tier.spec().ok_or(WeightsError::Missing)?;
    if candidate_paths_for(spec)
        .into_iter()
        .any(|(path, _)| path.is_file())
    {
        Ok(())
    } else {
        Err(WeightsError::Missing)
    }
}

pub fn candidate_paths() -> Vec<(PathBuf, WeightsSource)> {
    TitleTier::Smol360
        .spec()
        .map(candidate_paths_for)
        .unwrap_or_default()
}

pub fn candidate_paths_for(spec: &TierSpec) -> Vec<(PathBuf, WeightsSource)> {
    let mut out = Vec::new();
    if let Some(path) = OVERRIDE.get() {
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == spec.filename)
        {
            push_unique(&mut out, Some(path.clone()), WeightsSource::Override);
        }
    }
    if let Ok(path) = std::env::var("BRONZE_TITLE_WEIGHTS") {
        if !path.is_empty() {
            let path = PathBuf::from(path);
            if path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name == spec.filename)
            {
                push_unique(&mut out, Some(path), WeightsSource::Env);
            }
        }
    }
    if let Some(dir) = WEIGHTS_DIR.get() {
        push_unique(
            &mut out,
            Some(dir.join(spec.filename)),
            WeightsSource::BundleDir,
        );
    }
    push_unique(
        &mut out,
        Some(vendor_path_for(spec.filename)),
        WeightsSource::Vendor,
    );
    push_unique(
        &mut out,
        walk_vendor_from_manifest(spec.filename),
        WeightsSource::Workspace,
    );
    if let Ok(cwd) = std::env::current_dir() {
        push_unique(
            &mut out,
            walk_vendor(cwd, spec.filename),
            WeightsSource::Workspace,
        );
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            push_unique(
                &mut out,
                Some(dir.join("models").join(spec.filename)),
                WeightsSource::Exe,
            );
            push_unique(
                &mut out,
                Some(dir.join("../Resources/models").join(spec.filename)),
                WeightsSource::Resources,
            );
            push_unique(
                &mut out,
                walk_vendor(dir.to_path_buf(), spec.filename),
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
    verified_weights_for(TitleTier::Smol360)
}

pub fn verified_weights_for(tier: TitleTier) -> Result<(PathBuf, WeightsSource), WeightsError> {
    let spec = tier.spec().ok_or(WeightsError::Missing)?;
    let mut saw_mismatch = false;
    let mut saw_unreadable = false;
    for (path, source) in candidate_paths_for(spec) {
        match cached_verify(&path, spec.sha256_hex) {
            Ok(()) => {
                crate::emit_diag(&format!("weights resolved source={}", source.as_str()));
                crate::emit_diag("hash ok");
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
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    let sha = spec_for_filename(name)
        .map(|spec| spec.sha256_hex)
        .unwrap_or(SHA256_HEX);
    cached_verify(path, sha)
}

fn cached_verify(path: &Path, sha256_hex: &str) -> Result<(), WeightsError> {
    {
        let cache = HASH_CACHE
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(map) = cache.as_ref() {
            if let Some(hit) = map.get(path) {
                return *hit;
            }
        }
    }
    let result = hash_file(path, sha256_hex);
    let mut cache = HASH_CACHE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    cache
        .get_or_insert_with(HashMap::new)
        .insert(path.to_path_buf(), result);
    result
}

fn hash_file(path: &Path, sha256_hex: &str) -> Result<(), WeightsError> {
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
    if actual == sha256_hex {
        Ok(())
    } else {
        Err(WeightsError::HashMismatch)
    }
}

fn walk_vendor_from_manifest(filename: &str) -> Option<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|root| walk_vendor(root.to_path_buf(), filename))
}

fn walk_vendor(mut dir: PathBuf, filename: &str) -> Option<PathBuf> {
    for _ in 0..10 {
        if dir
            .file_name()
            .is_some_and(|name| name == "bronze-title-model")
        {
            let direct = dir.join("vendor").join(filename);
            if direct.is_file() {
                return Some(direct);
            }
        }
        let nested = dir.join("bronze-title-model").join("vendor").join(filename);
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
        let path = std::env::temp_dir().join("qwen2.5-0.5b-instruct-q4_k_m.gguf");
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
        let from_manifest = walk_vendor_from_manifest(FILENAME);
        let vendor = vendor_weights_path();
        if vendor.is_file() {
            assert_eq!(from_manifest.as_ref(), Some(&vendor));
            assert_eq!(verify_weights(&vendor), Ok(()));
        } else {
            assert_eq!(verify_weights(&vendor), Err(WeightsError::Missing));
        }
    }

    #[test]
    fn missing_qwen_file_does_not_fetch() {
        let spec = TitleTier::Qwen05.spec().expect("qwen");
        if !candidate_paths_for(spec)
            .into_iter()
            .any(|(path, _)| path.is_file())
        {
            assert_eq!(
                weights_present_for(TitleTier::Qwen05),
                Err(WeightsError::Missing)
            );
            assert_eq!(
                verified_weights_for(TitleTier::Qwen05),
                Err(WeightsError::Missing)
            );
        }
        let infer = include_str!("infer.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("prod");
        let weights = include_str!("weights.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("prod");
        for src in [infer, weights] {
            let lower = src.to_ascii_lowercase();
            assert!(!lower.contains("huggingface.co"));
            assert!(!lower.contains("download("));
        }
    }

    #[test]
    fn manifest_pins_allow_list() {
        let manifest = include_str!("../vendor/MANIFEST");
        assert!(manifest.contains("smol-360"));
        assert!(manifest.contains(FILENAME));
        assert!(manifest.contains(SHA256_HEX));
        let qwen = include_str!("../vendor/manifests/qwen-05");
        assert!(qwen.contains("qwen2.5-0.5b-instruct-q4_k_m.gguf"));
        assert!(qwen.contains("74a4da8c9fdbcd15bd1f6d01d621410d31c6fc00986f5eb687824e7b93d7a9db"));
        assert!(qwen.contains("Qwen/Qwen2.5-0.5B-Instruct-GGUF"));
        let smol135 = include_str!("../vendor/manifests/smol-135");
        assert!(
            smol135.contains("2e8040ceae7815abe0dcb3540b9995eaa1fa0d2ca9e797d0a635ae4433c68c2d")
        );
    }
}
