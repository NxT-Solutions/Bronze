//! Redacted support bundle (story 9.2, SUP-001/002, SEC-006).
//! Preview before export. No automatic upload. Human gates stay backlog.

use crate::{DiagnosticEvent, ResultCode, Stage};

pub const AUTOMATIC_UPLOAD: bool = false;
pub const HUMAN_GATES: &[&str] = &["3.9", "3.10", "5.5", "9.3"];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BundlePreview {
    pub event_count: usize,
    pub stages: Vec<Stage>,
    pub results: Vec<ResultCode>,
    pub known_limitations: Vec<&'static str>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BundleError {
    SecretDetected,
    PreviewRequired,
}

fn looks_secret(text: &str) -> bool {
    crate::report::contains_forbidden_payload(text)
}

pub fn preview_bundle(events: &[DiagnosticEvent]) -> Result<BundlePreview, BundleError> {
    let rendered = format!("{events:?}");
    if looks_secret(&rendered) {
        return Err(BundleError::SecretDetected);
    }
    Ok(BundlePreview {
        event_count: events.len(),
        stages: events.iter().map(|e| e.stage).collect(),
        results: events.iter().map(|e| e.result).collect(),
        known_limitations: HUMAN_GATES.to_vec(),
    })
}

pub fn export_bundle(preview: &BundlePreview, acknowledged: bool) -> Result<String, BundleError> {
    if !acknowledged {
        return Err(BundleError::PreviewRequired);
    }
    let body = format!(
        "events={} stages={:?} results={:?} limits={:?}",
        preview.event_count, preview.stages, preview.results, preview.known_limitations
    );
    if looks_secret(&body) {
        return Err(BundleError::SecretDetected);
    }
    Ok(body)
}

#[cfg(test)]
mod bundle_tests {
    use super::*;
    use crate::{DiagnosticEvent, TriggerKind};

    fn sample() -> DiagnosticEvent {
        DiagnosticEvent {
            timestamp_ms: 1,
            request_id: 9,
            stage: Stage::Store,
            result: ResultCode::Ok,
            duration_ms: 4,
            trigger_kind: TriggerKind::Menu,
            queue_depth: 0,
            overflow_count: 0,
        }
    }

    #[test]
    fn preview_redacts_and_requires_ack_without_upload() {
        assert!(!AUTOMATIC_UPLOAD);
        let preview = preview_bundle(&[sample()]).expect("preview");
        assert_eq!(preview.event_count, 1);
        assert_eq!(preview.known_limitations, HUMAN_GATES);
        assert!(preview.known_limitations.contains(&"3.9"));
        assert_eq!(
            export_bundle(&preview, false),
            Err(BundleError::PreviewRequired)
        );
        let exported = export_bundle(&preview, true).expect("export");
        assert!(!exported.contains("hunter2"));
        assert!(!exported.contains("http"));
        assert!(exported.contains("3.9"));
    }
}
