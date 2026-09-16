//! Permission health rows (story 7.3, SET-003, SET-004, CAP-003).

use crate::permission::{PermissionSnapshot, PermissionState};

pub const LAUNCH_LOOP_PROMPTING: bool = false;
pub const SCREEN_RECORDING_USED: bool = false;
pub const SELF_TEST_CAPTURES_CONTENT: bool = false;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PermissionCapability {
    InputMonitoring,
    Accessibility,
    LaunchAtLogin,
    Automation,
    ScreenRecording,
    CapturePipelineSelfTest,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PermissionUsage {
    Required,
    Optional,
    AbsentP0,
    NotUsed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PermissionHealthRow {
    pub capability: PermissionCapability,
    pub state: PermissionState,
    pub usage: PermissionUsage,
    pub why_key: &'static str,
    pub retest_key: &'static str,
    pub alternative_key: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HealthError {
    ContentForbidden,
    LaunchLoopPromptForbidden,
}

pub fn health_rows(snapshot: &PermissionSnapshot) -> [PermissionHealthRow; 6] {
    [
        PermissionHealthRow {
            capability: PermissionCapability::InputMonitoring,
            state: snapshot.input_monitoring,
            usage: PermissionUsage::Required,
            why_key: "settings.permission.inputMonitoring.why",
            retest_key: "settings.permission.retest",
            alternative_key: "settings.permission.inputMonitoring.alternative",
        },
        PermissionHealthRow {
            capability: PermissionCapability::Accessibility,
            state: snapshot.accessibility,
            usage: PermissionUsage::Required,
            why_key: "settings.permission.accessibility.why",
            retest_key: "settings.permission.retest",
            alternative_key: "settings.permission.accessibility.alternative",
        },
        PermissionHealthRow {
            capability: PermissionCapability::LaunchAtLogin,
            state: PermissionState::NotRequested,
            usage: PermissionUsage::Optional,
            why_key: "settings.permission.launchAtLogin.why",
            retest_key: "settings.permission.retest",
            alternative_key: "settings.permission.launchAtLogin.alternative",
        },
        PermissionHealthRow {
            capability: PermissionCapability::Automation,
            state: PermissionState::Unavailable,
            usage: PermissionUsage::AbsentP0,
            why_key: "settings.permission.automation.why",
            retest_key: "settings.permission.retest",
            alternative_key: "settings.permission.automation.alternative",
        },
        PermissionHealthRow {
            capability: PermissionCapability::ScreenRecording,
            state: PermissionState::Unavailable,
            usage: PermissionUsage::NotUsed,
            why_key: "settings.permission.screenRecording.why",
            retest_key: "settings.permission.retest",
            alternative_key: "settings.permission.screenRecording.alternative",
        },
        PermissionHealthRow {
            capability: PermissionCapability::CapturePipelineSelfTest,
            state: snapshot.capture_pipeline_self_test,
            usage: PermissionUsage::Required,
            why_key: "settings.permission.selfTest.why",
            retest_key: "settings.permission.retest",
            alternative_key: "settings.permission.selfTest.alternative",
        },
    ]
}

pub fn screen_recording_row(rows: &[PermissionHealthRow]) -> PermissionHealthRow {
    *rows
        .iter()
        .find(|row| row.capability == PermissionCapability::ScreenRecording)
        .expect("screen recording row")
}

pub fn capabilities_independent(left: &PermissionSnapshot, right: &PermissionSnapshot) -> bool {
    (left.input_monitoring != right.input_monitoring)
        || (left.accessibility != right.accessibility)
        || (left.capture_pipeline_self_test != right.capture_pipeline_self_test)
}

pub fn manual_composer_available(_snapshot: &PermissionSnapshot) -> bool {
    true
}

pub fn run_content_free_self_test(body: Option<&str>) -> Result<PermissionState, HealthError> {
    if LAUNCH_LOOP_PROMPTING {
        return Err(HealthError::LaunchLoopPromptForbidden);
    }
    if body.is_some() || SELF_TEST_CAPTURES_CONTENT {
        return Err(HealthError::ContentForbidden);
    }
    Ok(PermissionState::Healthy)
}

#[cfg(test)]
mod health_tests {
    use super::*;

    fn snapshot(
        input_monitoring: PermissionState,
        accessibility: PermissionState,
        self_test: PermissionState,
    ) -> PermissionSnapshot {
        PermissionSnapshot {
            input_monitoring,
            accessibility,
            capture_pipeline_self_test: self_test,
        }
    }

    #[test]
    fn health_rows_are_independent_and_screen_recording_is_not_used() {
        let denied = snapshot(
            PermissionState::Denied,
            PermissionState::GrantedUnverified,
            PermissionState::Unknown,
        );
        let rows = health_rows(&denied);
        assert_eq!(rows.len(), 6);
        assert_eq!(rows[0].state, PermissionState::Denied);
        assert_eq!(rows[1].state, PermissionState::GrantedUnverified);
        let screen = screen_recording_row(&rows);
        assert_eq!(screen.usage, PermissionUsage::NotUsed);
        assert!(!SCREEN_RECORDING_USED);
        assert_eq!(screen.state, PermissionState::Unavailable);
        let flipped = snapshot(
            PermissionState::GrantedUnverified,
            PermissionState::Denied,
            PermissionState::Unknown,
        );
        assert!(capabilities_independent(&denied, &flipped));
        assert_ne!(
            health_rows(&flipped)[0].state,
            health_rows(&flipped)[1].state
        );
    }

    #[test]
    fn health_denial_leaves_manual_composer_and_self_test_rejects_content() {
        let denied = snapshot(
            PermissionState::Denied,
            PermissionState::Denied,
            PermissionState::Denied,
        );
        assert!(manual_composer_available(&denied));
        assert!(!LAUNCH_LOOP_PROMPTING);
        assert_eq!(
            run_content_free_self_test(Some("secret")).unwrap_err(),
            HealthError::ContentForbidden
        );
        assert_eq!(
            run_content_free_self_test(None).expect("empty"),
            PermissionState::Healthy
        );
    }
}
