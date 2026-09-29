use crate::tiers::TierSpec;
use crate::weights::{cache_dir, WeightsError};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub fn pin_url(spec: &TierSpec) -> String {
    format!(
        "https://huggingface.co/{}/resolve/{}/{}",
        spec.hf_gguf_repo, spec.hf_revision, spec.filename
    )
}

pub fn cache_file(cache: &Path, spec: &TierSpec) -> PathBuf {
    cache.join(spec.sha256_hex).join(spec.filename)
}

pub fn ensure_cached_for(spec: &TierSpec) -> Result<PathBuf, WeightsError> {
    let Some(dir) = cache_dir() else {
        return Err(WeightsError::Missing);
    };
    persist_pin(&dir, spec, &UreqFetch)
}

pub(crate) fn persist_pin(
    cache: &Path,
    spec: &TierSpec,
    fetch: &dyn PinFetch,
) -> Result<PathBuf, WeightsError> {
    let dest = cache_file(cache, spec);
    if dest.is_file() && verify_downloaded(&dest, spec.sha256_hex).is_ok() {
        return Ok(dest);
    }
    let url = pin_url(spec);
    if !url.starts_with("https://") {
        return Err(WeightsError::DownloadFailed);
    }
    crate::emit_diag("download started");
    let parent = dest.parent().ok_or(WeightsError::Unreadable)?;
    create_cache_dir(parent)?;
    let part = parent.join(format!("{}.part", spec.filename));
    let _ = fs::remove_file(&part);
    let mut last_percent = None;
    fetch.fetch(&url, spec.bytes, &part, &mut |read, total| {
        note_download(read, total, &mut last_percent);
    })?;
    if verify_downloaded(&part, spec.sha256_hex).is_err() {
        let _ = fs::remove_file(&part);
        return Err(WeightsError::HashMismatch);
    }
    fs::rename(&part, &dest).map_err(|_| WeightsError::Unreadable)?;
    restrict_file(&dest);
    Ok(dest)
}

pub(crate) trait PinFetch {
    fn fetch(
        &self,
        url: &str,
        expected_bytes: u64,
        dest: &Path,
        on_progress: &mut dyn FnMut(u64, u64),
    ) -> Result<(), WeightsError>;
}

struct UreqFetch;

impl PinFetch for UreqFetch {
    fn fetch(
        &self,
        url: &str,
        expected_bytes: u64,
        dest: &Path,
        on_progress: &mut dyn FnMut(u64, u64),
    ) -> Result<(), WeightsError> {
        if !url.starts_with("https://") {
            return Err(WeightsError::DownloadFailed);
        }
        let response = http_agent()
            .get(url)
            .set(
                "User-Agent",
                "Bronze-title-model (https://github.com/NxT-Solutions/Bronze)",
            )
            .call()
            .map_err(|_| WeightsError::DownloadFailed)?;
        if response.status() != 200 {
            return Err(WeightsError::DownloadFailed);
        }
        let total = response
            .header("Content-Length")
            .and_then(|raw| raw.parse().ok())
            .filter(|n: &u64| *n > 0)
            .unwrap_or(expected_bytes);
        let mut reader = response.into_reader();
        let mut file = File::create(dest).map_err(|_| WeightsError::Unreadable)?;
        let mut buf = [0_u8; 64 * 1024];
        let mut read = 0u64;
        on_progress(0, total);
        loop {
            let n = reader
                .read(&mut buf)
                .map_err(|_| WeightsError::DownloadFailed)?;
            if n == 0 {
                break;
            }
            file.write_all(&buf[..n])
                .map_err(|_| WeightsError::Unreadable)?;
            read = read.saturating_add(n as u64);
            on_progress(read, total);
        }
        file.flush().map_err(|_| WeightsError::Unreadable)?;
        Ok(())
    }
}

fn http_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(30))
        .timeout_read(Duration::from_secs(120))
        .timeout_write(Duration::from_secs(30))
        .build()
}

fn note_download(bytes_read: u64, bytes_total: u64, last_percent: &mut Option<u8>) {
    if bytes_total == 0 {
        return;
    }
    let Some(percent) = crate::weights::next_read_percent(*last_percent, bytes_read, bytes_total)
    else {
        return;
    };
    *last_percent = Some(percent);
    crate::status::note_read_progress(bytes_read, bytes_total);
}

fn verify_downloaded(path: &Path, sha256_hex: &str) -> Result<(), WeightsError> {
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

fn create_cache_dir(path: &Path) -> Result<(), WeightsError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(path)
            .map_err(|_| WeightsError::Unreadable)?;
    }
    #[cfg(not(unix))]
    {
        fs::create_dir_all(path).map_err(|_| WeightsError::Unreadable)?;
    }
    Ok(())
}

fn restrict_file(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
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
mod fetch_tests {
    use super::*;
    use crate::tiers::TitleTier;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct ScriptedFetch {
        body: Vec<u8>,
        calls: AtomicUsize,
        fail: bool,
    }

    impl PinFetch for ScriptedFetch {
        fn fetch(
            &self,
            url: &str,
            expected_bytes: u64,
            dest: &Path,
            on_progress: &mut dyn FnMut(u64, u64),
        ) -> Result<(), WeightsError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            assert!(url.starts_with("https://"));
            if self.fail {
                return Err(WeightsError::DownloadFailed);
            }
            on_progress(0, expected_bytes.max(self.body.len() as u64));
            fs::write(dest, &self.body).expect("write part");
            on_progress(
                self.body.len() as u64,
                expected_bytes.max(self.body.len() as u64),
            );
            Ok(())
        }
    }

    fn sha_of(bytes: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        hex_lower(&hasher.finalize())
    }

    fn spec_with_sha(sha: &str) -> TierSpec {
        let mut spec = *TitleTier::Smol135.spec().expect("smol-135");
        spec.sha256_hex = {
            // TierSpec has sha256_hex as &'static str. Leak for the test.
            Box::leak(sha.to_string().into_boxed_str())
        };
        spec.filename = "pin-test.gguf";
        spec.bytes = 5;
        spec
    }

    #[test]
    fn pin_url_is_https_huggingface_revision() {
        let spec = TitleTier::Qwen05.spec().expect("qwen");
        let url = pin_url(spec);
        assert!(url.starts_with("https://huggingface.co/"));
        assert!(url.contains(spec.hf_gguf_repo));
        assert!(url.contains(spec.hf_revision));
        assert!(url.ends_with(spec.filename));
        assert!(!url.contains(" "));
    }

    #[test]
    fn cache_hit_skips_fetch() {
        let tmp = std::env::temp_dir().join(format!("bronze-pin-hit-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        let body = b"GGUF";
        let sha = sha_of(body);
        let spec = spec_with_sha(&sha);
        let dest = cache_file(&tmp, &spec);
        fs::create_dir_all(dest.parent().expect("parent")).expect("dir");
        fs::write(&dest, body).expect("seed");
        let fetch = ScriptedFetch {
            body: body.to_vec(),
            calls: AtomicUsize::new(0),
            fail: false,
        };
        let got = persist_pin(&tmp, &spec, &fetch).expect("hit");
        assert_eq!(got, dest);
        assert_eq!(fetch.calls.load(Ordering::SeqCst), 0);
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn fetch_writes_checksum_keyed_file_once() {
        let tmp = std::env::temp_dir().join(format!("bronze-pin-fetch-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        let body = b"GGUF";
        let sha = sha_of(body);
        let spec = spec_with_sha(&sha);
        let fetch = ScriptedFetch {
            body: body.to_vec(),
            calls: AtomicUsize::new(0),
            fail: false,
        };
        let got = persist_pin(&tmp, &spec, &fetch).expect("fetch");
        assert_eq!(got, cache_file(&tmp, &spec));
        assert_eq!(fs::read(&got).expect("read"), body);
        assert_eq!(fetch.calls.load(Ordering::SeqCst), 1);
        let again = persist_pin(&tmp, &spec, &fetch).expect("reuse");
        assert_eq!(again, got);
        assert_eq!(fetch.calls.load(Ordering::SeqCst), 1);
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn bad_hash_does_not_install() {
        let tmp = std::env::temp_dir().join(format!("bronze-pin-bad-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        let spec = spec_with_sha("aa".repeat(32).as_str());
        let fetch = ScriptedFetch {
            body: b"nope".to_vec(),
            calls: AtomicUsize::new(0),
            fail: false,
        };
        assert_eq!(
            persist_pin(&tmp, &spec, &fetch),
            Err(WeightsError::HashMismatch)
        );
        assert!(!cache_file(&tmp, &spec).is_file());
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn ensure_without_cache_dir_does_not_fetch() {
        assert_eq!(
            ensure_cached_for(TitleTier::Qwen05.spec().expect("qwen")),
            Err(WeightsError::Missing)
        );
        let fetch = include_str!("fetch.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("prod");
        assert!(fetch.contains("huggingface.co"));
        assert!(fetch.contains("AgentBuilder::new()"));
        assert!(!fetch.contains("proxy_from_env"));
        assert!(!fetch.contains("HTTP_PROXY"));
    }
}
