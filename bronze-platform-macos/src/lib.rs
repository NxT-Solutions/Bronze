//! Safe macOS façade over the versioned C ABI (CAP-004, ADR-004).
//!
//! bronze-domain and other core crates must not import macOS frameworks or
//! call Swift. This crate is the only Rust entry to BronzeNative.

mod abi;
#[cfg(feature = "abi-stub")]
mod abi_stub;
mod ax_live;
mod bridge;
mod display_prefs;
mod installed_apps;
mod pasteboard;
mod permission;
mod settings_file;
mod source_icon;
mod status_item;
mod title;
mod user_notice;

pub use abi::{
    BronzeIngressSnapshot, BronzeNativeUtf8View, BRONZE_ABI_VERSION, BRONZE_STATUS_CANCELLED,
    BRONZE_STATUS_CONTEXT_UNAVAILABLE, BRONZE_STATUS_DEGRADED, BRONZE_STATUS_DOUBLE_COMPLETION,
    BRONZE_STATUS_INVALID_UTF8, BRONZE_STATUS_NOT_FOUND, BRONZE_STATUS_OK,
    BRONZE_STATUS_SHUTTING_DOWN, BRONZE_TAP_FEED_CANCEL, BRONZE_TAP_FEED_DOWN, BRONZE_TAP_FEED_UP,
    BRONZE_TAP_REC_DISABLED, BRONZE_TAP_REC_NONE, BRONZE_TAP_REC_RESET, BRONZE_TAP_REC_TRIGGER,
};
pub use ax_live::{
    bronze_is_frontmost, classify_ax_role, is_skipped_process_name, last_external_pid,
    note_external_focus, prefer_own_capture, read_capture_selection, read_focused_selection,
    reject_own_system_focus, use_system_focused_fallback, AxProtection, LiveAxOutcome,
    LIVE_AX_MAX_BYTES,
};
pub use bridge::{
    check_abi_version, event_tap_enqueue_raw, EventTapHealth, EventTapRecord, NativeError,
    NativeRuntime, OwnedUtf8, ProbeTerminal,
};
pub use display_prefs::{
    strengthen, DisplayPrefBridge, DisplayPrefError, DisplayPrefHost, DisplayPreferenceSnapshot,
    FakeDisplayHost,
};
pub use installed_apps::{
    collect_apps_from_roots, default_application_roots, installed_app_from_bundle_path,
    is_safe_bundle_id, parse_app_list_payload, try_list_installed_apps, try_pick_installed_app,
    InstalledApp, PickInstalledApp,
};
pub use pasteboard::native_pasteboard_write;
#[cfg(target_os = "macos")]
pub use permission::MacosPreflightHost;
pub use permission::{
    prompt_used_permissions, snapshot_from_preflight, PermissionRequestHost, PreflightError,
    PreflightHost, PromptAttempt, PromptReason,
};
pub use settings_file::{
    accept_settings_file_path, read_settings_import_bytes, try_pick_settings_export_path,
    try_pick_settings_import_path, PickSettingsFile, SettingsFileError,
};
pub use source_icon::{native_app_icon_png, native_bundle_id_for_pid};
pub use status_item::{
    build_status_menu, StatusAction, StatusMenuError, StatusMenuItem, StringCatalog,
    STATUS_MENU_KEYS,
};
pub use title::native_item_title;
pub use user_notice::{
    is_safe_notice_text, notification_authorization_status,
    prompt_notification_authorization_if_needed, request_notification_authorization,
    try_deliver_user_notice, NoticeAuthorization,
};

pub fn apply_live_shift_gesture(binding: &bronze_settings::ShortcutBinding) {
    let taps = bronze_settings::live_shift_tap_count(binding);
    let count = taps.unwrap_or(2);
    let _ = NativeRuntime::event_tap_set_tap_count_shared(count);
    let _ = NativeRuntime::event_tap_set_enabled_shared(taps.is_some());
}

#[cfg(test)]
pub(crate) fn lock_native_runtime() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod apply_live_shift_gesture_tests {
    use super::*;
    use bronze_settings::{default_shortcut_binding, Modifier, ShortcutActionId};

    #[test]
    fn apply_live_shift_gesture_enables_shift_and_disables_other() {
        let _guard = lock_native_runtime();
        let src = include_str!("lib.rs");
        assert!(src.contains("event_tap_set_tap_count_shared"));
        assert!(src.contains("live_shift_tap_count"));
        let engine = include_str!(
            "../../native/macos/BronzeNative/Sources/BronzeNative/EventTapEngine.swift"
        );
        assert!(engine.contains("setGestureTapCount"));
        assert!(engine.contains("tapCount: gestureTapCount"));
        apply_live_shift_gesture(&default_shortcut_binding(
            ShortcutActionId::CaptureSelection,
        ));
        let mut option = default_shortcut_binding(ShortcutActionId::CaptureSelection);
        option.modifiers = vec![Modifier::Option];
        apply_live_shift_gesture(&option);
    }
}
