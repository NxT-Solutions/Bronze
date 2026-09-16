//! Capture request/state machine. Story 3.4 lands ingress; 3.5 the coordinator.

mod coordinator;
mod ingress;

pub use coordinator::{
    AlwaysPersist, CaptureCoordinator, CaptureReceipt, DiagnosticEvent, FeedbackHook, NoFeedback,
    PersistError, PersistHook, Terminal,
};
pub use ingress::{
    CaptureIngressContext, IngressError, IngressSeqlock, INGRESS_ROUTE_CHORD,
    INGRESS_ROUTE_EVENT_TAP, INGRESS_ROUTE_MENU,
};
