use crate::infer::FallbackReason;
use crate::tiers::TitleTier;
use std::sync::Mutex;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnginePhase {
    Idle,
    Loading,
    Hashing,
    Ready,
    Missing,
    Failed,
}

impl EnginePhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Loading => "loading",
            Self::Hashing => "hashing",
            Self::Ready => "ready",
            Self::Missing => "missing",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TitleEngineStatus {
    pub tier: TitleTier,
    pub phase: EnginePhase,
    pub reason: Option<FallbackReason>,
    pub bytes_read: Option<u64>,
    pub bytes_total: Option<u64>,
}

impl TitleEngineStatus {
    pub const fn idle() -> Self {
        Self {
            tier: TitleTier::Extractive,
            phase: EnginePhase::Idle,
            reason: None,
            bytes_read: None,
            bytes_total: None,
        }
    }

    const fn at(tier: TitleTier, phase: EnginePhase, reason: Option<FallbackReason>) -> Self {
        Self {
            tier,
            phase,
            reason,
            bytes_read: None,
            bytes_total: None,
        }
    }
}

pub fn note_read_progress(bytes_read: u64, bytes_total: u64) {
    if bytes_total == 0 {
        return;
    }
    observe_diag(&format!(
        "read progress bytes={bytes_read} total={bytes_total}"
    ));
}

type Listener = Box<dyn Fn(TitleEngineStatus) + Send + Sync>;

static SNAPSHOT: Mutex<TitleEngineStatus> = Mutex::new(TitleEngineStatus::idle());
static LISTENER: Mutex<Option<Listener>> = Mutex::new(None);

fn snapshot() -> std::sync::MutexGuard<'static, TitleEngineStatus> {
    SNAPSHOT
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn current_status() -> TitleEngineStatus {
    *snapshot()
}

pub fn subscribe_status(listener: impl Fn(TitleEngineStatus) + Send + Sync + 'static) {
    *LISTENER
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(Box::new(listener));
}

fn notify(next: TitleEngineStatus) {
    if let Some(listener) = LISTENER
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .as_ref()
    {
        listener(next);
    }
}

fn parse_read_progress(message: &str) -> Option<(u64, u64)> {
    let rest = message.strip_prefix("read progress bytes=")?;
    let (read, total) = rest.split_once(" total=")?;
    let read = read.parse().ok()?;
    let total = total.parse().ok()?;
    if total == 0 {
        return None;
    }
    Some((read, total))
}

pub fn apply_diag(current: TitleEngineStatus, message: &str) -> TitleEngineStatus {
    let message = message.trim();
    if let Some((bytes_read, bytes_total)) = parse_read_progress(message) {
        if !matches!(current.phase, EnginePhase::Loading | EnginePhase::Hashing) {
            return current;
        }
        return TitleEngineStatus {
            tier: current.tier,
            phase: current.phase,
            reason: None,
            bytes_read: Some(bytes_read),
            bytes_total: Some(bytes_total),
        };
    }
    if let Some(raw) = message.strip_prefix("switch scheduled tier=") {
        let Some(tier) = TitleTier::parse(raw) else {
            return current;
        };
        return if tier == TitleTier::Extractive {
            TitleEngineStatus::at(tier, EnginePhase::Idle, Some(FallbackReason::Extractive))
        } else {
            TitleEngineStatus::at(tier, EnginePhase::Loading, None)
        };
    }
    if message.starts_with("weights resolved") || message == "hash ok" {
        if matches!(current.phase, EnginePhase::Loading | EnginePhase::Hashing) {
            return TitleEngineStatus::at(current.tier, EnginePhase::Hashing, None);
        }
        return current;
    }
    if message == "model loaded" {
        if current.tier == TitleTier::Extractive {
            return TitleEngineStatus::at(
                TitleTier::Extractive,
                EnginePhase::Idle,
                Some(FallbackReason::Extractive),
            );
        }
        return TitleEngineStatus::at(current.tier, EnginePhase::Ready, None);
    }
    let Some(raw) = message.strip_prefix("fallback reason=") else {
        return current;
    };
    match raw {
        "extractive" => TitleEngineStatus::at(
            TitleTier::Extractive,
            EnginePhase::Idle,
            Some(FallbackReason::Extractive),
        ),
        "missing_weights" => TitleEngineStatus::at(
            current.tier,
            EnginePhase::Missing,
            Some(FallbackReason::MissingWeights),
        ),
        "bad_hash" | "timeout" | "unreadable" => {
            if current.phase == EnginePhase::Ready {
                return current;
            }
            let reason = match raw {
                "bad_hash" => FallbackReason::BadHash,
                "timeout" => FallbackReason::Timeout,
                _ => FallbackReason::Unreadable,
            };
            TitleEngineStatus::at(current.tier, EnginePhase::Failed, Some(reason))
        }
        _ => current,
    }
}

pub fn observe_diag(message: &str) {
    let next = {
        let mut guard = snapshot();
        let next = apply_diag(*guard, message);
        if next == *guard {
            return;
        }
        *guard = next;
        next
    };
    notify(next);
}

#[cfg(test)]
mod status_tests {
    use super::*;

    #[test]
    fn diag_stages_map_to_lifecycle() {
        let idle = TitleEngineStatus::idle();
        let loading = apply_diag(idle, "switch scheduled tier=qwen-05");
        assert_eq!(loading.tier, TitleTier::Qwen05);
        assert_eq!(loading.phase, EnginePhase::Loading);
        assert_eq!(loading.reason, None);

        let hashing = apply_diag(loading, "weights resolved source=vendor");
        assert_eq!(hashing.phase, EnginePhase::Hashing);
        assert_eq!(apply_diag(hashing, "hash ok").phase, EnginePhase::Hashing);

        let ready = apply_diag(hashing, "model loaded");
        assert_eq!(ready.phase, EnginePhase::Ready);
        assert_eq!(ready.tier, TitleTier::Qwen05);
        assert_eq!(ready.reason, None);

        assert_eq!(
            apply_diag(ready, "fallback reason=timeout").phase,
            EnginePhase::Ready
        );
        assert_eq!(
            apply_diag(ready, "refine attempted").phase,
            EnginePhase::Ready
        );
        assert_eq!(
            apply_diag(ready, "fallback reason=first_sentence").phase,
            EnginePhase::Ready
        );
        assert_eq!(
            apply_diag(ready, "fallback reason=empty").phase,
            EnginePhase::Ready
        );
        assert_eq!(
            apply_diag(ready, "fallback reason=ungrounded").phase,
            EnginePhase::Ready
        );
    }

    #[test]
    fn extractive_and_missing_and_load_failure() {
        let idle = apply_diag(
            TitleEngineStatus::idle(),
            "switch scheduled tier=extractive",
        );
        assert_eq!(idle.phase, EnginePhase::Idle);
        assert_eq!(idle.tier, TitleTier::Extractive);

        let loading = apply_diag(idle, "switch scheduled tier=smol-360");
        let missing = apply_diag(loading, "fallback reason=missing_weights");
        assert_eq!(missing.phase, EnginePhase::Missing);
        assert_eq!(missing.reason, Some(FallbackReason::MissingWeights));
        assert_eq!(missing.tier, TitleTier::Smol360);

        assert_eq!(
            apply_diag(loading, "fallback reason=bad_hash").reason,
            Some(FallbackReason::BadHash)
        );
        assert_eq!(
            apply_diag(loading, "fallback reason=timeout").reason,
            Some(FallbackReason::Timeout)
        );
        assert_eq!(
            apply_diag(loading, "fallback reason=unreadable").reason,
            Some(FallbackReason::Unreadable)
        );
    }

    #[test]
    fn read_progress_is_determinate_until_a_terminal_phase() {
        let loading = apply_diag(TitleEngineStatus::idle(), "switch scheduled tier=smol-360");
        assert_eq!(loading.bytes_read, None);
        assert_eq!(loading.bytes_total, None);

        let reading = apply_diag(loading, "read progress bytes=135295440 total=270590880");
        assert_eq!(reading.phase, EnginePhase::Loading);
        assert_eq!(reading.bytes_read, Some(135_295_440));
        assert_eq!(reading.bytes_total, Some(270_590_880));

        let hashing = apply_diag(reading, "hash ok");
        assert_eq!(hashing.phase, EnginePhase::Hashing);
        assert_eq!(hashing.bytes_read, None);
        assert_eq!(hashing.bytes_total, None);

        let ready = apply_diag(hashing, "model loaded");
        assert_eq!(ready.phase, EnginePhase::Ready);
        assert_eq!(ready.bytes_read, None);
        assert_eq!(
            apply_diag(ready, "read progress bytes=1 total=2").phase,
            EnginePhase::Ready
        );

        let missing = apply_diag(reading, "fallback reason=missing_weights");
        assert_eq!(missing.phase, EnginePhase::Missing);
        assert_ne!(missing.phase, EnginePhase::Loading);
        assert_eq!(missing.bytes_read, None);

        let unreadable = apply_diag(reading, "fallback reason=unreadable");
        assert_eq!(unreadable.phase, EnginePhase::Failed);
        assert_eq!(unreadable.reason, Some(FallbackReason::Unreadable));
        assert_eq!(unreadable.bytes_read, None);
        assert_eq!(unreadable.bytes_total, None);
    }

    #[test]
    fn phases_do_not_leak_paths() {
        for phase in [
            EnginePhase::Idle,
            EnginePhase::Loading,
            EnginePhase::Hashing,
            EnginePhase::Ready,
            EnginePhase::Missing,
            EnginePhase::Failed,
        ] {
            assert!(!phase.as_str().contains('/'));
        }
    }
}
