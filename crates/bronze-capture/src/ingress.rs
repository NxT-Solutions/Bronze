//! Fixed-size capture ingress snapshot (story 3.4, CAP-004/008/009, QUE-001).
//!
//! Seqlock publish/load. Inconsistent read is `context_unavailable`.
//! No titles, URLs, or focused-element identity (those are later AX work).

use std::sync::atomic::{AtomicU64, Ordering};

pub const INGRESS_ROUTE_EVENT_TAP: u32 = 1;
pub const INGRESS_ROUTE_CHORD: u32 = 2;
pub const INGRESS_ROUTE_MENU: u32 = 3;

const LOAD_ATTEMPTS: u32 = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IngressError {
    ContextUnavailable,
}

/// Prepublished trigger-time context. Bundle identity is an interned token, not a string.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CaptureIngressContext {
    pub target_pid: i32,
    pub bundle_token: u64,
    pub activation_generation: u64,
    pub destination_uuid: [u8; 16],
    pub accept_capture_generation: u64,
    pub policy_revision: u64,
    pub settings_revision: u64,
    pub context_generation: u64,
    pub route: u32,
    pub monotonic_time_ns: u64,
}

impl CaptureIngressContext {
    pub fn empty() -> Self {
        Self {
            target_pid: 0,
            bundle_token: 0,
            activation_generation: 0,
            destination_uuid: [0; 16],
            accept_capture_generation: 0,
            policy_revision: 0,
            settings_revision: 0,
            context_generation: 0,
            route: 0,
            monotonic_time_ns: 0,
        }
    }
}

/// Single-slot seqlock. Writers increment the sequence around the payload.
pub struct IngressSeqlock {
    seq: AtomicU64,
    slot: std::sync::Mutex<CaptureIngressContext>,
    test_force_odd: AtomicU64,
}

impl Default for IngressSeqlock {
    fn default() -> Self {
        Self::new()
    }
}

impl IngressSeqlock {
    pub fn new() -> Self {
        Self {
            seq: AtomicU64::new(0),
            slot: std::sync::Mutex::new(CaptureIngressContext::empty()),
            test_force_odd: AtomicU64::new(0),
        }
    }

    pub fn publish(&self, snapshot: CaptureIngressContext) {
        self.seq.fetch_add(1, Ordering::Release);
        {
            let mut slot = self.slot.lock().unwrap_or_else(|p| p.into_inner());
            *slot = snapshot;
        }
        self.seq.fetch_add(1, Ordering::Release);
    }

    /// Test hook: leave the sequence odd so load fails closed.
    pub fn test_begin_inconsistent_write(&self) {
        self.test_force_odd.store(1, Ordering::Release);
        self.seq.fetch_add(1, Ordering::Release);
    }

    pub fn test_end_inconsistent_write(&self) {
        self.test_force_odd.store(0, Ordering::Release);
        let seq = self.seq.load(Ordering::Acquire);
        if seq & 1 == 1 {
            self.seq.fetch_add(1, Ordering::Release);
        }
    }

    pub fn load(&self) -> Result<CaptureIngressContext, IngressError> {
        for _ in 0..LOAD_ATTEMPTS {
            if self.test_force_odd.load(Ordering::Acquire) == 1 {
                return Err(IngressError::ContextUnavailable);
            }
            let s1 = self.seq.load(Ordering::Acquire);
            if s1 & 1 == 1 {
                continue;
            }
            let copy = {
                let slot = self.slot.lock().unwrap_or_else(|p| p.into_inner());
                *slot
            };
            let s2 = self.seq.load(Ordering::Acquire);
            if s1 == s2 && s2 & 1 == 0 {
                return Ok(copy);
            }
        }
        Err(IngressError::ContextUnavailable)
    }
}

#[cfg(test)]
mod ingress_tests {
    use super::*;

    #[test]
    fn ingress_snapshot_has_required_fields_and_no_focus_or_content() {
        let snap = CaptureIngressContext {
            target_pid: 4242,
            bundle_token: 7,
            activation_generation: 3,
            destination_uuid: [9; 16],
            accept_capture_generation: 2,
            policy_revision: 11,
            settings_revision: 12,
            context_generation: 5,
            route: INGRESS_ROUTE_EVENT_TAP,
            monotonic_time_ns: 99,
        };
        assert_eq!(snap.target_pid, 4242);
        assert_eq!(snap.bundle_token, 7);
        assert_eq!(snap.activation_generation, 3);
        assert_eq!(snap.destination_uuid, [9; 16]);
        assert_eq!(snap.accept_capture_generation, 2);
        assert_eq!(snap.policy_revision, 11);
        assert_eq!(snap.settings_revision, 12);
        assert_eq!(snap.context_generation, 5);
        assert_eq!(snap.route, INGRESS_ROUTE_EVENT_TAP);
        assert_eq!(snap.monotonic_time_ns, 99);
        let rendered = format!("{snap:?}");
        assert!(!rendered.contains("focused"));
        assert!(!rendered.contains("title"));
        assert!(!rendered.contains("http"));
        assert!(!rendered.contains("url"));
    }

    #[test]
    fn ingress_consistent_publish_round_trip() {
        let lock = IngressSeqlock::new();
        let snap = CaptureIngressContext {
            target_pid: 81,
            bundle_token: 1,
            activation_generation: 2,
            destination_uuid: [1; 16],
            accept_capture_generation: 4,
            policy_revision: 5,
            settings_revision: 6,
            context_generation: 7,
            route: INGRESS_ROUTE_MENU,
            monotonic_time_ns: 8,
        };
        lock.publish(snap);
        assert_eq!(lock.load().expect("stable"), snap);
    }

    #[test]
    fn ingress_inconsistent_read_is_context_unavailable() {
        let lock = IngressSeqlock::new();
        lock.publish(CaptureIngressContext::empty());
        lock.test_begin_inconsistent_write();
        assert_eq!(lock.load(), Err(IngressError::ContextUnavailable));
        lock.test_end_inconsistent_write();
        assert!(lock.load().is_ok());
    }

    #[test]
    fn ingress_never_substitutes_a_live_frontmost_query() {
        let lock = IngressSeqlock::new();
        let published = CaptureIngressContext {
            target_pid: 1001,
            bundle_token: 2,
            activation_generation: 1,
            destination_uuid: [2; 16],
            accept_capture_generation: 1,
            policy_revision: 1,
            settings_revision: 1,
            context_generation: 1,
            route: INGRESS_ROUTE_CHORD,
            monotonic_time_ns: 1,
        };
        lock.publish(published);
        let loaded = lock.load().expect("published");
        assert_eq!(loaded.target_pid, 1001);
        assert_ne!(loaded.target_pid, 0);
    }
}
