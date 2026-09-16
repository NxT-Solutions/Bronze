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
    pub fn bronze_native_event_tap_drain(kind: *mut u32, sequence: *mut u64) -> u32;
    pub fn bronze_native_event_tap_fsm_state() -> u32;
    pub fn bronze_native_event_tap_test_attach() -> u32;
    pub fn bronze_native_event_tap_test_feed(kind: u32, carbon_key: i32, time_ns: u64) -> u32;
    pub fn bronze_native_event_tap_test_disable() -> u32;
    pub fn bronze_native_event_tap_test_enqueue_from_caller(kind: u32) -> u32;
}
