//! First-run wizard state (SET-003, SET-004). Completing writes a local flag.
//! Title-model splash lives on the Quick Panel (`first_launch`); this module
//! owns the onboarding window and a fallback splash pane if models are still
//! missing when the wizard opens.

use std::path::{Path, PathBuf};

use bronze_title_model::{any_candidate_file, current_status, EnginePhase};
use serde::{Deserialize, Serialize};

pub const ONBOARDING_STATE_FILE: &str = "onboarding-state.json";

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct OnboardingState {
    #[serde(default)]
    pub completed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FirstRunStatusDto {
    pub onboarding_complete: bool,
    pub needs_wizard: bool,
    pub needs_splash: bool,
    pub title_phase: String,
    pub title_tier: String,
    pub title_reason: Option<String>,
    pub bytes_read: Option<u64>,
    pub bytes_total: Option<u64>,
}

pub fn state_path(data_dir: &Path) -> PathBuf {
    data_dir.join(ONBOARDING_STATE_FILE)
}

pub fn load_state(data_dir: &Path) -> OnboardingState {
    let Ok(raw) = std::fs::read(state_path(data_dir)) else {
        return OnboardingState::default();
    };
    serde_json::from_slice(&raw).unwrap_or_default()
}

pub fn write_state(data_dir: &Path, state: &OnboardingState) -> Result<(), String> {
    if let Some(parent) = state_path(data_dir).parent() {
        std::fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let raw = serde_json::to_vec(state).map_err(|err| err.to_string())?;
    std::fs::write(state_path(data_dir), raw).map_err(|err| err.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ =
            std::fs::set_permissions(state_path(data_dir), std::fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

pub fn looks_like_existing_install(data_dir: &Path) -> bool {
    data_dir.join("settings.json").is_file() || data_dir.join("permission-prompts.json").is_file()
}

pub fn needs_splash() -> bool {
    if !any_candidate_file() {
        return true;
    }
    matches!(
        current_status().phase,
        EnginePhase::Missing | EnginePhase::Loading | EnginePhase::Hashing
    )
}

pub fn status_for(data_dir: &Path) -> FirstRunStatusDto {
    let mut completed = load_state(data_dir).completed;
    if !completed && looks_like_existing_install(data_dir) {
        let _ = write_state(data_dir, &OnboardingState { completed: true });
        completed = true;
    }
    let title = current_status();
    let splash = !completed && needs_splash();
    FirstRunStatusDto {
        onboarding_complete: completed,
        needs_wizard: !completed,
        needs_splash: splash,
        title_phase: title.phase.as_str().to_string(),
        title_tier: title.tier.as_str().to_string(),
        title_reason: title.reason.map(|reason| reason.as_str().into()),
        bytes_read: title.bytes_read,
        bytes_total: title.bytes_total,
    }
}

pub fn needs_first_run(data_dir: &Path) -> bool {
    status_for(data_dir).needs_wizard
}

pub fn mark_completed(data_dir: &Path) -> Result<FirstRunStatusDto, String> {
    write_state(data_dir, &OnboardingState { completed: true })?;
    Ok(status_for(data_dir))
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn first_run_status(app: tauri::AppHandle) -> FirstRunStatusDto {
    use tauri::Manager;
    let dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::temp_dir());
    status_for(&dir)
}

#[cfg(test)]
mod tests {
    use super::{
        load_state, looks_like_existing_install, mark_completed, needs_first_run, status_for,
        write_state, OnboardingState,
    };
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "bronze-onboarding-{label}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ))
    }

    #[test]
    fn empty_data_dir_needs_the_wizard() {
        let dir = temp_dir("empty");
        std::fs::create_dir_all(&dir).expect("dir");
        assert!(needs_first_run(&dir));
        let status = status_for(&dir);
        assert!(!status.onboarding_complete);
        assert!(status.needs_wizard);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn finish_marks_onboarding_complete() {
        let dir = temp_dir("finish");
        std::fs::create_dir_all(&dir).expect("dir");
        let status = mark_completed(&dir).expect("complete");
        assert!(status.onboarding_complete);
        assert!(!status.needs_wizard);
        assert!(!status.needs_splash);
        assert!(!needs_first_run(&dir));
        assert!(load_state(&dir).completed);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn existing_settings_skip_the_wizard() {
        let dir = temp_dir("existing");
        std::fs::create_dir_all(&dir).expect("dir");
        std::fs::write(dir.join("settings.json"), b"{}").expect("settings");
        assert!(looks_like_existing_install(&dir));
        assert!(!needs_first_run(&dir));
        assert!(load_state(&dir).completed);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn permission_ledger_also_grandfathers() {
        let dir = temp_dir("ledger");
        std::fs::create_dir_all(&dir).expect("dir");
        std::fs::write(dir.join("permission-prompts.json"), b"{}").expect("ledger");
        assert!(!needs_first_run(&dir));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn written_complete_flag_survives_reload() {
        let dir = temp_dir("persist");
        std::fs::create_dir_all(&dir).expect("dir");
        write_state(&dir, &OnboardingState { completed: true }).expect("write");
        assert!(!needs_first_run(&dir));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn onboarding_html_is_the_first_run_surface() {
        let html = include_str!("../../src/onboarding.html");
        assert!(html.contains("data-onboarding-step=\"welcome\""));
        assert!(html.contains("data-onboarding-step=\"permissions\""));
        assert!(html.contains("data-permission-retest=\"accessibility\""));
        assert!(html.contains("onboarding-media/bronze-queue.png"));
        assert!(html.contains("role=\"status\""));
    }
}
