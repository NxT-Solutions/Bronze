//! Versioned C ABI surface (CAP-004, SEC-002, ADR-004).
//!
//! Layout matches `native/macos/BronzeNative/Sources/BronzeNative/include/BronzeNative.h`.
//! Domain crates must not call these symbols; use [`crate::bridge`].

use std::fmt;

/// Must match `BRONZE_ABI_VERSION` in BronzeNative.h.
pub const BRONZE_ABI_VERSION: u32 = 1;

pub const BRONZE_STATUS_OK: u32 = 0;
pub const BRONZE_STATUS_INVALID_UTF8: u32 = 1;
pub const BRONZE_STATUS_DOUBLE_COMPLETION: u32 = 2;
pub const BRONZE_STATUS_CANCELLED: u32 = 3;
pub const BRONZE_STATUS_NOT_FOUND: u32 = 4;
pub const BRONZE_STATUS_SHUTTING_DOWN: u32 = 5;
pub const BRONZE_STATUS_DEGRADED: u32 = 6;
pub const BRONZE_STATUS_CONTEXT_UNAVAILABLE: u32 = 7;

pub const BRONZE_EVENT_TAP_IDLE: u32 = 0;
pub const BRONZE_EVENT_TAP_LISTENING: u32 = 1;
pub const BRONZE_EVENT_TAP_DEGRADED: u32 = 2;

pub const BRONZE_TAP_REC_NONE: u32 = 0;
pub const BRONZE_TAP_REC_TRIGGER: u32 = 1;
pub const BRONZE_TAP_REC_RESET: u32 = 2;
pub const BRONZE_TAP_REC_DISABLED: u32 = 3;

pub const BRONZE_TAP_FEED_DOWN: u32 = 1;
pub const BRONZE_TAP_FEED_UP: u32 = 2;
pub const BRONZE_TAP_FEED_CANCEL: u32 = 3;

pub const BRONZE_PASTEBOARD_KIND_NONE: u32 = 0;
pub const BRONZE_PASTEBOARD_KIND_HTML: u32 = 1;
pub const BRONZE_PASTEBOARD_KIND_RTF: u32 = 2;
pub const BRONZE_PASTEBOARD_KIND_PLAIN: u32 = 3;

/// Non-owning UTF-8 view. `ptr` may be null only when `len == 0`.
/// Length-delimited; never NUL-terminated. Embedded 0x00 is valid UTF-8.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct BronzeNativeUtf8View {
    pub ptr: *const u8,
    pub len: u64,
}

// CAP-010 / SEC-006: never print user content from a view.
impl fmt::Debug for BronzeNativeUtf8View {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BronzeNativeUtf8View")
            .field("len", &self.len)
            .finish_non_exhaustive()
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BronzeIngressSnapshot {
    pub target_pid: i32,
    pub bundle_token: u64,
    pub activation_generation: u64,
    pub destination_uuid: [u8; 16],
    pub accept_capture_generation: u64,
    pub policy_revision: u64,
    pub settings_revision: u64,
    pub context_generation: u64,
    pub route: u32,
    pub monotonic_time_ns: u64,
}

extern "C" {
    pub fn bronze_native_abi_version() -> u32;
    pub fn bronze_native_validate_utf8(view: BronzeNativeUtf8View) -> u32;
    pub fn bronze_native_test_view_len(view: BronzeNativeUtf8View) -> u64;
    pub fn bronze_native_init();
    pub fn bronze_native_shutdown();
    pub fn bronze_native_utf8_owned_copy(
        src: BronzeNativeUtf8View,
        out: *mut BronzeNativeUtf8View,
    ) -> u32;
    pub fn bronze_native_utf8_free(view: BronzeNativeUtf8View) -> u32;
    pub fn bronze_native_probe_begin(request_id: u64, view: BronzeNativeUtf8View) -> u32;
    pub fn bronze_native_probe_complete(request_id: u64) -> u32;
    pub fn bronze_native_probe_cancel(request_id: u64) -> u32;
    pub fn bronze_native_probe_outstanding() -> u64;
    pub fn bronze_native_event_tap_start() -> u32;
    pub fn bronze_native_event_tap_stop() -> u32;
    pub fn bronze_native_event_tap_health() -> u32;
    pub fn bronze_native_event_tap_set_enabled(enabled: u32) -> u32;
    pub fn bronze_native_event_tap_set_tap_count(count: u32) -> u32;
    pub fn bronze_native_event_tap_drain(kind: *mut u32, sequence: *mut u64) -> u32;
    pub fn bronze_native_event_tap_fsm_state() -> u32;
    pub fn bronze_native_event_tap_test_attach() -> u32;
    pub fn bronze_native_event_tap_test_feed(kind: u32, carbon_key: i32, time_ns: u64) -> u32;
    pub fn bronze_native_event_tap_test_disable() -> u32;
    pub fn bronze_native_event_tap_test_enqueue_from_caller(kind: u32) -> u32;
    pub fn bronze_native_frontmost_pid() -> i32;
    pub fn bronze_native_item_title(
        body: BronzeNativeUtf8View,
        out: *mut BronzeNativeUtf8View,
    ) -> u32;
    pub fn bronze_native_ingress_publish(
        target_pid: i32,
        bundle_token: u64,
        activation_generation: u64,
        destination_uuid: *const u8,
        accept_capture_generation: u64,
        policy_revision: u64,
        settings_revision: u64,
        context_generation: u64,
        route: u32,
        monotonic_time_ns: u64,
    ) -> u32;
    pub fn bronze_native_ingress_load(
        target_pid: *mut i32,
        bundle_token: *mut u64,
        activation_generation: *mut u64,
        destination_uuid: *mut u8,
        accept_capture_generation: *mut u64,
        policy_revision: *mut u64,
        settings_revision: *mut u64,
        context_generation: *mut u64,
        route: *mut u32,
        monotonic_time_ns: *mut u64,
    ) -> u32;
    pub fn bronze_native_ingress_test_begin_inconsistent() -> u32;
    pub fn bronze_native_ingress_test_end_inconsistent() -> u32;
    pub fn bronze_native_pasteboard_write(
        plain: BronzeNativeUtf8View,
        html: BronzeNativeUtf8View,
    ) -> u32;
    pub fn bronze_native_bounded_copy_read(
        target_pid: i32,
        kind: *mut u32,
        post_copy_count: *mut u64,
        payload: *mut BronzeNativeUtf8View,
        snapshot: *mut BronzeNativeUtf8View,
    ) -> u32;
    pub fn bronze_native_pasteboard_restore_if_unchanged(
        snapshot: BronzeNativeUtf8View,
        expected_count: u64,
    ) -> u32;
    pub fn bronze_native_bundle_id_for_pid(pid: i32, out: *mut BronzeNativeUtf8View) -> u32;
    pub fn bronze_native_app_icon_png(
        bundle_or_name: BronzeNativeUtf8View,
        out: *mut BronzeNativeUtf8View,
    ) -> u32;
    pub fn bronze_native_list_installed_apps(out: *mut BronzeNativeUtf8View) -> u32;
    pub fn bronze_native_pick_installed_app(out: *mut BronzeNativeUtf8View) -> u32;
    pub fn bronze_native_pick_settings_export_path(out: *mut BronzeNativeUtf8View) -> u32;
    pub fn bronze_native_pick_settings_import_path(out: *mut BronzeNativeUtf8View) -> u32;
    pub fn bronze_native_pick_support_export_path(out: *mut BronzeNativeUtf8View) -> u32;
    pub fn bronze_native_deliver_user_notice(
        title: BronzeNativeUtf8View,
        body: BronzeNativeUtf8View,
    ) -> u32;
    pub fn bronze_native_notification_authorization_status() -> u32;
    pub fn bronze_native_request_notification_authorization() -> u32;
    pub fn bronze_native_login_item_status() -> u32;
    pub fn bronze_native_apply_login_item(enabled: u32) -> u32;
}
