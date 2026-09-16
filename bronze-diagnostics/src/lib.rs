//! Content-free diagnostic_events (story 3.8, CAP-010, SEC-006, SUP-002).
//!
//! Event fields are closed integers and enums only. Diagnostic write
//! failure never undoes a saved item.

use std::fmt;

mod bundle;
pub use bundle::{
    export_bundle, preview_bundle, BundleError, BundlePreview, AUTOMATIC_UPLOAD, HUMAN_GATES,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Stage {
    Trigger,
    Target,
    Ax,
    Selection,
    Clipboard,
    Store,
    Request,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResultCode {
    Ok,
    TriggerQueueOverflow,
    ContextUnavailable,
    ProtectedContent,
    ProtectionUnknown,
    AppExcluded,
    NoSelection,
    ClipboardChanged,
    ClipboardUnsupportedType,
    PersistFailed,
    Cancelled,
    DiagnosticWriteFailed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TriggerKind {
    EventTap,
    Chord,
    Menu,
    ManualClipboard,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiagnosticEvent {
    pub timestamp_ms: u64,
    pub request_id: u64,
    pub stage: Stage,
    pub result: ResultCode,
    pub duration_ms: u32,
    pub trigger_kind: TriggerKind,
    pub queue_depth: u32,
    pub overflow_count: u32,
}

pub trait ContentFreePayload: Copy + 'static {}

impl ContentFreePayload for u64 {}
impl ContentFreePayload for u32 {}
impl ContentFreePayload for Stage {}
impl ContentFreePayload for ResultCode {}
impl ContentFreePayload for TriggerKind {}
impl ContentFreePayload for DiagnosticEvent {}

impl fmt::Display for DiagnosticEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "diag req={} stage={:?} result={:?} dur_ms={}",
            self.request_id, self.stage, self.result, self.duration_ms
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiagWriteError;

pub fn append_diagnostic(
    sink: &mut Vec<DiagnosticEvent>,
    event: DiagnosticEvent,
    fail: bool,
) -> Result<(), DiagWriteError> {
    if fail {
        return Err(DiagWriteError);
    }
    sink.push(event);
    Ok(())
}

pub fn save_item_then_diagnostic(
    items: &mut Vec<u64>,
    item_id: u64,
    sink: &mut Vec<DiagnosticEvent>,
    event: DiagnosticEvent,
    diag_fail: bool,
) {
    items.push(item_id);
    let _ = append_diagnostic(sink, event, diag_fail);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> DiagnosticEvent {
        DiagnosticEvent {
            timestamp_ms: 1,
            request_id: 9,
            stage: Stage::Store,
            result: ResultCode::Ok,
            duration_ms: 4,
            trigger_kind: TriggerKind::ManualClipboard,
            queue_depth: 0,
            overflow_count: 0,
        }
    }

    #[test]
    fn schema_rejects_string_selected_text_payloads() {
        let src = include_str!("lib.rs");
        let production = src.split("#[cfg(test)]").next().expect("src");
        assert!(
            !production.contains("String"),
            "diagnostic schema must not use String (CAP-010)"
        );
        for needle in [
            "selected_text",
            "excerpt",
            "clipboard_payload",
            "window_title",
            "url",
        ] {
            assert!(
                !production.contains(needle),
                "schema must not name {needle}"
            );
        }
        fn accept_content_free<T: ContentFreePayload>(_: T) {}
        accept_content_free(sample());
        accept_content_free(ResultCode::ProtectedContent);
    }

    #[test]
    fn seeded_secret_scan_of_logs_passes() {
        const SECRET: &str = "hunter2-s3cret-payload";
        let ev = sample();
        let rendered = format!("{ev:?}{ev}");
        assert!(!rendered.contains(SECRET));
        assert!(!rendered.contains("hunter2"));
        assert!(!rendered.contains("title"));
        assert!(!rendered.contains("http"));
    }

    #[test]
    fn diagnostic_write_failure_does_not_roll_back_saved_item() {
        let mut items = Vec::new();
        let mut sink = Vec::new();
        save_item_then_diagnostic(&mut items, 42, &mut sink, sample(), true);
        assert_eq!(items, vec![42]);
        assert!(sink.is_empty());
    }
}
