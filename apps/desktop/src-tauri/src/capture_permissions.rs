//! Used-permission prompts for native start, first capture, and health retest (SET-003, SET-004).

use bronze_platform_macos::{prompt_used_permissions, PromptAttempt, PromptReason};
use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PermissionPromptDto {
    pub input_monitoring: String,
    pub accessibility: String,
    pub listen_requested: bool,
    pub accessibility_requested: bool,
    pub screen_recording_requested: bool,
    pub privacy_settings_input_monitoring: Option<String>,
    pub privacy_settings_accessibility: Option<String>,
}

impl PermissionPromptDto {
    pub fn from_attempt(attempt: PromptAttempt) -> Self {
        Self {
            input_monitoring: attempt.snapshot.input_monitoring.as_str().to_string(),
            accessibility: attempt.snapshot.accessibility.as_str().to_string(),
            listen_requested: attempt.listen_requested,
            accessibility_requested: attempt.accessibility_requested,
            screen_recording_requested: attempt.screen_recording_requested,
            privacy_settings_input_monitoring: bronze_settings::privacy_settings_url(
                bronze_settings::PermissionCapability::InputMonitoring,
            )
            .map(str::to_string),
            privacy_settings_accessibility: bronze_settings::privacy_settings_url(
                bronze_settings::PermissionCapability::Accessibility,
            )
            .map(str::to_string),
        }
    }
}

#[cfg(target_os = "macos")]
pub fn prompt_on_native_start() -> PromptAttempt {
    prompt_used_permissions(
        &bronze_platform_macos::MacosPreflightHost,
        PromptReason::NativeStart,
    )
}

#[cfg(target_os = "macos")]
pub fn prompt_notification_if_undetermined() -> bronze_platform_macos::NoticeAuthorization {
    bronze_platform_macos::prompt_notification_authorization_if_needed()
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn notification_authorization_status() -> String {
    bronze_platform_macos::notification_authorization_status()
        .as_str()
        .to_string()
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn request_notification_authorization() -> String {
    bronze_platform_macos::request_notification_authorization()
        .as_str()
        .to_string()
}

#[cfg(target_os = "macos")]
pub fn prompt_on_first_capture_path() -> Option<PromptAttempt> {
    use std::sync::atomic::{AtomicBool, Ordering};
    static DONE: AtomicBool = AtomicBool::new(false);
    if DONE.swap(true, Ordering::SeqCst) {
        return None;
    }
    Some(prompt_used_permissions(
        &bronze_platform_macos::MacosPreflightHost,
        PromptReason::FirstCapture,
    ))
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn retest_used_permissions() -> PermissionPromptDto {
    let attempt = prompt_used_permissions(
        &bronze_platform_macos::MacosPreflightHost,
        PromptReason::HealthRetest,
    );
    PermissionPromptDto::from_attempt(attempt)
}

pub fn privacy_settings_open_target(capability: &str) -> Option<&'static str> {
    match capability {
        "inputMonitoring" => bronze_settings::privacy_settings_url(
            bronze_settings::PermissionCapability::InputMonitoring,
        ),
        "accessibility" => bronze_settings::privacy_settings_url(
            bronze_settings::PermissionCapability::Accessibility,
        ),
        "notifications" => {
            Some("x-apple.systempreferences:com.apple.Notifications-Settings.extension")
        }
        _ => None,
    }
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn open_privacy_settings(capability: String) -> Result<(), String> {
    let Some(url) = privacy_settings_open_target(&capability) else {
        return Err("capability_not_used".into());
    };
    std::process::Command::new("/usr/bin/open")
        .arg(url)
        .status()
        .map_err(|err| err.to_string())
        .and_then(|status| {
            if status.success() {
                Ok(())
            } else {
                Err("open_failed".into())
            }
        })
}

#[cfg(test)]
mod tests {
    use super::{privacy_settings_open_target, PermissionPromptDto};
    use bronze_platform_macos::{
        prompt_used_permissions, PermissionRequestHost, PreflightError, PreflightHost, PromptReason,
    };
    use bronze_settings::PermissionState;

    struct FakeHost;

    impl PreflightHost for FakeHost {
        fn listen_event_access(&self) -> Result<bool, PreflightError> {
            Ok(false)
        }

        fn accessibility_trusted(&self) -> Result<bool, PreflightError> {
            Ok(false)
        }
    }

    impl PermissionRequestHost for FakeHost {
        fn request_listen_event_access(&self) -> Result<bool, PreflightError> {
            Ok(false)
        }

        fn request_accessibility_trusted(&self) -> Result<bool, PreflightError> {
            Ok(false)
        }
    }

    #[test]
    fn dto_maps_denied_request_and_never_opens_screen_recording() {
        let attempt = prompt_used_permissions(&FakeHost, PromptReason::HealthRetest);
        let dto = PermissionPromptDto::from_attempt(attempt);
        assert_eq!(dto.input_monitoring, PermissionState::Denied.as_str());
        assert_eq!(dto.accessibility, PermissionState::Denied.as_str());
        assert!(dto.listen_requested);
        assert!(dto.accessibility_requested);
        assert!(!dto.screen_recording_requested);
        assert!(dto.privacy_settings_input_monitoring.is_some());
        assert!(dto.privacy_settings_accessibility.is_some());
        assert_eq!(privacy_settings_open_target("screenRecording"), None);
        assert_eq!(privacy_settings_open_target("automation"), None);
        assert!(privacy_settings_open_target("inputMonitoring").is_some());
        assert!(privacy_settings_open_target("accessibility").is_some());
        assert!(privacy_settings_open_target("notifications").is_some());
    }

    #[test]
    fn desktop_start_requests_used_permissions_not_screen_recording() {
        let src = include_str!("lib.rs");
        assert!(src.contains("prompt_on_native_start"));
        assert!(src.contains("prompt_notification_if_undetermined"));
        assert!(src.contains("prompt_on_first_capture_path"));
        let used = include_str!("../permissions/used-permissions.toml");
        assert!(used.contains("notification_authorization_status"));
        assert!(used.contains("request_notification_authorization"));
        assert!(!src.contains("CGRequestScreenCaptureAccess"));
        assert!(!src.contains("NSScreenCapture"));
        let perms = include_str!("capture_permissions.rs");
        let (production, tests) = perms
            .split_once("#[cfg(test)]")
            .expect("production and tests");
        assert!(production.contains("PromptReason::NativeStart"));
        assert!(production.contains("prompt_notification_if_undetermined"));
        assert!(production.contains("notification_authorization_status"));
        assert!(production.contains("request_notification_authorization"));
        assert!(production.contains("PromptReason::FirstCapture"));
        assert!(production.contains("PromptReason::HealthRetest"));
        assert!(!production.contains("CGRequestScreenCaptureAccess"));
        let live_host = format!("{}{}{}", "Macos", "Preflight", "Host");
        assert!(
            !tests.contains(&live_host),
            "tests must use a fake host only and never invoke the live host"
        );
    }
}
