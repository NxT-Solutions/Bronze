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
    BronzeNativeUtf8View, BRONZE_ABI_VERSION, BRONZE_STATUS_DOUBLE_COMPLETION,
    BRONZE_STATUS_INVALID_UTF8, BRONZE_STATUS_OK,
};
pub use bridge::{check_abi_version, NativeError, NativeRuntime};
#[cfg(target_os = "macos")]
pub use permission::MacosPreflightHost;
pub use permission::{snapshot_from_preflight, PreflightError, PreflightHost};
