//! Safe Rust façade over the versioned C ABI (CAP-004, ADR-004).
//!
//! Domain crates use this module only. They must not import Swift, AppKit,
//! or the raw `extern "C"` symbols. Panic at the FFI boundary is contained.

use crate::abi::{
    self, BronzeNativeUtf8View, BRONZE_ABI_VERSION, BRONZE_EVENT_TAP_DEGRADED,
    BRONZE_EVENT_TAP_IDLE, BRONZE_EVENT_TAP_LISTENING, BRONZE_STATUS_CANCELLED,
    BRONZE_STATUS_CONTEXT_UNAVAILABLE, BRONZE_STATUS_DEGRADED, BRONZE_STATUS_DOUBLE_COMPLETION,
    BRONZE_STATUS_INVALID_UTF8, BRONZE_STATUS_NOT_FOUND, BRONZE_STATUS_OK,
    BRONZE_STATUS_SHUTTING_DOWN,
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
    Cancelled,
    DoubleCompletion,
    NotFound,
    ShuttingDown,
    Degraded,
    ContextUnavailable,
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
            Self::Cancelled => write!(f, "probe cancelled"),
            Self::DoubleCompletion => write!(f, "double completion forbidden (CAP-004)"),
            Self::NotFound => write!(f, "probe id not found"),
            Self::ShuttingDown => write!(f, "native runtime is shutting down"),
            Self::Degraded => write!(
                f,
                "event tap degraded; chord/menu remain (CAP-002, no event suppression)"
            ),
            Self::ContextUnavailable => write!(f, "ingress context_unavailable (CAP-004)"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventTapHealth {
    Idle,
    Listening,
    Degraded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EventTapRecord {
    pub kind: u32,
    pub sequence: u64,
}

/// Terminal for a one-shot probe. Cancel is a valid outcome, not a drop.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProbeTerminal {
    Completed,
    Cancelled,
}

/// Native-owned UTF-8 copy. Debug prints length only (SEC-006).
pub struct OwnedUtf8 {
    view: BronzeNativeUtf8View,
    freed: bool,
}

impl fmt::Debug for OwnedUtf8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OwnedUtf8")
            .field("len", &self.view.len)
            .finish_non_exhaustive()
    }
}

impl OwnedUtf8 {
    pub fn len(&self) -> u64 {
        self.view.len
    }

    pub fn is_empty(&self) -> bool {
        self.view.len == 0
    }

    pub fn free(mut self) -> Result<(), NativeError> {
        self.release()
    }

    fn release(&mut self) -> Result<(), NativeError> {
        if self.freed {
            return Err(NativeError::DoubleCompletion);
        }
        let status = catch_ffi(|| unsafe { abi::bronze_native_utf8_free(self.view) })?;
        self.freed = true;
        self.view.ptr = std::ptr::null();
        self.view.len = 0;
        map_status(status)
    }
}

impl Drop for OwnedUtf8 {
    fn drop(&mut self) {
        if !self.freed {
            let _ = self.release();
        }
    }
}

fn map_status(status: u32) -> Result<(), NativeError> {
    match status {
        BRONZE_STATUS_OK => Ok(()),
        BRONZE_STATUS_INVALID_UTF8 => Err(NativeError::InvalidUtf8),
        BRONZE_STATUS_DOUBLE_COMPLETION => Err(NativeError::DoubleCompletion),
        BRONZE_STATUS_CANCELLED => Err(NativeError::Cancelled),
        BRONZE_STATUS_NOT_FOUND => Err(NativeError::NotFound),
        BRONZE_STATUS_SHUTTING_DOWN => Err(NativeError::ShuttingDown),
        BRONZE_STATUS_DEGRADED => Err(NativeError::Degraded),
        BRONZE_STATUS_CONTEXT_UNAVAILABLE => Err(NativeError::ContextUnavailable),
        _ => Err(NativeError::InvalidUtf8),
    }
}

fn map_complete_status(status: u32) -> Result<ProbeTerminal, NativeError> {
    match status {
        BRONZE_STATUS_OK => Ok(ProbeTerminal::Completed),
        BRONZE_STATUS_CANCELLED => Ok(ProbeTerminal::Cancelled),
        BRONZE_STATUS_INVALID_UTF8 => Err(NativeError::InvalidUtf8),
        BRONZE_STATUS_DOUBLE_COMPLETION => Err(NativeError::DoubleCompletion),
        BRONZE_STATUS_NOT_FOUND => Err(NativeError::NotFound),
        BRONZE_STATUS_SHUTTING_DOWN => Err(NativeError::ShuttingDown),
        _ => Err(NativeError::InvalidUtf8),
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
        map_status(status)
    }

    pub fn test_view_len(&self, view: BronzeNativeUtf8View) -> Result<u64, NativeError> {
        catch_ffi(|| unsafe { abi::bronze_native_test_view_len(view) })
    }

    pub fn copy_utf8(&self, bytes: &[u8]) -> Result<OwnedUtf8, NativeError> {
        let src = BronzeNativeUtf8View {
            ptr: bytes.as_ptr(),
            len: bytes.len() as u64,
        };
        let mut out = BronzeNativeUtf8View {
            ptr: std::ptr::null(),
            len: 0,
        };
        let status = catch_ffi(|| unsafe { abi::bronze_native_utf8_owned_copy(src, &mut out) })?;
        map_status(status)?;
        Ok(OwnedUtf8 {
            view: out,
            freed: false,
        })
    }

    pub fn probe_begin(&self, request_id: u64, bytes: &[u8]) -> Result<(), NativeError> {
        Self::probe_begin_id(request_id, bytes)
    }

    pub fn probe_complete(&self, request_id: u64) -> Result<ProbeTerminal, NativeError> {
        Self::probe_complete_id(request_id)
    }

    pub fn probe_cancel(&self, request_id: u64) -> Result<ProbeTerminal, NativeError> {
        Self::probe_cancel_id(request_id)
    }

    pub fn probe_outstanding(&self) -> Result<u64, NativeError> {
        Self::probe_outstanding_id()
    }

    /// Process-global probe entry for shutdown-race tests (C ABI is not Sync).
    pub fn probe_begin_id(request_id: u64, bytes: &[u8]) -> Result<(), NativeError> {
        let view = BronzeNativeUtf8View {
            ptr: bytes.as_ptr(),
            len: bytes.len() as u64,
        };
        let status = catch_ffi(|| unsafe { abi::bronze_native_probe_begin(request_id, view) })?;
        map_status(status)
    }

    pub fn probe_complete_id(request_id: u64) -> Result<ProbeTerminal, NativeError> {
        let status = catch_ffi(|| unsafe { abi::bronze_native_probe_complete(request_id) })?;
        map_complete_status(status)
    }

    pub fn probe_cancel_id(request_id: u64) -> Result<ProbeTerminal, NativeError> {
        let status = catch_ffi(|| unsafe { abi::bronze_native_probe_cancel(request_id) })?;
        map_complete_status(status)
    }

    pub fn probe_outstanding_id() -> Result<u64, NativeError> {
        catch_ffi(|| unsafe { abi::bronze_native_probe_outstanding() })
    }

    pub fn event_tap_start(&self) -> Result<EventTapHealth, NativeError> {
        Self::event_tap_start_shared()
    }

    pub fn event_tap_start_shared() -> Result<EventTapHealth, NativeError> {
        let status = catch_ffi(|| unsafe { abi::bronze_native_event_tap_start() })?;
        match status {
            BRONZE_STATUS_OK => Self::event_tap_health_shared(),
            BRONZE_STATUS_DEGRADED => Ok(EventTapHealth::Degraded),
            other => map_status(other).map(|()| EventTapHealth::Idle),
        }
    }

    pub fn event_tap_stop(&self) -> Result<(), NativeError> {
        let status = catch_ffi(|| unsafe { abi::bronze_native_event_tap_stop() })?;
        map_status(status)
    }

    pub fn event_tap_health(&self) -> Result<EventTapHealth, NativeError> {
        Self::event_tap_health_shared()
    }

    pub fn event_tap_health_shared() -> Result<EventTapHealth, NativeError> {
        let raw = catch_ffi(|| unsafe { abi::bronze_native_event_tap_health() })?;
        match raw {
            BRONZE_EVENT_TAP_IDLE => Ok(EventTapHealth::Idle),
            BRONZE_EVENT_TAP_LISTENING => Ok(EventTapHealth::Listening),
            BRONZE_EVENT_TAP_DEGRADED => Ok(EventTapHealth::Degraded),
            _ => Err(NativeError::NotFound),
        }
    }

    pub fn event_tap_set_enabled(&self, enabled: bool) -> Result<(), NativeError> {
        Self::event_tap_set_enabled_shared(enabled)
    }

    pub fn event_tap_set_enabled_shared(enabled: bool) -> Result<(), NativeError> {
        let status =
            catch_ffi(|| unsafe { abi::bronze_native_event_tap_set_enabled(u32::from(enabled)) })?;
        map_status(status)
    }

    pub fn event_tap_drain(&self) -> Result<Option<EventTapRecord>, NativeError> {
        Self::event_tap_drain_shared()
    }

    pub fn event_tap_drain_shared() -> Result<Option<EventTapRecord>, NativeError> {
        let mut kind = 0u32;
        let mut sequence = 0u64;
        let status =
            catch_ffi(|| unsafe { abi::bronze_native_event_tap_drain(&mut kind, &mut sequence) })?;
        map_status(status)?;
        if kind == 0 {
            Ok(None)
        } else {
            Ok(Some(EventTapRecord { kind, sequence }))
        }
    }

    pub fn event_tap_fsm_state(&self) -> Result<u32, NativeError> {
        catch_ffi(|| unsafe { abi::bronze_native_event_tap_fsm_state() })
    }

    pub fn event_tap_test_attach(&self) -> Result<(), NativeError> {
        let status = catch_ffi(|| unsafe { abi::bronze_native_event_tap_test_attach() })?;
        map_status(status)
    }

    pub fn event_tap_test_feed(
        &self,
        kind: u32,
        carbon_key: i32,
        time_ns: u64,
    ) -> Result<(), NativeError> {
        let status = catch_ffi(|| unsafe {
            abi::bronze_native_event_tap_test_feed(kind, carbon_key, time_ns)
        })?;
        map_status(status)
    }

    pub fn event_tap_test_disable(&self) -> Result<(), NativeError> {
        let status = catch_ffi(|| unsafe { abi::bronze_native_event_tap_test_disable() })?;
        map_status(status)
    }

    pub fn event_tap_test_enqueue_from_caller(&self, kind: u32) -> Result<(), NativeError> {
        map_status(event_tap_enqueue_raw(kind))
    }

    pub fn ingress_publish(
        &self,
        snap: crate::abi::BronzeIngressSnapshot,
    ) -> Result<(), NativeError> {
        let status = catch_ffi(|| unsafe {
            abi::bronze_native_ingress_publish(
                snap.target_pid,
                snap.bundle_token,
                snap.activation_generation,
                snap.destination_uuid.as_ptr(),
                snap.accept_capture_generation,
                snap.policy_revision,
                snap.settings_revision,
                snap.context_generation,
                snap.route,
                snap.monotonic_time_ns,
            )
        })?;
        map_status(status)
    }

    pub fn ingress_load(&self) -> Result<crate::abi::BronzeIngressSnapshot, NativeError> {
        let mut snap = crate::abi::BronzeIngressSnapshot {
            target_pid: 0,
            bundle_token: 0,
            activation_generation: 0,
            destination_uuid: [0; 16],
            accept_capture_generation: 0,
            policy_revision: 0,
            settings_revision: 0,
            context_generation: 0,
            route: 0,
            monotonic_time_ns: 0,
        };
        let status = catch_ffi(|| unsafe {
            abi::bronze_native_ingress_load(
                &mut snap.target_pid,
                &mut snap.bundle_token,
                &mut snap.activation_generation,
                snap.destination_uuid.as_mut_ptr(),
                &mut snap.accept_capture_generation,
                &mut snap.policy_revision,
                &mut snap.settings_revision,
                &mut snap.context_generation,
                &mut snap.route,
                &mut snap.monotonic_time_ns,
            )
        })?;
        map_status(status)?;
        Ok(snap)
    }

    pub fn ingress_test_begin_inconsistent(&self) -> Result<(), NativeError> {
        let status = catch_ffi(|| unsafe { abi::bronze_native_ingress_test_begin_inconsistent() })?;
        map_status(status)
    }

    pub fn ingress_test_end_inconsistent(&self) -> Result<(), NativeError> {
        let status = catch_ffi(|| unsafe { abi::bronze_native_ingress_test_end_inconsistent() })?;
        map_status(status)
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

/// Raw SPSC write for single-producer tests. Must not be used as a second product route.
pub fn event_tap_enqueue_raw(kind: u32) -> u32 {
    match catch_ffi(|| unsafe { abi::bronze_native_event_tap_test_enqueue_from_caller(kind) }) {
        Ok(status) => status,
        Err(_) => crate::abi::BRONZE_STATUS_NOT_FOUND,
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
    use crate::lock_native_runtime as lock_runtime;

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

    #[test]
    fn owned_copy_empty_nul_invalid_large_and_double_free() {
        let _guard = lock_runtime();
        let runtime = NativeRuntime::start().expect("start");
        let empty = runtime.copy_utf8(b"").expect("empty");
        assert!(empty.is_empty());
        empty.free().expect("free empty");

        let nul = runtime.copy_utf8(b"a\0b").expect("nul");
        assert_eq!(nul.len(), 3);
        let rendered = format!("{nul:?}");
        assert!(rendered.contains("len"));
        assert!(
            !rendered.contains('\0'),
            "OwnedUtf8 debug must not include bytes (SEC-006): {rendered}"
        );
        nul.free().expect("free nul");

        assert_eq!(
            runtime.copy_utf8(&[0xff]).unwrap_err(),
            NativeError::InvalidUtf8
        );

        let large = vec![b'x'; 1024 * 1024];
        let owned = runtime.copy_utf8(&large).expect("large");
        assert_eq!(owned.len(), large.len() as u64);
        owned.free().expect("free large");
        runtime.shutdown().expect("shutdown");
    }

    #[test]
    fn probe_completes_once_or_cancels() {
        let _guard = lock_runtime();
        let runtime = NativeRuntime::start().expect("start");
        runtime.probe_begin(1, b"probe").expect("begin");
        assert_eq!(runtime.probe_outstanding().expect("count"), 1);
        assert_eq!(
            runtime.probe_complete(1).expect("complete"),
            super::ProbeTerminal::Completed
        );
        assert_eq!(runtime.probe_outstanding().expect("count"), 0);
        assert_eq!(
            runtime.probe_complete(1).unwrap_err(),
            NativeError::DoubleCompletion
        );
        runtime.probe_begin(2, b"cancel").expect("begin cancel");
        assert_eq!(
            runtime.probe_cancel(2).expect("cancel"),
            super::ProbeTerminal::Cancelled
        );
        assert_eq!(
            runtime.probe_complete(2).unwrap_err(),
            NativeError::DoubleCompletion
        );
        assert_eq!(
            runtime.probe_complete(99).unwrap_err(),
            NativeError::NotFound
        );
        runtime.shutdown().expect("shutdown");
    }

    #[test]
    fn shutdown_cancels_open_probe() {
        let _guard = lock_runtime();
        let runtime = NativeRuntime::start().expect("start");
        runtime.probe_begin(3, b"open").expect("begin");
        runtime.shutdown().expect("shutdown");
        assert_eq!(
            super::NativeRuntime::probe_complete_id(3).unwrap_err(),
            NativeError::DoubleCompletion
        );
        assert_eq!(
            super::NativeRuntime::probe_begin_id(4, b"").unwrap_err(),
            NativeError::ShuttingDown
        );
        let runtime = NativeRuntime::start().expect("reinit");
        runtime.probe_begin(4, b"").expect("empty after reinit");
        runtime.probe_complete(4).expect("complete empty");
        runtime.shutdown().expect("shutdown");
    }

    #[test]
    fn shutdown_race_completes_once_or_cancels() {
        let _guard = lock_runtime();
        let runtime = NativeRuntime::start().expect("start");
        runtime.probe_begin(7, b"race").expect("begin");
        let handle = std::thread::spawn(|| super::NativeRuntime::probe_complete_id(7));
        let _ = runtime.shutdown();
        let complete = handle.join().expect("join");
        let completed_ok = complete == Ok(super::ProbeTerminal::Completed);
        let cancelled_or_double = matches!(
            complete,
            Ok(super::ProbeTerminal::Cancelled)
                | Err(NativeError::DoubleCompletion)
                | Err(NativeError::NotFound)
                | Err(NativeError::Cancelled)
        );
        assert!(
            completed_ok || cancelled_or_double,
            "shutdown race must complete once or cancel: {complete:?}"
        );
        let second = super::NativeRuntime::probe_complete_id(7);
        assert_ne!(
            second,
            Ok(super::ProbeTerminal::Completed),
            "second complete after race must not succeed"
        );
    }

    #[test]
    fn event_tap_callback_sources_have_no_forbidden_work() {
        let engine = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../native/macos/BronzeNative/Sources/BronzeNative/EventTapEngine.swift"
        ));
        let frontmost = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../native/macos/BronzeNative/Sources/BronzeNative/FrontmostABI.swift"
        ));
        assert!(frontmost.contains("NSWorkspace"));
        assert!(frontmost.contains("bronze_native_frontmost_pid"));
        for needle in [
            "AXUIElement",
            "NSPasteboard",
            "NSWindow",
            "NSWorkspace",
            "sqlite",
            "NSLog",
            "os_log",
            "print(",
            "Logger(",
            "FileManager",
        ] {
            assert!(
                !engine.contains(needle),
                "event-tap callback file must not contain {needle} (CAP-002/CAP-004)"
            );
        }
    }

    #[test]
    fn event_tap_spsc_single_producer_and_disable_resets_fsm() {
        let _guard = lock_runtime();
        let runtime = NativeRuntime::start().expect("start");
        let health = runtime.event_tap_start().expect("start tap");
        assert!(
            health == super::EventTapHealth::Degraded || health == super::EventTapHealth::Listening
        );
        runtime.event_tap_stop().expect("stop");
        runtime.event_tap_test_attach().expect("attach");
        runtime.event_tap_set_enabled(true).expect("enable");
        runtime
            .event_tap_test_feed(crate::abi::BRONZE_TAP_FEED_DOWN, 56, 1_000)
            .expect("down");
        assert_eq!(runtime.event_tap_fsm_state().expect("state"), 1);
        runtime.event_tap_test_disable().expect("disable");
        assert_eq!(runtime.event_tap_fsm_state().expect("idle"), 0);
        let rec = runtime.event_tap_drain().expect("drain").expect("disabled");
        assert_eq!(rec.kind, crate::abi::BRONZE_TAP_REC_DISABLED);
        runtime
            .event_tap_test_enqueue_from_caller(crate::abi::BRONZE_TAP_REC_RESET)
            .expect("producer enqueue");
        let foreign = std::thread::spawn(|| {
            let status = unsafe { crate::abi::bronze_native_event_tap_test_enqueue_from_caller(1) };
            status
        });
        assert_eq!(
            foreign.join().expect("join"),
            crate::abi::BRONZE_STATUS_NOT_FOUND
        );
        runtime.event_tap_stop().expect("stop");
        runtime.shutdown().expect("shutdown");
    }

    #[test]
    fn ingress_native_slot_unavailable_on_inconsistent_read() {
        let _guard = lock_runtime();
        let runtime = NativeRuntime::start().expect("start");
        let snap = crate::abi::BronzeIngressSnapshot {
            target_pid: 9,
            bundle_token: 1,
            activation_generation: 1,
            destination_uuid: [3; 16],
            accept_capture_generation: 1,
            policy_revision: 1,
            settings_revision: 1,
            context_generation: 1,
            route: 2,
            monotonic_time_ns: 4,
        };
        runtime.ingress_publish(snap).expect("publish");
        assert_eq!(runtime.ingress_load().expect("load").target_pid, 9);
        runtime
            .ingress_test_begin_inconsistent()
            .expect("begin torn");
        assert_eq!(
            runtime.ingress_load().unwrap_err(),
            NativeError::ContextUnavailable
        );
        runtime.ingress_test_end_inconsistent().expect("end torn");
        runtime.shutdown().expect("shutdown");
    }
}
