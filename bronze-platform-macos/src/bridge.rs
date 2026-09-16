//! Safe Rust façade over the versioned C ABI (CAP-004, ADR-004).
//!
//! Domain crates use this module only. They must not import Swift, AppKit,
//! or the raw `extern "C"` symbols. Panic at the FFI boundary is contained.

use crate::abi::{
    self, BronzeNativeUtf8View, BRONZE_ABI_VERSION, BRONZE_STATUS_INVALID_UTF8, BRONZE_STATUS_OK,
};
use std::fmt;
use std::panic::{self, AssertUnwindSafe};

/// Fail-closed native-bridge errors. Unknown ABI versions never proceed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeError {
    VersionMismatch { expected: u32, actual: u32 },
    FfiPanic,
    InvalidUtf8,
    AlreadyActive,
}

impl fmt::Display for NativeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::VersionMismatch { expected, actual } => write!(
                f,
                "ABI version mismatch: expected {expected}, got {actual} (CAP-004 fail-closed)"
            ),
            Self::FfiPanic => write!(f, "panic contained at FFI boundary (ADR-004)"),
            Self::InvalidUtf8 => write!(f, "invalid UTF-8 in length-delimited view"),
            Self::AlreadyActive => write!(f, "native runtime already initialized"),
        }
    }
}

impl std::error::Error for NativeError {}

/// Compare a reported ABI version against [`BRONZE_ABI_VERSION`].
///
/// Mismatch is fail-closed: no further ABI use.
pub fn check_abi_version(actual: u32) -> Result<(), NativeError> {
    if actual == BRONZE_ABI_VERSION {
        Ok(())
    } else {
        Err(NativeError::VersionMismatch {
            expected: BRONZE_ABI_VERSION,
            actual,
        })
    }
}

fn catch_ffi<T>(f: impl FnOnce() -> T) -> Result<T, NativeError> {
    panic::catch_unwind(AssertUnwindSafe(f)).map_err(|_| NativeError::FfiPanic)
}

/// Process-local native runtime. One active instance; init/shutdown pair.
pub struct NativeRuntime {
    _not_send_sync: *const (),
}

// The C ABI is process-global; do not share the guard across threads.
unsafe impl Send for NativeRuntime {}

static ACTIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

impl NativeRuntime {
    /// Initialize the ABI and fail closed if the version does not match.
    pub fn start() -> Result<Self, NativeError> {
        if ACTIVE.swap(true, std::sync::atomic::Ordering::SeqCst) {
            return Err(NativeError::AlreadyActive);
        }

        if let Err(err) = catch_ffi(|| unsafe { abi::bronze_native_init() }) {
            ACTIVE.store(false, std::sync::atomic::Ordering::SeqCst);
            return Err(err);
        }

        let version = match catch_ffi(|| unsafe { abi::bronze_native_abi_version() }) {
            Ok(version) => version,
            Err(err) => {
                let _ = Self::force_shutdown();
                return Err(err);
            }
        };

        if let Err(err) = check_abi_version(version) {
            let _ = Self::force_shutdown();
            return Err(err);
        }

        Ok(Self {
            _not_send_sync: std::ptr::null(),
        })
    }

    pub fn abi_version(&self) -> Result<u32, NativeError> {
        catch_ffi(|| unsafe { abi::bronze_native_abi_version() })
    }

    /// Length-delimited UTF-8 check. Never uses CString/strlen.
    pub fn validate_utf8(&self, bytes: &[u8]) -> Result<(), NativeError> {
        let view = BronzeNativeUtf8View {
            ptr: bytes.as_ptr(),
            len: bytes.len() as u64,
        };
        self.validate_view(view)
    }

    pub fn validate_view(&self, view: BronzeNativeUtf8View) -> Result<(), NativeError> {
        let status = catch_ffi(|| unsafe { abi::bronze_native_validate_utf8(view) })?;
        match status {
            BRONZE_STATUS_OK => Ok(()),
            BRONZE_STATUS_INVALID_UTF8 => Err(NativeError::InvalidUtf8),
            _ => Err(NativeError::InvalidUtf8),
        }
    }

    pub fn test_view_len(&self, view: BronzeNativeUtf8View) -> Result<u64, NativeError> {
        catch_ffi(|| unsafe { abi::bronze_native_test_view_len(view) })
    }

    pub fn shutdown(self) -> Result<(), NativeError> {
        Self::force_shutdown()
    }

    fn force_shutdown() -> Result<(), NativeError> {
        let result = catch_ffi(|| unsafe { abi::bronze_native_shutdown() });
        ACTIVE.store(false, std::sync::atomic::Ordering::SeqCst);
        result
    }
}

impl Drop for NativeRuntime {
    fn drop(&mut self) {
        if ACTIVE.swap(false, std::sync::atomic::Ordering::SeqCst) {
            let _ = catch_ffi(|| unsafe { abi::bronze_native_shutdown() });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{check_abi_version, NativeError, NativeRuntime, BRONZE_ABI_VERSION};
    use crate::abi::BronzeNativeUtf8View;
    use std::sync::{Mutex, MutexGuard};

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn lock_runtime() -> MutexGuard<'static, ()> {
        TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[test]
    fn version_query_matches_header() {
        let _guard = lock_runtime();
        let runtime = NativeRuntime::start().expect("start");
        assert_eq!(runtime.abi_version().expect("version"), BRONZE_ABI_VERSION);
        runtime.shutdown().expect("shutdown");
    }

    #[test]
    fn init_shutdown_round_trip() {
        let _guard = lock_runtime();
        let runtime = NativeRuntime::start().expect("start");
        runtime.shutdown().expect("shutdown");
        let runtime = NativeRuntime::start().expect("restart after shutdown");
        runtime.shutdown().expect("second shutdown");
    }

    #[test]
    fn double_start_is_rejected() {
        let _guard = lock_runtime();
        let first = NativeRuntime::start().expect("start");
        assert_eq!(
            NativeRuntime::start()
                .err()
                .expect("second start must fail"),
            NativeError::AlreadyActive
        );
        first.shutdown().expect("shutdown");
    }

    #[test]
    fn version_mismatch_fails_closed() {
        assert_eq!(
            check_abi_version(999),
            Err(NativeError::VersionMismatch {
                expected: BRONZE_ABI_VERSION,
                actual: 999,
            })
        );
        assert!(check_abi_version(BRONZE_ABI_VERSION).is_ok());
    }

    #[test]
    fn utf8_happy_embedded_nul_and_invalid() {
        let _guard = lock_runtime();
        let runtime = NativeRuntime::start().expect("start");
        runtime.validate_utf8(b"hello").expect("valid");
        runtime
            .validate_utf8(b"a\0b")
            .expect("embedded NUL is valid UTF-8");
        runtime.validate_utf8(b"").expect("empty");
        assert_eq!(
            runtime.validate_utf8(&[0xff]).unwrap_err(),
            NativeError::InvalidUtf8
        );
        assert_eq!(
            runtime.validate_utf8(&[0xc3]).unwrap_err(),
            NativeError::InvalidUtf8
        );
        runtime.shutdown().expect("shutdown");
    }

    #[test]
    fn view_debug_omits_content() {
        let secret = b"hunter2-not-for-logs";
        let view = BronzeNativeUtf8View {
            ptr: secret.as_ptr(),
            len: secret.len() as u64,
        };
        let rendered = format!("{view:?}");
        assert!(rendered.contains("len"));
        assert!(
            !rendered.contains("hunter2"),
            "Debug must not include view bytes (SEC-006/CAP-010): {rendered}"
        );
    }

    #[test]
    fn nil_ptr_nonzero_len_is_invalid() {
        let _guard = lock_runtime();
        let runtime = NativeRuntime::start().expect("start");
        let view = BronzeNativeUtf8View {
            ptr: std::ptr::null(),
            len: 3,
        };
        assert_eq!(
            runtime.validate_view(view).unwrap_err(),
            NativeError::InvalidUtf8
        );
        runtime.shutdown().expect("shutdown");
    }

    #[test]
    fn test_view_len_is_length_delimited() {
        let _guard = lock_runtime();
        let runtime = NativeRuntime::start().expect("start");
        let bytes = [0x61, 0x00, 0x62];
        let view = BronzeNativeUtf8View {
            ptr: bytes.as_ptr(),
            len: 3,
        };
        assert_eq!(runtime.test_view_len(view).expect("len"), 3);
        runtime.shutdown().expect("shutdown");
    }
}
