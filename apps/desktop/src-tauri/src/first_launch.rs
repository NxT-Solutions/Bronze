//! First-launch title-model setup gate for the Quick Panel splash.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use bronze_title_model::{
    auto_pick_title_tier, current_status, present_gguf_tiers, verified_weights_for, FallbackReason,
    TitleEngineStatus, TitleTier, GGUF_TIERS,
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use crate::live_session::physical_ram_bytes;

pub const FIRST_LAUNCH_SETUP_EVENT: &str = "first-launch-setup";
const READY_VERSION: u32 = 1;

static DATA_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FirstLaunchSetupDto {
    pub needed: bool,
    pub phase: String,
    pub model: Option<String>,
    pub bytes_read: Option<u64>,
    pub bytes_total: Option<u64>,
    pub error: Option<String>,
    pub complete: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct ReadyMarker {
    v: u32,
    sha256: Vec<String>,
}

pub fn bind_data_dir(dir: PathBuf) {
    *DATA_DIR
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(dir);
}

pub fn emit_from_engine(app: &AppHandle, status: TitleEngineStatus) {
    let _ = app.emit(FIRST_LAUNCH_SETUP_EVENT, snapshot_from_engine(status));
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn first_launch_setup() -> FirstLaunchSetupDto {
    snapshot_from_engine(current_status())
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn retry_first_launch_setup() -> FirstLaunchSetupDto {
    clear_ready_marker();
    bronze_title_model::warmup();
    snapshot_from_engine(current_status())
}

pub fn needs_setup() -> bool {
    snapshot_from_engine(current_status()).needed
}

pub fn finish_first_launch_setup() -> FirstLaunchSetupDto {
    remember_ready();
    snapshot_from_engine(current_status())
}

pub fn snapshot_from_engine(status: TitleEngineStatus) -> FirstLaunchSetupDto {
    let marker_ok = ready_marker_matches();
    let required = required_tier();
    let model = named_model(status.tier, required);
    if marker_ok {
        return FirstLaunchSetupDto {
            needed: false,
            phase: "ready".into(),
            model,
            bytes_read: None,
            bytes_total: None,
            error: None,
            complete: true,
        };
    }
    let mut phase = splash_phase(status, required);
    let complete = phase == "ready" || (required == TitleTier::Extractive && phase == "finishing");
    if phase == "ready" {
        phase = "finishing".into();
    }
    let error = splash_error(status, required, &phase);
    FirstLaunchSetupDto {
        needed: true,
        phase,
        model,
        bytes_read: status.bytes_read,
        bytes_total: status.bytes_total,
        error,
        complete,
    }
}

fn splash_phase(status: TitleEngineStatus, required: TitleTier) -> String {
    match status.phase.as_str() {
        "downloading" => "downloading".into(),
        "hashing" => "checking".into(),
        "loading" => "checking".into(),
        "ready" => "ready".into(),
        "failed" | "missing" => {
            if required == TitleTier::Extractive {
                "ready".into()
            } else {
                "failed".into()
            }
        }
        "idle" => {
            if required == TitleTier::Extractive {
                "finishing".into()
            } else {
                "checking".into()
            }
        }
        other => other.into(),
    }
}

fn splash_error(status: TitleEngineStatus, required: TitleTier, phase: &str) -> Option<String> {
    if phase != "failed" || required == TitleTier::Extractive {
        return None;
    }
    Some(
        match status.reason.unwrap_or(FallbackReason::Unreadable) {
            FallbackReason::MissingWeights => "missing",
            FallbackReason::BadHash => "bad_hash",
            other => {
                let raw = other.as_str();
                if raw == "download_failed" {
                    "download_failed"
                } else {
                    "unreadable"
                }
            }
        }
        .into(),
    )
}

fn named_model(status_tier: TitleTier, required: TitleTier) -> Option<String> {
    let tier = if status_tier == TitleTier::Extractive {
        required
    } else {
        status_tier
    };
    match tier {
        TitleTier::Smol135 | TitleTier::Smol360 | TitleTier::Qwen05 => Some(tier.as_str().into()),
        _ => None,
    }
}

fn required_tier() -> TitleTier {
    auto_pick_title_tier(physical_ram_bytes(), &present_gguf_tiers())
}

fn ready_path() -> Option<PathBuf> {
    DATA_DIR
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .as_ref()
        .map(|dir| dir.join("title-models").join("ready.json"))
}

fn read_marker(path: &Path) -> Option<ReadyMarker> {
    let raw = fs::read(path).ok()?;
    let marker: ReadyMarker = serde_json::from_slice(&raw).ok()?;
    if marker.v != READY_VERSION {
        return None;
    }
    Some(marker)
}

fn verified_checksums() -> Vec<String> {
    let mut out = Vec::new();
    for spec in GGUF_TIERS {
        if verified_weights_for(spec.tier).is_ok() {
            out.push(spec.sha256_hex.to_string());
        }
    }
    out.sort();
    out.dedup();
    out
}

fn ready_marker_matches() -> bool {
    let Some(path) = ready_path() else {
        return false;
    };
    let Some(marker) = read_marker(&path) else {
        return false;
    };
    let required = required_tier();
    if marker.sha256.is_empty() {
        return required == TitleTier::Extractive;
    }
    let verified = verified_checksums();
    !marker.sha256.is_empty()
        && marker
            .sha256
            .iter()
            .all(|sha| verified.iter().any(|got| got == sha))
}

fn remember_ready() {
    let Some(path) = ready_path() else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let required = required_tier();
    let sha256 = if required == TitleTier::Extractive {
        Vec::new()
    } else {
        let verified = verified_checksums();
        if verified.is_empty() {
            GGUF_TIERS
                .iter()
                .find(|spec| spec.tier == required)
                .map(|spec| vec![spec.sha256_hex.to_string()])
                .unwrap_or_default()
        } else {
            verified
        }
    };
    let blob = serde_json::to_vec(&ReadyMarker {
        v: READY_VERSION,
        sha256,
    });
    if let Ok(blob) = blob {
        let _ = fs::write(path, blob);
    }
}

fn clear_ready_marker() {
    if let Some(path) = ready_path() {
        let _ = fs::remove_file(path);
    }
}

#[cfg(test)]
mod first_launch_tests {
    use super::*;
    use bronze_title_model::{apply_diag, EnginePhase};
    use std::sync::Mutex;

    static TEST_DIR_LOCK: Mutex<()> = Mutex::new(());

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "bronze-first-launch-{name}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("temp");
        dir
    }

    fn lock_data_dir() -> std::sync::MutexGuard<'static, ()> {
        TEST_DIR_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[test]
    fn matching_marker_skips_splash() {
        let _lock = lock_data_dir();
        let dir = temp_dir("skip");
        bind_data_dir(dir.clone());
        let path = dir.join("title-models").join("ready.json");
        fs::create_dir_all(path.parent().expect("parent")).expect("dir");
        fs::write(&path, r#"{"v":1,"sha256":[]}"#).expect("marker");
        let dto = snapshot_from_engine(TitleEngineStatus::idle());
        if required_tier() == TitleTier::Extractive {
            assert!(!dto.needed);
            assert_eq!(dto.phase, "ready");
        }
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_marker_shows_checking() {
        let _lock = lock_data_dir();
        let dir = temp_dir("check");
        bind_data_dir(dir.clone());
        let dto = snapshot_from_engine(TitleEngineStatus::idle());
        assert!(dto.needed);
        assert!(dto.phase == "checking" || dto.phase == "finishing");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn progress_bytes_pass_through_while_downloading() {
        let _lock = lock_data_dir();
        let loading = apply_diag(TitleEngineStatus::idle(), "switch scheduled tier=smol-360");
        let reading = apply_diag(loading, "read progress bytes=50 total=100");
        let dir = temp_dir("bytes");
        bind_data_dir(dir.clone());
        let dto = snapshot_from_engine(reading);
        assert!(dto.needed);
        assert_eq!(dto.bytes_read, Some(50));
        assert_eq!(dto.bytes_total, Some(100));
        assert_eq!(dto.phase, "checking");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn hashing_is_checking_and_ready_writes_marker() {
        let _lock = lock_data_dir();
        let loading = apply_diag(TitleEngineStatus::idle(), "switch scheduled tier=qwen-05");
        let hashing = apply_diag(loading, "hash ok");
        let dir = temp_dir("hash");
        bind_data_dir(dir.clone());
        let checking = snapshot_from_engine(hashing);
        assert!(checking.needed);
        assert_eq!(checking.phase, "checking");
        let ready = apply_diag(hashing, "model loaded");
        assert_eq!(ready.phase, EnginePhase::Ready);
        let finishing = snapshot_from_engine(ready);
        assert!(finishing.needed);
        assert_eq!(finishing.phase, "finishing");
        assert!(finishing.complete);
        assert!(!dir.join("title-models").join("ready.json").is_file());
        remember_ready();
        assert!(dir.join("title-models").join("ready.json").is_file());
        let again = snapshot_from_engine(TitleEngineStatus::idle());
        if required_tier() == TitleTier::Extractive || !verified_checksums().is_empty() {
            assert!(!again.needed);
            assert_eq!(again.phase, "ready");
        } else {
            assert!(again.needed);
        }
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn failed_model_keeps_splash_and_retry_clears_marker() {
        let _lock = lock_data_dir();
        let loading = apply_diag(TitleEngineStatus::idle(), "switch scheduled tier=smol-135");
        let failed = apply_diag(loading, "fallback reason=bad_hash");
        let dir = temp_dir("fail");
        bind_data_dir(dir.clone());
        let dto = snapshot_from_engine(failed);
        if required_tier() != TitleTier::Extractive {
            assert!(dto.needed);
            assert_eq!(dto.phase, "failed");
            assert_eq!(dto.error.as_deref(), Some("bad_hash"));
        }
        remember_ready();
        clear_ready_marker();
        assert!(!dir.join("title-models").join("ready.json").is_file());
        let src = include_str!("first_launch.rs");
        assert!(src.contains("first-launch-setup"));
        assert!(src.contains("download_failed"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn download_started_diag_is_downloading_when_present() {
        let _lock = lock_data_dir();
        let loading = apply_diag(TitleEngineStatus::idle(), "switch scheduled tier=smol-360");
        let downloading = apply_diag(loading, "download started");
        let dir = temp_dir("dl");
        bind_data_dir(dir.clone());
        let dto = snapshot_from_engine(downloading);
        if downloading.phase.as_str() == "downloading" {
            assert_eq!(dto.phase, "downloading");
            assert!(dto.needed);
        }
        let _ = fs::remove_dir_all(&dir);
    }
}
