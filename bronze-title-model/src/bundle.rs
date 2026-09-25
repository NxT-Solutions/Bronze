use crate::tiers::{TitleTier, GGUF_TIERS};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

pub const BUNDLE_MODELS_DIR: &str = "models";

pub fn vendor_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("vendor")
}

pub fn staged_model_filenames() -> Vec<&'static str> {
    GGUF_TIERS.iter().map(|spec| spec.filename).collect()
}

pub fn stage_bundled_models(vendor: &Path, dest: &Path) -> Vec<(TitleTier, PathBuf)> {
    let _ = fs::create_dir_all(dest);
    let mut copied = Vec::new();
    for spec in GGUF_TIERS {
        let src = vendor.join(spec.filename);
        if !src.is_file() {
            continue;
        }
        if !hash_matches(&src, spec.sha256_hex) {
            continue;
        }
        let out = dest.join(spec.filename);
        if fs::copy(&src, &out).is_ok() {
            copied.push((spec.tier, out));
        }
    }
    copied
}

fn hash_matches(path: &Path, expected: &str) -> bool {
    let Ok(mut file) = fs::File::open(path) else {
        return false;
    };
    let mut hasher = Sha256::new();
    let mut buf = [0_u8; 8192];
    loop {
        let Ok(n) = file.read(&mut buf) else {
            return false;
        };
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    hex_lower(&hasher.finalize()) == expected
}

fn hex_lower(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tiers::GGUF_TIERS;

    #[test]
    fn staged_names_match_allow_list() {
        let names = staged_model_filenames();
        assert_eq!(names.len(), 3);
        assert_eq!(names[0], GGUF_TIERS[0].filename);
        assert_eq!(names[1], GGUF_TIERS[1].filename);
        assert_eq!(names[2], GGUF_TIERS[2].filename);
    }

    #[test]
    fn stage_skips_missing_and_bad_hash() {
        let tmp = std::env::temp_dir().join(format!("bronze-stage-{}", std::process::id()));
        let vendor = tmp.join("vendor");
        let dest = tmp.join("models");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&vendor).expect("vendor");
        fs::write(vendor.join(GGUF_TIERS[0].filename), b"not-a-gguf").expect("write");
        let copied = stage_bundled_models(&vendor, &dest);
        assert!(copied.is_empty());
        assert!(!dest.join(GGUF_TIERS[0].filename).is_file());
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn vendor_dir_points_at_crate_vendor() {
        assert_eq!(
            vendor_dir().file_name().and_then(|n| n.to_str()),
            Some("vendor")
        );
    }
}
