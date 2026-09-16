//! Capture request/state machine. Story 3.4 lands ingress context here.

mod ingress;

pub use ingress::{
    CaptureIngressContext, IngressError, IngressSeqlock, INGRESS_ROUTE_CHORD,
    INGRESS_ROUTE_EVENT_TAP, INGRESS_ROUTE_MENU,
};
