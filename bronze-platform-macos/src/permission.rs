//! No-prompt Input Monitoring and Accessibility preflight wrap (SET-003, CAP-010, ADR-005).
//!
//! Business mapping lives in `bronze-settings`. This façade only reads listen-event
//! preflight and the no-prompt accessibility trusted-check. Request-access is not
//! wrapped. Prompts exist later behind a labeled Enable path, not this story.
//! Tests use [`PreflightHost`] fakes only.

use bronze_settings::{PermissionSnapshot, PermissionState};

/// Closed probe error. Unrecognized results are [`Self::Unknown`]; known failures
/// that are not a grant bit are [`Self::Fail`]. Neither maps to healthy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreflightError {
    Unknown,
    Fail,
}

/// Host that reports Input Monitoring and Accessibility preflight bits.
///
/// Methods have no prompt flag. Implementations must not request access.
pub trait PreflightHost {
    fn listen_event_access(&self) -> Result<bool, PreflightError>;
    fn accessibility_trusted(&self) -> Result<bool, PreflightError>;
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

/// Live macOS preflight host. Unit tests must not construct or call this type.
///
/// Uses `CGPreflightListenEventAccess` and `AXIsProcessTrusted` (the no-prompt
/// trusted-check). Prompt cannot be enabled; this type exposes no prompt parameter.
#[cfg(target_os = "macos")]
pub struct MacosPreflightHost;

#[cfg(target_os = "macos")]
mod sys {
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        pub(super) fn CGPreflightListenEventAccess() -> u8;
    }

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        pub(super) fn AXIsProcessTrusted() -> u8;
    }
}

#[cfg(target_os = "macos")]
impl PreflightHost for MacosPreflightHost {
    fn listen_event_access(&self) -> Result<bool, PreflightError> {
        // SAFETY: listen-event preflight is a no-argument, no-prompt query.
        Ok(unsafe { sys::CGPreflightListenEventAccess() } != 0)
    }

    fn accessibility_trusted(&self) -> Result<bool, PreflightError> {
        // SAFETY: AXIsProcessTrusted is the no-prompt trusted-check; no options
        // dictionary and therefore no prompt bit can be set.
        Ok(unsafe { sys::AXIsProcessTrusted() } != 0)
    }
}

#[cfg(test)]
mod tests {
    use super::{snapshot_from_preflight, PreflightError, PreflightHost};
    use bronze_settings::PermissionState;

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
    fn permission_no_prompt_surface_uses_fake_host_only() {
        let src = include_str!("permission.rs");
        let (production, tests) = src
            .split_once("#[cfg(test)]")
            .expect("production and test modules");
        let request_access = ["CGRequest", "ListenEvent", "Access"].concat();
        let prompt_flag = format!("{}: {}", "prompt", "bool");
        let prompt_enabled = format!("{} = {}", "prompt", "true");
        assert!(
            !production.contains(&request_access),
            "request-access must not be wrapped; prompts are a later labeled Enable path"
        );
        assert!(
            !production.contains(&prompt_flag) && !production.contains(&prompt_enabled),
            "public wrap must not expose or enable a prompt flag"
        );

        let live_host = format!("{}{}{}", "Macos", "Preflight", "Host");
        assert!(
            !tests.contains(&live_host),
            "tests must use a fake host only and never invoke the live host"
        );

        let host = FakeHost {
            listen: Ok(false),
            accessibility: Ok(false),
        };
        let _ = snapshot_from_preflight(&host);
    }
}
