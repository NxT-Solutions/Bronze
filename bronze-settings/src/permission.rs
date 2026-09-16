//! Closed permission health states and content-free snapshots (SET-003, SET-004, CAP-010).
//!
//! Snapshots are locale-neutral machine types (I18N-001) and carry no selected text, keys,
//! titles, URLs, or paths (ADR-015). Input Monitoring is independent of Accessibility and
//! is required later for the listen-only tap (ADR-005). Granted is never healthy.

/// Closed permission health state. Names match `tooling/planning-checks.py`.
///
/// Boolean macOS preflight cannot express this set. Map only what a bit means; other
/// variants stay constructible for later revoke, self-test, and relaunch paths.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PermissionState {
    Unknown,
    NotRequested,
    Denied,
    GrantedUnverified,
    Healthy,
    Degraded,
    Unavailable,
    RequiresRelaunch,
}

impl PermissionState {
    /// Locale-neutral snake_case name aligned with planning-checks (not UI copy).
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::NotRequested => "not_requested",
            Self::Denied => "denied",
            Self::GrantedUnverified => "granted_unverified",
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
            Self::Unavailable => "unavailable",
            Self::RequiresRelaunch => "requires_relaunch",
        }
    }

    /// Map a listen-event or AX trusted preflight bit.
    ///
    /// `true` is `granted_unverified`, never `healthy`. This story never prompts, so
    /// `false` is `not_requested`, not `denied`.
    pub const fn from_preflight_granted(granted: bool) -> Self {
        if granted {
            Self::GrantedUnverified
        } else {
            Self::NotRequested
        }
    }

    /// Unrecognized or unspecified platform probe result. Never healthy.
    pub const fn from_platform_unknown() -> Self {
        Self::Unknown
    }

    /// Known platform probe failure (not a boolean grant bit). Never healthy.
    pub const fn from_platform_probe_failure() -> Self {
        Self::Degraded
    }

    /// True only for an explicit operation-level self-test success.
    pub const fn is_healthy(self) -> bool {
        matches!(self, Self::Healthy)
    }
}

/// Content-free capability snapshot (SET-003, CAP-010, ADR-015).
///
/// Three independent P0 capabilities. Automation and Launch at Login are omitted from
/// P0. Screen Recording is not a capability. `Copy` compile-rejects `String` content
/// slots (selected text, clipboard, titles, URLs, paths, tokens, hashes).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PermissionSnapshot {
    pub input_monitoring: PermissionState,
    pub accessibility: PermissionState,
    pub capture_pipeline_self_test: PermissionState,
}

#[cfg(test)]
mod tests {
    use super::{PermissionSnapshot, PermissionState};

    #[test]
    fn permission_closed_set_matches_planning_checks() {
        let expected = [
            (PermissionState::Unknown, "unknown", false),
            (PermissionState::NotRequested, "not_requested", false),
            (PermissionState::Denied, "denied", false),
            (
                PermissionState::GrantedUnverified,
                "granted_unverified",
                false,
            ),
            (PermissionState::Healthy, "healthy", true),
            (PermissionState::Degraded, "degraded", false),
            (PermissionState::Unavailable, "unavailable", false),
            (
                PermissionState::RequiresRelaunch,
                "requires_relaunch",
                false,
            ),
        ];
        assert_eq!(expected.len(), 8);
        for (state, name, healthy) in expected {
            assert_eq!(state.as_str(), name);
            assert_eq!(state.is_healthy(), healthy);
        }
    }

    #[test]
    fn permission_granted_not_healthy() {
        let granted = PermissionState::from_preflight_granted(true);
        assert_eq!(granted, PermissionState::GrantedUnverified);
        assert!(!granted.is_healthy());
        assert_ne!(granted, PermissionState::Healthy);
        assert_eq!(
            PermissionState::from_preflight_granted(false),
            PermissionState::NotRequested
        );
    }

    #[test]
    fn permission_platform_unknown_never_healthy() {
        let state = PermissionState::from_platform_unknown();
        assert_eq!(state, PermissionState::Unknown);
        assert!(!state.is_healthy());
    }

    #[test]
    fn permission_platform_probe_failure_is_degraded_never_healthy() {
        let state = PermissionState::from_platform_probe_failure();
        assert_eq!(state, PermissionState::Degraded);
        assert!(!state.is_healthy());
    }

    #[test]
    fn permission_independent_states() {
        let snapshot = PermissionSnapshot {
            input_monitoring: PermissionState::from_preflight_granted(true),
            accessibility: PermissionState::from_preflight_granted(false),
            capture_pipeline_self_test: PermissionState::from_platform_unknown(),
        };
        assert_eq!(
            snapshot.input_monitoring,
            PermissionState::GrantedUnverified
        );
        assert_eq!(snapshot.accessibility, PermissionState::NotRequested);
        assert_eq!(
            snapshot.capture_pipeline_self_test,
            PermissionState::Unknown
        );
        assert!(!snapshot.input_monitoring.is_healthy());
        assert!(!snapshot.accessibility.is_healthy());
    }

    #[test]
    fn permission_explicit_denied_is_not_rewritten_to_healthy() {
        let snapshot = PermissionSnapshot {
            input_monitoring: PermissionState::Denied,
            accessibility: PermissionState::Denied,
            capture_pipeline_self_test: PermissionState::Denied,
        };
        assert_eq!(snapshot.input_monitoring, PermissionState::Denied);
        assert_eq!(snapshot.accessibility, PermissionState::Denied);
        assert_eq!(snapshot.capture_pipeline_self_test, PermissionState::Denied);
        assert!(!snapshot.input_monitoring.is_healthy());
        assert_ne!(snapshot.input_monitoring, PermissionState::Healthy);
        assert_ne!(
            snapshot.input_monitoring,
            PermissionState::GrantedUnverified
        );
    }

    #[test]
    fn permission_self_test_only_healthy() {
        let snapshot = PermissionSnapshot {
            input_monitoring: PermissionState::from_preflight_granted(true),
            accessibility: PermissionState::from_preflight_granted(true),
            capture_pipeline_self_test: PermissionState::Healthy,
        };
        assert_eq!(
            snapshot.input_monitoring,
            PermissionState::GrantedUnverified
        );
        assert_eq!(snapshot.accessibility, PermissionState::GrantedUnverified);
        assert_eq!(
            snapshot.capture_pipeline_self_test,
            PermissionState::Healthy
        );
        assert!(!snapshot.input_monitoring.is_healthy());
        assert!(!snapshot.accessibility.is_healthy());
        assert!(snapshot.capture_pipeline_self_test.is_healthy());
    }

    #[test]
    fn permission_snapshot_has_no_content_fields() {
        fn assert_copy<T: Copy>() {}
        assert_copy::<PermissionState>();
        assert_copy::<PermissionSnapshot>();

        let snapshot = PermissionSnapshot {
            input_monitoring: PermissionState::Unknown,
            accessibility: PermissionState::NotRequested,
            capture_pipeline_self_test: PermissionState::Denied,
        };
        let PermissionSnapshot {
            input_monitoring,
            accessibility,
            capture_pipeline_self_test,
        } = snapshot;
        let _: PermissionState = input_monitoring;
        let _: PermissionState = accessibility;
        let _: PermissionState = capture_pipeline_self_test;

        let debug = format!("{snapshot:?}");
        assert!(debug.contains("input_monitoring"));
        assert!(debug.contains("accessibility"));
        assert!(debug.contains("capture_pipeline_self_test"));
        for forbidden in [
            "selected",
            "clipboard",
            "keycode",
            "key_stream",
            "http://",
            "https://",
            "screen_recording",
            "automation",
            "launch_at_login",
            "microphone",
            "camera",
        ] {
            assert!(
                !debug.to_ascii_lowercase().contains(forbidden),
                "snapshot Debug must not carry {forbidden}: {debug}"
            );
        }
    }
}
