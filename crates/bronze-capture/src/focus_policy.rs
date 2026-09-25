//! Capture-only focus policy (story 5.3, WIN-003, CAP-004, A11Y-006).
//! Hidden WKWebView live regions are not sufficient feedback.

pub const CAPTURE_ONLY_ANNOUNCE_KEY: &str = "capture.announce.saved";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CaptureMode {
    CaptureOnly,
    OpenPanel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FocusOwner {
    Source,
    Bronze,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FocusSnapshot {
    pub owner: FocusOwner,
    pub source_token: u64,
}

pub trait Announcer {
    fn announce(&mut self, catalog_key: &str);
}

#[derive(Default)]
pub struct FakeAnnouncer {
    pub keys: Vec<String>,
}

impl Announcer for FakeAnnouncer {
    fn announce(&mut self, catalog_key: &str) {
        self.keys.push(catalog_key.to_string());
    }
}

pub fn hidden_webview_live_region_is_sufficient() -> bool {
    false
}

pub fn apply_capture_success(
    mode: CaptureMode,
    prior: FocusSnapshot,
    announcer: &mut dyn Announcer,
    webview_visible: bool,
) -> FocusOwner {
    match mode {
        CaptureMode::CaptureOnly => {
            if !webview_visible && !hidden_webview_live_region_is_sufficient() {
                announcer.announce(CAPTURE_ONLY_ANNOUNCE_KEY);
            }
            prior.owner
        }
        CaptureMode::OpenPanel => FocusOwner::Bronze,
    }
}

pub fn escape_restores_prior_focus(prior: FocusSnapshot, restore_safe: bool) -> FocusOwner {
    if restore_safe {
        prior.owner
    } else {
        FocusOwner::Bronze
    }
}

#[cfg(test)]
mod focus_policy_tests {
    use super::*;

    #[test]
    fn focus_policy_capture_only_does_not_steal_source_focus() {
        let prior = FocusSnapshot {
            owner: FocusOwner::Source,
            source_token: 9,
        };
        let mut announce = FakeAnnouncer::default();
        let after = apply_capture_success(CaptureMode::CaptureOnly, prior, &mut announce, false);
        assert_eq!(after, FocusOwner::Source);
        assert_ne!(after, FocusOwner::Bronze);
        let panel = apply_capture_success(
            CaptureMode::OpenPanel,
            prior,
            &mut FakeAnnouncer::default(),
            true,
        );
        assert_eq!(panel, FocusOwner::Bronze);
    }

    #[test]
    fn focus_policy_announces_via_native_api_when_webview_hidden() {
        assert!(!hidden_webview_live_region_is_sufficient());
        let prior = FocusSnapshot {
            owner: FocusOwner::Source,
            source_token: 1,
        };
        let mut hidden = FakeAnnouncer::default();
        apply_capture_success(CaptureMode::CaptureOnly, prior, &mut hidden, false);
        assert_eq!(hidden.keys, [CAPTURE_ONLY_ANNOUNCE_KEY]);
        let mut visible = FakeAnnouncer::default();
        apply_capture_success(CaptureMode::CaptureOnly, prior, &mut visible, true);
        assert!(visible.keys.is_empty());
    }

    #[test]
    fn focus_policy_escape_restores_prior_focus_when_safe() {
        let prior = FocusSnapshot {
            owner: FocusOwner::Source,
            source_token: 3,
        };
        assert_eq!(escape_restores_prior_focus(prior, true), FocusOwner::Source);
        assert_eq!(
            escape_restores_prior_focus(prior, false),
            FocusOwner::Bronze
        );
    }
}
