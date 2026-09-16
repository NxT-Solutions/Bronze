//! Capture request/state machine. Stories 3.4–3.6: ingress, coordinator, AX fakes.

mod ax;
mod clipboard_manual;
mod coordinator;
mod focus_policy;
mod ingress;

pub use ax::{
    capture as ax_capture, classify as ax_classify, store_capture as ax_store_capture, AxOutcome,
    AxRole, AxSubrole, CapturedText, FakeAxNode, FakeAxTree, FakeSelection, ProtectionClass,
    AX_MAX_SELECTION_BYTES,
};
pub use clipboard_manual::{
    create_from_clipboard, ClipboardKind, ClipboardOutcome, ClipboardSettings, Pasteboard,
};
pub use coordinator::{
    AlwaysPersist, CaptureCoordinator, CaptureReceipt, DiagnosticEvent, FeedbackHook, NoFeedback,
    PersistError, PersistHook, Terminal,
};
pub use focus_policy::{
    apply_capture_success, escape_restores_prior_focus, hidden_webview_live_region_is_sufficient,
    Announcer, CaptureMode, FakeAnnouncer, FocusOwner, FocusSnapshot, CAPTURE_ONLY_ANNOUNCE_KEY,
};
pub use ingress::{
    CaptureIngressContext, IngressError, IngressSeqlock, INGRESS_ROUTE_CHORD,
    INGRESS_ROUTE_EVENT_TAP, INGRESS_ROUTE_MENU,
};
