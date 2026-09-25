//! Serial capture coordinator (story 3.5, CAP-004, G-01, G-02).
//!
//! One terminal per request ID. Overflow synthesizes `trigger_queue_overflow`
//! for each missing sequence. Success feedback never precedes the persist hook.

use crate::ingress::CaptureIngressContext;
use std::collections::{HashMap, VecDeque};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Terminal {
    Saved,
    Rejected,
    Failed,
    Cancelled,
    TriggerQueueOverflow,
    ContextUnavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PersistError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CaptureReceipt {
    pub request_id: u64,
    pub terminal: Terminal,
}

pub trait PersistHook {
    fn persist(&mut self, request_id: u64) -> Result<(), PersistError>;
}

pub trait FeedbackHook {
    fn success(&mut self, request_id: u64);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiagnosticEvent {
    pub request_id: u64,
    pub terminal: Terminal,
}

pub struct AlwaysPersist;

impl PersistHook for AlwaysPersist {
    fn persist(&mut self, _request_id: u64) -> Result<(), PersistError> {
        Ok(())
    }
}

pub struct NoFeedback;

impl FeedbackHook for NoFeedback {
    fn success(&mut self, _request_id: u64) {}
}

pub struct CaptureCoordinator<P, F> {
    capacity: usize,
    next_sequence: u64,
    completed_watermark: u64,
    drained_through: u64,
    queued: VecDeque<u64>,
    slots: HashMap<u64, CaptureIngressContext>,
    receipts: HashMap<u64, CaptureReceipt>,
    drain_order: Vec<u64>,
    diagnostics: Vec<DiagnosticEvent>,
    persist: P,
    feedback: F,
}

impl CaptureCoordinator<AlwaysPersist, NoFeedback> {
    pub fn new(capacity: usize) -> Self {
        Self::with_hooks(capacity, AlwaysPersist, NoFeedback)
    }
}

impl<P: PersistHook, F: FeedbackHook> CaptureCoordinator<P, F> {
    pub fn with_hooks(capacity: usize, persist: P, feedback: F) -> Self {
        Self {
            capacity: capacity.max(1),
            next_sequence: 1,
            completed_watermark: 0,
            drained_through: 0,
            queued: VecDeque::new(),
            slots: HashMap::new(),
            receipts: HashMap::new(),
            drain_order: Vec::new(),
            diagnostics: Vec::new(),
            persist,
            feedback,
        }
    }

    pub fn submit(&mut self, ingress: CaptureIngressContext) -> u64 {
        let request_id = self.next_sequence;
        self.next_sequence += 1;
        if self.queued.len() < self.capacity {
            self.queued.push_back(request_id);
            self.slots.insert(request_id, ingress);
        }
        self.completed_watermark = request_id;
        request_id
    }

    pub fn submit_unavailable(&mut self) -> u64 {
        let request_id = self.next_sequence;
        self.next_sequence += 1;
        self.complete(request_id, Terminal::ContextUnavailable);
        self.completed_watermark = request_id;
        request_id
    }

    pub fn drain(&mut self) {
        while self.drained_through < self.completed_watermark {
            self.drained_through += 1;
            let request_id = self.drained_through;
            if let Some(receipt) = self.receipts.get(&request_id).copied() {
                self.drain_order.push(receipt.request_id);
                continue;
            }
            if let Some(_ingress) = self.slots.remove(&request_id) {
                self.queued.retain(|id| *id != request_id);
                self.persist_then_feedback(request_id);
            } else {
                self.complete(request_id, Terminal::TriggerQueueOverflow);
            }
        }
    }

    pub fn receipt(&self, request_id: u64) -> Option<CaptureReceipt> {
        self.receipts.get(&request_id).copied()
    }

    pub fn drain_order(&self) -> &[u64] {
        &self.drain_order
    }

    pub fn diagnostics(&self) -> &[DiagnosticEvent] {
        &self.diagnostics
    }

    fn persist_then_feedback(&mut self, request_id: u64) {
        match self.persist.persist(request_id) {
            Ok(()) => {
                self.complete(request_id, Terminal::Saved);
                self.feedback.success(request_id);
            }
            Err(PersistError) => self.complete(request_id, Terminal::Failed),
        }
    }

    fn complete(&mut self, request_id: u64, terminal: Terminal) {
        if let Some(existing) = self.receipts.get(&request_id) {
            self.drain_order.push(existing.request_id);
            return;
        }
        let receipt = CaptureReceipt {
            request_id,
            terminal,
        };
        self.receipts.insert(request_id, receipt);
        self.diagnostics.push(DiagnosticEvent {
            request_id,
            terminal,
        });
        self.drain_order.push(request_id);
    }
}

#[cfg(test)]
mod coordinator_tests {
    use super::*;
    use crate::ingress::{CaptureIngressContext, INGRESS_ROUTE_EVENT_TAP};

    fn snap(token: u64) -> CaptureIngressContext {
        let mut ctx = CaptureIngressContext::empty();
        ctx.bundle_token = token;
        ctx.route = INGRESS_ROUTE_EVENT_TAP;
        ctx.monotonic_time_ns = token;
        ctx
    }

    #[test]
    fn coordinator_twenty_synthetic_triggers_preserve_order_and_terminate() {
        let mut coord = CaptureCoordinator::new(32);
        let ids: Vec<u64> = (0..20).map(|i| coord.submit(snap(i + 1))).collect();
        coord.drain();
        assert_eq!(ids, (1..=20).collect::<Vec<_>>());
        assert_eq!(coord.drain_order(), ids.as_slice());
        for id in ids {
            let receipt = coord.receipt(id).expect("terminal");
            assert_eq!(receipt.terminal, Terminal::Saved);
            assert_eq!(receipt.request_id, id);
        }
    }

    #[test]
    fn coordinator_overflow_emits_trigger_queue_overflow_per_missing_id() {
        let mut coord = CaptureCoordinator::new(2);
        let ids: Vec<u64> = (0..5).map(|i| coord.submit(snap(i + 1))).collect();
        coord.drain();
        assert_eq!(coord.receipt(ids[0]).unwrap().terminal, Terminal::Saved);
        assert_eq!(coord.receipt(ids[1]).unwrap().terminal, Terminal::Saved);
        for id in [ids[2], ids[3], ids[4]] {
            assert_eq!(
                coord.receipt(id).unwrap().terminal,
                Terminal::TriggerQueueOverflow
            );
        }
        assert_eq!(coord.drain_order(), ids.as_slice());
    }

    #[test]
    fn coordinator_success_feedback_cannot_fire_before_persist() {
        use std::cell::RefCell;
        use std::rc::Rc;

        #[derive(Default)]
        struct Order {
            persist_done: bool,
            feedback_before_persist: bool,
        }

        struct Persist(Rc<RefCell<Order>>);
        impl PersistHook for Persist {
            fn persist(&mut self, _request_id: u64) -> Result<(), PersistError> {
                self.0.borrow_mut().persist_done = true;
                Ok(())
            }
        }

        struct Feedback(Rc<RefCell<Order>>);
        impl FeedbackHook for Feedback {
            fn success(&mut self, _request_id: u64) {
                let mut order = self.0.borrow_mut();
                if !order.persist_done {
                    order.feedback_before_persist = true;
                }
            }
        }

        let order = Rc::new(RefCell::new(Order::default()));
        let mut coord =
            CaptureCoordinator::with_hooks(4, Persist(order.clone()), Feedback(order.clone()));
        coord.submit(snap(1));
        coord.drain();
        let order = order.borrow();
        assert!(order.persist_done);
        assert!(!order.feedback_before_persist);
        assert_eq!(coord.receipt(1).unwrap().terminal, Terminal::Saved);
    }

    #[test]
    fn coordinator_diagnostics_independent_of_persist_failure() {
        struct FailPersist;
        impl PersistHook for FailPersist {
            fn persist(&mut self, _request_id: u64) -> Result<(), PersistError> {
                Err(PersistError)
            }
        }
        struct FlagFeedback {
            fired: bool,
        }
        impl FeedbackHook for FlagFeedback {
            fn success(&mut self, _request_id: u64) {
                self.fired = true;
            }
        }
        let mut coord =
            CaptureCoordinator::with_hooks(4, FailPersist, FlagFeedback { fired: false });
        coord.submit(snap(1));
        coord.drain();
        assert_eq!(coord.receipt(1).unwrap().terminal, Terminal::Failed);
        assert_eq!(
            coord.diagnostics(),
            &[DiagnosticEvent {
                request_id: 1,
                terminal: Terminal::Failed,
            }]
        );
        assert!(!coord.feedback.fired);
    }

    #[test]
    fn coordinator_repeated_id_returns_same_terminal() {
        let mut coord = CaptureCoordinator::new(4);
        let id = coord.submit(snap(1));
        coord.drain();
        let first = coord.receipt(id).unwrap();
        coord.complete(id, Terminal::Failed);
        assert_eq!(coord.receipt(id).unwrap(), first);
        assert_eq!(first.terminal, Terminal::Saved);
    }

    #[test]
    fn coordinator_context_unavailable_is_terminal() {
        let mut coord = CaptureCoordinator::new(4);
        let id = coord.submit_unavailable();
        coord.drain();
        assert_eq!(
            coord.receipt(id).unwrap().terminal,
            Terminal::ContextUnavailable
        );
    }

    #[test]
    fn coordinator_zero_capacity_still_terminates_the_first_request() {
        let mut coord = CaptureCoordinator::new(0);
        let first = coord.submit(snap(1));
        let second = coord.submit(snap(2));
        coord.drain();
        assert_eq!(coord.receipt(first).unwrap().terminal, Terminal::Saved);
        assert_eq!(
            coord.receipt(second).unwrap().terminal,
            Terminal::TriggerQueueOverflow
        );
    }
}
