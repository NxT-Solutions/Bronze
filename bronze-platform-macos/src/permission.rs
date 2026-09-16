//! Input Monitoring and Accessibility preflight plus request wrap (SET-003, SET-004, CAP-010, ADR-005).
//!
//! Business mapping lives in `bronze-settings`. Live macOS calls stay behind
//! [`PreflightHost`] / [`PermissionRequestHost`]. Tests use fakes only.
//! Screen Recording is never requested.

use bronze_settings::{PermissionSnapshot, PermissionState};

/// Closed probe error. Unrecognized results are [`Self::Unknown`]; known failures
/// that are not a grant bit are [`Self::Fail`]. Neither maps to healthy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreflightError {
    Unknown,
    Fail,
}

/// Why a used-permission prompt is being attempted (SET-004).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromptReason {
    NativeStart,
    FirstCapture,
    HealthRetest,
}

/// Result of requesting only the permissions Bronze actually uses.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PromptAttempt {
    pub snapshot: PermissionSnapshot,
    pub listen_requested: bool,
    pub accessibility_requested: bool,
    pub screen_recording_requested: bool,
}

/// Host that reports Input Monitoring and Accessibility preflight bits.
pub trait PreflightHost {
    fn listen_event_access(&self) -> Result<bool, PreflightError>;
    fn accessibility_trusted(&self) -> Result<bool, PreflightError>;
}

/// Host that may show the OS prompt for used permissions.
///
/// Implementations must not request Screen Recording.
pub trait PermissionRequestHost {
    fn request_listen_event_access(&self) -> Result<bool, PreflightError>;
    fn request_accessibility_trusted(&self) -> Result<bool, PreflightError>;
}

/// Map independent preflight bits through settings mappers (SET-003).
///
/// Self-test stays [`PermissionState::Unknown`] until a caller supplies an explicit
/// operation-level result. Preflight never emits `healthy`.
pub fn snapshot_from_preflight<H: PreflightHost + ?Sized>(host: &H) -> PermissionSnapshot {
    PermissionSnapshot {
        input_monitoring: map_probe(host.listen_event_access()),
        accessibility: map_probe(host.accessibility_trusted()),
        capture_pipeline_self_test: PermissionState::from_platform_unknown(),
    }
}

fn map_probe(result: Result<bool, PreflightError>) -> PermissionState {
    match result {
        Ok(granted) => PermissionState::from_preflight_granted(granted),
        Err(PreflightError::Unknown) => PermissionState::from_platform_unknown(),
        Err(PreflightError::Fail) => PermissionState::from_platform_probe_failure(),
    }
}

fn map_request(result: Result<bool, PreflightError>) -> PermissionState {
    match result {
        Ok(granted) => PermissionState::from_request_granted(granted),
        Err(PreflightError::Unknown) => PermissionState::from_platform_unknown(),
        Err(PreflightError::Fail) => PermissionState::from_platform_probe_failure(),
    }
}

fn snapshot_from_request(
    listen: Result<bool, PreflightError>,
    accessibility: Result<bool, PreflightError>,
) -> PermissionSnapshot {
    PermissionSnapshot {
        input_monitoring: map_request(listen),
        accessibility: map_request(accessibility),
        capture_pipeline_self_test: PermissionState::from_platform_unknown(),
    }
}

/// Request Accessibility and Input Monitoring. Never Screen Recording.
///
/// Native start and first capture request only when preflight is not already
/// granted. Health retest always attempts the OS request APIs (the OS may
/// refuse to show a second dialog). Launch is not a prompt loop.
pub fn prompt_used_permissions<H>(host: &H, reason: PromptReason) -> PromptAttempt
where
    H: PreflightHost + PermissionRequestHost + ?Sized,
{
    match reason {
        PromptReason::HealthRetest => {
            let listen = host.request_listen_event_access();
            let accessibility = host.request_accessibility_trusted();
            PromptAttempt {
                snapshot: snapshot_from_request(listen, accessibility),
                listen_requested: true,
                accessibility_requested: true,
                screen_recording_requested: false,
            }
        }
        PromptReason::NativeStart | PromptReason::FirstCapture => {
            let (listen, listen_requested) = match host.listen_event_access() {
                Ok(true) => (Ok(true), false),
                _ => (host.request_listen_event_access(), true),
            };
            let (accessibility, accessibility_requested) = match host.accessibility_trusted() {
                Ok(true) => (Ok(true), false),
                _ => (host.request_accessibility_trusted(), true),
            };
            let snapshot = if listen_requested || accessibility_requested {
                snapshot_from_request(listen, accessibility)
            } else {
                PermissionSnapshot {
                    input_monitoring: PermissionState::from_preflight_granted(true),
                    accessibility: PermissionState::from_preflight_granted(true),
                    capture_pipeline_self_test: PermissionState::from_platform_unknown(),
                }
            };
            PromptAttempt {
                snapshot,
                listen_requested,
                accessibility_requested,
                screen_recording_requested: false,
            }
        }
    }
}

/// Live macOS host. Unit tests must not construct or call this type.
#[cfg(target_os = "macos")]
pub struct MacosPreflightHost;

#[cfg(target_os = "macos")]
mod sys {
    use std::ffi::c_void;

    pub type CFTypeRef = *const c_void;
    pub type CFDictionaryRef = *const c_void;
    pub type CFIndex = isize;

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        pub(super) fn CGPreflightListenEventAccess() -> u8;
        pub(super) fn CGRequestListenEventAccess() -> u8;
    }

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        pub(super) fn AXIsProcessTrusted() -> u8;
        pub(super) fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> u8;
        pub(super) static kAXTrustedCheckOptionPrompt: CFTypeRef;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        pub(super) static kCFBooleanTrue: CFTypeRef;
        pub(super) static kCFTypeDictionaryKeyCallBacks: c_void;
        pub(super) static kCFTypeDictionaryValueCallBacks: c_void;
        pub(super) fn CFDictionaryCreate(
            allocator: CFTypeRef,
            keys: *const CFTypeRef,
            values: *const CFTypeRef,
            num_values: CFIndex,
            key_call_backs: *const c_void,
            value_call_backs: *const c_void,
        ) -> CFDictionaryRef;
        pub(super) fn CFRelease(cf: CFTypeRef);
    }
}

#[cfg(target_os = "macos")]
fn ax_request_trusted_with_prompt() -> bool {
    unsafe {
        let key = sys::kAXTrustedCheckOptionPrompt;
        let value = sys::kCFBooleanTrue;
        if key.is_null() || value.is_null() {
            return sys::AXIsProcessTrusted() != 0;
        }
        let keys = [key];
        let values = [value];
        let dict = sys::CFDictionaryCreate(
            std::ptr::null(),
            keys.as_ptr(),
            values.as_ptr(),
            1,
            std::ptr::addr_of!(sys::kCFTypeDictionaryKeyCallBacks).cast(),
            std::ptr::addr_of!(sys::kCFTypeDictionaryValueCallBacks).cast(),
        );
        if dict.is_null() {
            return sys::AXIsProcessTrusted() != 0;
        }
        let trusted = sys::AXIsProcessTrustedWithOptions(dict);
        sys::CFRelease(dict);
        trusted != 0
    }
}

#[cfg(target_os = "macos")]
impl PreflightHost for MacosPreflightHost {
    fn listen_event_access(&self) -> Result<bool, PreflightError> {
        // SAFETY: listen-event preflight is a no-argument, no-prompt query.
        Ok(unsafe { sys::CGPreflightListenEventAccess() } != 0)
    }

    fn accessibility_trusted(&self) -> Result<bool, PreflightError> {
        // SAFETY: AXIsProcessTrusted is the no-prompt trusted-check.
        Ok(unsafe { sys::AXIsProcessTrusted() } != 0)
    }
}

#[cfg(target_os = "macos")]
impl PermissionRequestHost for MacosPreflightHost {
    fn request_listen_event_access(&self) -> Result<bool, PreflightError> {
        // SAFETY: CGRequestListenEventAccess is the listen-only Input Monitoring
        // request. Must not run on the event-tap callback thread.
        Ok(unsafe { sys::CGRequestListenEventAccess() } != 0)
    }

    fn request_accessibility_trusted(&self) -> Result<bool, PreflightError> {
        // SAFETY: AXIsProcessTrustedWithOptions with kAXTrustedCheckOptionPrompt
        // may show the system dialog. Must not run on the event-tap callback thread.
        Ok(ax_request_trusted_with_prompt())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        prompt_used_permissions, snapshot_from_preflight, PermissionRequestHost, PreflightError,
        PreflightHost, PromptReason,
    };
    use bronze_settings::{
        manual_composer_available, privacy_settings_url, PermissionCapability, PermissionState,
        SCREEN_RECORDING_USED,
    };
    use std::cell::Cell;

    struct FakeHost {
        listen: Result<bool, PreflightError>,
        accessibility: Result<bool, PreflightError>,
    }

    impl PreflightHost for FakeHost {
        fn listen_event_access(&self) -> Result<bool, PreflightError> {
            self.listen
        }

        fn accessibility_trusted(&self) -> Result<bool, PreflightError> {
            self.accessibility
        }
    }

    struct RecordingHost {
        listen: Result<bool, PreflightError>,
        accessibility: Result<bool, PreflightError>,
        listen_grant: Result<bool, PreflightError>,
        accessibility_grant: Result<bool, PreflightError>,
        listen_requests: Cell<u32>,
        accessibility_requests: Cell<u32>,
    }

    impl PreflightHost for RecordingHost {
        fn listen_event_access(&self) -> Result<bool, PreflightError> {
            self.listen
        }

        fn accessibility_trusted(&self) -> Result<bool, PreflightError> {
            self.accessibility
        }
    }

    impl PermissionRequestHost for RecordingHost {
        fn request_listen_event_access(&self) -> Result<bool, PreflightError> {
            self.listen_requests.set(self.listen_requests.get() + 1);
            self.listen_grant
        }

        fn request_accessibility_trusted(&self) -> Result<bool, PreflightError> {
            self.accessibility_requests
                .set(self.accessibility_requests.get() + 1);
            self.accessibility_grant
        }
    }

    #[test]
    fn permission_preflight_both_true_maps_granted_unverified() {
        let snapshot = snapshot_from_preflight(&FakeHost {
            listen: Ok(true),
            accessibility: Ok(true),
        });
        assert_eq!(
            snapshot.input_monitoring,
            PermissionState::GrantedUnverified
        );
        assert_eq!(snapshot.accessibility, PermissionState::GrantedUnverified);
        assert_eq!(
            snapshot.capture_pipeline_self_test,
            PermissionState::Unknown
        );
        assert!(!snapshot.input_monitoring.is_healthy());
        assert!(!snapshot.accessibility.is_healthy());
        assert!(!snapshot.capture_pipeline_self_test.is_healthy());
    }

    #[test]
    fn permission_preflight_both_false_maps_not_requested() {
        let snapshot = snapshot_from_preflight(&FakeHost {
            listen: Ok(false),
            accessibility: Ok(false),
        });
        assert_eq!(snapshot.input_monitoring, PermissionState::NotRequested);
        assert_eq!(snapshot.accessibility, PermissionState::NotRequested);
        assert_eq!(
            snapshot.capture_pipeline_self_test,
            PermissionState::Unknown
        );
    }

    #[test]
    fn permission_independent_listen_and_ax_states() {
        let snapshot = snapshot_from_preflight(&FakeHost {
            listen: Ok(true),
            accessibility: Ok(false),
        });
        assert_eq!(
            snapshot.input_monitoring,
            PermissionState::GrantedUnverified
        );
        assert_eq!(snapshot.accessibility, PermissionState::NotRequested);
        assert!(!snapshot.input_monitoring.is_healthy());
        assert!(!snapshot.accessibility.is_healthy());
    }

    #[test]
    fn permission_platform_unknown_maps_unknown() {
        let snapshot = snapshot_from_preflight(&FakeHost {
            listen: Err(PreflightError::Unknown),
            accessibility: Err(PreflightError::Unknown),
        });
        assert_eq!(snapshot.input_monitoring, PermissionState::Unknown);
        assert_eq!(snapshot.accessibility, PermissionState::Unknown);
        assert!(!snapshot.input_monitoring.is_healthy());
        assert!(!snapshot.accessibility.is_healthy());
        assert_ne!(snapshot.input_monitoring, PermissionState::Healthy);
    }

    #[test]
    fn permission_platform_probe_fail_maps_degraded() {
        let snapshot = snapshot_from_preflight(&FakeHost {
            listen: Err(PreflightError::Fail),
            accessibility: Ok(true),
        });
        assert_eq!(snapshot.input_monitoring, PermissionState::Degraded);
        assert_eq!(snapshot.accessibility, PermissionState::GrantedUnverified);
        assert!(!snapshot.input_monitoring.is_healthy());
        assert!(!snapshot.accessibility.is_healthy());
        assert_ne!(snapshot.input_monitoring, PermissionState::Healthy);
    }

    #[test]
    fn permission_granted_not_healthy_from_preflight_wrap() {
        let snapshot = snapshot_from_preflight(&FakeHost {
            listen: Ok(true),
            accessibility: Ok(true),
        });
        assert_ne!(snapshot.input_monitoring, PermissionState::Healthy);
        assert_ne!(snapshot.accessibility, PermissionState::Healthy);
        assert_ne!(
            snapshot.capture_pipeline_self_test,
            PermissionState::Healthy
        );
    }

    #[test]
    fn native_start_requests_prompt_when_preflight_is_false() {
        let host = RecordingHost {
            listen: Ok(false),
            accessibility: Ok(false),
            listen_grant: Ok(false),
            accessibility_grant: Ok(false),
            listen_requests: Cell::new(0),
            accessibility_requests: Cell::new(0),
        };
        let attempt = prompt_used_permissions(&host, PromptReason::NativeStart);
        assert_eq!(host.listen_requests.get(), 1);
        assert_eq!(host.accessibility_requests.get(), 1);
        assert!(attempt.listen_requested);
        assert!(attempt.accessibility_requested);
        assert!(!attempt.screen_recording_requested);
        assert!(!SCREEN_RECORDING_USED);
        assert_eq!(attempt.snapshot.input_monitoring, PermissionState::Denied);
        assert_eq!(attempt.snapshot.accessibility, PermissionState::Denied);
        assert!(manual_composer_available(&attempt.snapshot));
    }

    #[test]
    fn first_capture_requests_prompt_when_preflight_is_false() {
        let host = RecordingHost {
            listen: Ok(false),
            accessibility: Ok(true),
            listen_grant: Ok(true),
            accessibility_grant: Ok(true),
            listen_requests: Cell::new(0),
            accessibility_requests: Cell::new(0),
        };
        let attempt = prompt_used_permissions(&host, PromptReason::FirstCapture);
        assert_eq!(host.listen_requests.get(), 1);
        assert_eq!(host.accessibility_requests.get(), 0);
        assert!(attempt.listen_requested);
        assert!(!attempt.accessibility_requested);
        assert!(!attempt.screen_recording_requested);
        assert_eq!(
            attempt.snapshot.input_monitoring,
            PermissionState::GrantedUnverified
        );
        assert_eq!(
            attempt.snapshot.accessibility,
            PermissionState::GrantedUnverified
        );
    }

    #[test]
    fn native_start_skips_request_when_already_granted() {
        let host = RecordingHost {
            listen: Ok(true),
            accessibility: Ok(true),
            listen_grant: Ok(true),
            accessibility_grant: Ok(true),
            listen_requests: Cell::new(0),
            accessibility_requests: Cell::new(0),
        };
        let attempt = prompt_used_permissions(&host, PromptReason::NativeStart);
        assert_eq!(host.listen_requests.get(), 0);
        assert_eq!(host.accessibility_requests.get(), 0);
        assert!(!attempt.listen_requested);
        assert!(!attempt.accessibility_requested);
        assert!(!attempt.screen_recording_requested);
    }

    #[test]
    fn health_retest_always_requests_used_permissions() {
        let host = RecordingHost {
            listen: Ok(true),
            accessibility: Ok(true),
            listen_grant: Ok(false),
            accessibility_grant: Ok(false),
            listen_requests: Cell::new(0),
            accessibility_requests: Cell::new(0),
        };
        let attempt = prompt_used_permissions(&host, PromptReason::HealthRetest);
        assert_eq!(host.listen_requests.get(), 1);
        assert_eq!(host.accessibility_requests.get(), 1);
        assert!(attempt.listen_requested);
        assert!(attempt.accessibility_requested);
        assert!(!attempt.screen_recording_requested);
        assert_eq!(attempt.snapshot.input_monitoring, PermissionState::Denied);
        assert_eq!(attempt.snapshot.accessibility, PermissionState::Denied);
        assert!(manual_composer_available(&attempt.snapshot));
        assert!(privacy_settings_url(PermissionCapability::InputMonitoring).is_some());
        assert!(privacy_settings_url(PermissionCapability::Accessibility).is_some());
        assert_eq!(
            privacy_settings_url(PermissionCapability::ScreenRecording),
            None
        );
    }

    #[test]
    fn permission_production_requests_prompt_not_preflight_only() {
        let src = include_str!("permission.rs");
        let (production, tests) = src
            .split_once("#[cfg(test)]")
            .expect("production and test modules");
        let request_access = ["CGRequest", "ListenEvent", "Access"].concat();
        let ax_prompt = ["AXIsProcessTrusted", "WithOptions"].concat();
        let prompt_option = ["kAXTrustedCheckOption", "Prompt"].concat();
        assert!(
            production.contains(&request_access),
            "production path must request listen-event access"
        );
        assert!(
            production.contains(&ax_prompt) && production.contains(&prompt_option),
            "production path must request Accessibility with prompt"
        );
        assert!(
            !production.contains("CGRequestScreenCaptureAccess")
                && !production.contains("CGPreflightScreenCaptureAccess")
                && !production.contains("CGWindowListCreateImage")
                && !production.contains("NSScreenCapture")
                && !production.contains("SCStream"),
            "Screen Recording APIs must not be called"
        );

        let live_host = format!("{}{}{}", "Macos", "Preflight", "Host");
        assert!(
            !tests.contains(&live_host),
            "tests must use a fake host only and never invoke the live host"
        );
    }
}
