//! bronze-platform-macos scaffold (macOS façade slot; no AX content query)

mod permission;
#[cfg(target_os = "macos")]
pub use permission::MacosPreflightHost;
pub use permission::{snapshot_from_preflight, PreflightError, PreflightHost};
