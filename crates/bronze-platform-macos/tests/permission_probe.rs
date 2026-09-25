//! Request-path mapping when a preflight probe is not a grant bit.

use std::cell::Cell;

use bronze_platform_macos::{
    prompt_used_permissions, MemoryPromptLedger, PermissionRequestHost, PreflightError,
    PreflightHost, PromptReason,
};
use bronze_settings::{manual_composer_available, PermissionState};

struct ProbeHost {
    listen: Result<bool, PreflightError>,
    accessibility: Result<bool, PreflightError>,
    listen_grant: Result<bool, PreflightError>,
    accessibility_grant: Result<bool, PreflightError>,
    listen_requests: Cell<u32>,
    accessibility_requests: Cell<u32>,
}

impl PreflightHost for ProbeHost {
    fn listen_event_access(&self) -> Result<bool, PreflightError> {
        self.listen
    }

    fn accessibility_trusted(&self) -> Result<bool, PreflightError> {
        self.accessibility
    }
}

impl PermissionRequestHost for ProbeHost {
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
fn native_start_requests_when_preflight_is_not_a_grant() {
    let host = ProbeHost {
        listen: Err(PreflightError::Fail),
        accessibility: Err(PreflightError::Unknown),
        listen_grant: Ok(false),
        accessibility_grant: Ok(true),
        listen_requests: Cell::new(0),
        accessibility_requests: Cell::new(0),
    };
    let attempt = prompt_used_permissions(
        &host,
        PromptReason::NativeStart,
        &MemoryPromptLedger::default(),
    );
    assert_eq!(host.listen_requests.get(), 1);
    assert_eq!(host.accessibility_requests.get(), 1);
    assert!(attempt.listen_requested);
    assert!(attempt.accessibility_requested);
    assert!(!attempt.screen_recording_requested);
    assert_eq!(attempt.snapshot.input_monitoring, PermissionState::Denied);
    assert_eq!(
        attempt.snapshot.accessibility,
        PermissionState::GrantedUnverified
    );
    assert!(!attempt.snapshot.input_monitoring.is_healthy());
    assert!(manual_composer_available(&attempt.snapshot));
}

#[test]
fn health_retest_probe_errors_stay_unhealthy() {
    let host = ProbeHost {
        listen: Ok(true),
        accessibility: Ok(true),
        listen_grant: Err(PreflightError::Fail),
        accessibility_grant: Err(PreflightError::Unknown),
        listen_requests: Cell::new(0),
        accessibility_requests: Cell::new(0),
    };
    let attempt = prompt_used_permissions(
        &host,
        PromptReason::HealthRetest,
        &MemoryPromptLedger::default(),
    );
    assert_eq!(host.listen_requests.get(), 1);
    assert_eq!(host.accessibility_requests.get(), 1);
    assert_eq!(attempt.snapshot.input_monitoring, PermissionState::Degraded);
    assert_eq!(attempt.snapshot.accessibility, PermissionState::Unknown);
    assert!(!attempt.snapshot.input_monitoring.is_healthy());
    assert!(!attempt.snapshot.accessibility.is_healthy());
    assert!(!attempt.screen_recording_requested);
}
