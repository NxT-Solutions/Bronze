//! Safe macOS façade over the versioned C ABI (CAP-004, ADR-004).
//!
//! bronze-domain and other core crates must not import macOS frameworks or
//! call Swift. This crate is the only Rust entry to BronzeNative.

mod abi;
#[cfg(feature = "abi-stub")]
mod abi_stub;
mod bridge;
mod permission;

pub use abi::{
    BronzeIngressSnapshot, BronzeNativeUtf8View, BRONZE_ABI_VERSION, BRONZE_STATUS_CANCELLED,
    BRONZE_STATUS_CONTEXT_UNAVAILABLE, BRONZE_STATUS_DEGRADED, BRONZE_STATUS_DOUBLE_COMPLETION,
    BRONZE_STATUS_INVALID_UTF8, BRONZE_STATUS_NOT_FOUND, BRONZE_STATUS_OK,
    BRONZE_STATUS_SHUTTING_DOWN, BRONZE_TAP_FEED_CANCEL, BRONZE_TAP_FEED_DOWN, BRONZE_TAP_FEED_UP,
    BRONZE_TAP_REC_DISABLED, BRONZE_TAP_REC_NONE, BRONZE_TAP_REC_RESET, BRONZE_TAP_REC_TRIGGER,
};
pub use bridge::{
    check_abi_version, event_tap_enqueue_raw, EventTapHealth, EventTapRecord, NativeError,
    NativeRuntime, OwnedUtf8, ProbeTerminal,
};
#[cfg(target_os = "macos")]
pub use permission::MacosPreflightHost;
pub use permission::{snapshot_from_preflight, PreflightError, PreflightHost};
