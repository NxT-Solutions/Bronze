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
mod pasteboard;
mod permission;
mod source_icon;
mod status_item;
mod title;

pub use abi::{
    BronzeIngressSnapshot, BronzeNativeUtf8View, BRONZE_ABI_VERSION, BRONZE_STATUS_CANCELLED,
    BRONZE_STATUS_CONTEXT_UNAVAILABLE, BRONZE_STATUS_DEGRADED, BRONZE_STATUS_DOUBLE_COMPLETION,
    BRONZE_STATUS_INVALID_UTF8, BRONZE_STATUS_NOT_FOUND, BRONZE_STATUS_OK,
    BRONZE_STATUS_SHUTTING_DOWN, BRONZE_TAP_FEED_CANCEL, BRONZE_TAP_FEED_DOWN, BRONZE_TAP_FEED_UP,
    BRONZE_TAP_REC_DISABLED, BRONZE_TAP_REC_NONE, BRONZE_TAP_REC_RESET, BRONZE_TAP_REC_TRIGGER,
};
pub use ax_live::{
    classify_ax_role, is_skipped_process_name, last_external_pid, note_external_focus,
    read_capture_selection, read_focused_selection, reject_own_system_focus,
    use_system_focused_fallback, AxProtection, LiveAxOutcome, LIVE_AX_MAX_BYTES,
};
pub use bridge::{
    check_abi_version, event_tap_enqueue_raw, EventTapHealth, EventTapRecord, NativeError,
    NativeRuntime, OwnedUtf8, ProbeTerminal,
};
pub use display_prefs::{
    strengthen, DisplayPrefBridge, DisplayPrefError, DisplayPrefHost, DisplayPreferenceSnapshot,
    FakeDisplayHost,
};
pub use pasteboard::native_pasteboard_write;
#[cfg(target_os = "macos")]
pub use permission::MacosPreflightHost;
pub use permission::{
    prompt_used_permissions, snapshot_from_preflight, PermissionRequestHost, PreflightError,
    PreflightHost, PromptAttempt, PromptReason,
};
pub use source_icon::{native_app_icon_png, native_bundle_id_for_pid};
pub use status_item::{
    build_status_menu, StatusAction, StatusMenuError, StatusMenuItem, StringCatalog,
    STATUS_MENU_KEYS,
};
pub use title::native_item_title;

#[cfg(test)]
pub(crate) fn lock_native_runtime() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}
