//! Used-permission prompts for native start, first capture, and health retest (SET-003, SET-004).

use std::path::{Path, PathBuf};

use bronze_platform_macos::{
    prompt_used_permissions, PromptAttempt, PromptLedger, PromptReason, ShownPrompts,
};
use serde::{Deserialize, Serialize};

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

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
struct PromptRecord {
    accessibility: String,
    listen: String,
}

/// Remembers which binary already showed each system prompt.
///
/// Identity is the executable path, mtime, and length. Relaunching that
/// binary does not prompt again. A replaced binary has a new identity.
pub struct FilePromptLedger {
    path: PathBuf,
    identity: String,
}

impl FilePromptLedger {
    pub fn at(path: PathBuf, identity: impl Into<String>) -> Self {
        Self {
            path,
            identity: identity.into(),
        }
    }

    pub fn for_data_dir(data_dir: &Path) -> Self {
        Self::at(
            data_dir.join("permission-prompts.json"),
            current_binary_identity(),
        )
    }

    fn load(&self) -> PromptRecord {
        let Ok(raw) = std::fs::read(&self.path) else {
            return PromptRecord::default();
        };
        serde_json::from_slice(&raw).unwrap_or_default()
    }
}

impl PromptLedger for FilePromptLedger {
    fn shown(&self) -> ShownPrompts {
        if self.identity.is_empty() {
            return ShownPrompts::default();
        }
        let record = self.load();
        ShownPrompts {
            accessibility: record.accessibility == self.identity,
            listen: record.listen == self.identity,
        }
    }

    fn remember(&self, shown: ShownPrompts) {
        if self.identity.is_empty() {
            return;
        }
        let mut record = self.load();
        if shown.accessibility {
            record.accessibility = self.identity.clone();
        }
        if shown.listen {
            record.listen = self.identity.clone();
        }
        if let Some(parent) = self.path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let Ok(raw) = serde_json::to_vec(&record) else {
            return;
        };
        if std::fs::write(&self.path, raw).is_ok() {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ =
                    std::fs::set_permissions(&self.path, std::fs::Permissions::from_mode(0o600));
            }
        }
    }
}

fn current_binary_identity() -> String {
    let Ok(path) = std::env::current_exe() else {
        return String::new();
    };
    let Ok(meta) = std::fs::metadata(&path) else {
        return String::new();
    };
    let modified = meta
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    format!("{}:{modified}:{}", path.display(), meta.len())
}

#[cfg(target_os = "macos")]
pub fn prompt_on_native_start(data_dir: &Path) -> PromptAttempt {
    prompt_used_permissions(
        &bronze_platform_macos::MacosPreflightHost,
        PromptReason::NativeStart,
        &FilePromptLedger::for_data_dir(data_dir),
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
pub fn prompt_on_first_capture_path(data_dir: &Path) -> Option<PromptAttempt> {
    use std::sync::atomic::{AtomicBool, Ordering};
    static DONE: AtomicBool = AtomicBool::new(false);
    if DONE.swap(true, Ordering::SeqCst) {
        return None;
    }
    Some(prompt_used_permissions(
        &bronze_platform_macos::MacosPreflightHost,
        PromptReason::FirstCapture,
        &FilePromptLedger::for_data_dir(data_dir),
    ))
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn read_used_permissions() -> PermissionPromptDto {
    let snapshot =
        bronze_platform_macos::snapshot_from_preflight(&bronze_platform_macos::MacosPreflightHost);
    PermissionPromptDto::from_attempt(PromptAttempt {
        snapshot,
        listen_requested: false,
        accessibility_requested: false,
        screen_recording_requested: false,
    })
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn retest_used_permissions(app: tauri::AppHandle) -> PermissionPromptDto {
    use tauri::Manager;
    let dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::temp_dir());
    let attempt = prompt_used_permissions(
        &bronze_platform_macos::MacosPreflightHost,
        PromptReason::HealthRetest,
        &FilePromptLedger::for_data_dir(&dir),
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
        "launchAtLogin" => bronze_settings::privacy_settings_url(
            bronze_settings::PermissionCapability::LaunchAtLogin,
        ),
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
    use super::FilePromptLedger;
    use super::{privacy_settings_open_target, PermissionPromptDto};
    use bronze_platform_macos::{
        prompt_used_permissions, PermissionRequestHost, PreflightError, PreflightHost, PromptReason,
    };
    use bronze_settings::PermissionState;
    use std::cell::Cell;

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
        let attempt = prompt_used_permissions(
            &FakeHost,
            PromptReason::HealthRetest,
            &bronze_platform_macos::MemoryPromptLedger::default(),
        );
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
        assert!(privacy_settings_open_target("launchAtLogin").is_some());
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
        assert!(production.contains("read_used_permissions"));
        assert!(production.contains("permission-prompts.json"));
        assert!(production.contains("FilePromptLedger"));
        assert!(!production.contains("CGRequestScreenCaptureAccess"));
        let live_host = format!("{}{}{}", "Macos", "Preflight", "Host");
        assert!(
            !tests.contains(&live_host),
            "tests must use a fake host only and never invoke the live host"
        );
    }

    #[test]
    fn remembered_prompt_is_not_repeated_for_the_same_binary() {
        struct CountingHost {
            requests: Cell<u32>,
        }

        impl PreflightHost for CountingHost {
            fn listen_event_access(&self) -> Result<bool, PreflightError> {
                Ok(false)
            }

            fn accessibility_trusted(&self) -> Result<bool, PreflightError> {
                Ok(false)
            }
        }

        impl PermissionRequestHost for CountingHost {
            fn request_listen_event_access(&self) -> Result<bool, PreflightError> {
                self.requests.set(self.requests.get() + 1);
                Ok(false)
            }

            fn request_accessibility_trusted(&self) -> Result<bool, PreflightError> {
                self.requests.set(self.requests.get() + 1);
                Ok(false)
            }
        }

        let dir = std::env::temp_dir().join(format!(
            "bronze-prompt-ledger-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let path = dir.join("permission-prompts.json");
        let host = CountingHost {
            requests: Cell::new(0),
        };
        let first = FilePromptLedger::at(path.clone(), "binary-a");
        let opened = prompt_used_permissions(&host, PromptReason::NativeStart, &first);
        assert!(opened.listen_requested);
        assert!(opened.accessibility_requested);
        assert_eq!(host.requests.get(), 2);

        let relaunch = FilePromptLedger::at(path.clone(), "binary-a");
        let again = prompt_used_permissions(&host, PromptReason::NativeStart, &relaunch);
        assert!(!again.listen_requested);
        assert!(!again.accessibility_requested);
        assert_eq!(again.snapshot.accessibility, PermissionState::Denied);
        assert_eq!(host.requests.get(), 2);

        let replaced = FilePromptLedger::at(path, "binary-b");
        let next_binary = prompt_used_permissions(&host, PromptReason::NativeStart, &replaced);
        assert!(next_binary.accessibility_requested);
        assert!(next_binary.listen_requested);
        assert_eq!(host.requests.get(), 4);
        let _ = std::fs::remove_dir_all(dir);
    }
}
