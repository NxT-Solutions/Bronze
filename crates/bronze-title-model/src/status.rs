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
}

impl TitleEngineStatus {
    pub const fn idle() -> Self {
        Self {
            tier: TitleTier::Extractive,
            phase: EnginePhase::Idle,
            reason: None,
        }
    }
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

fn publish(next: TitleEngineStatus) {
    *snapshot() = next;
    if let Some(listener) = LISTENER
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .as_ref()
    {
        listener(next);
    }
}

pub fn apply_diag(current: TitleEngineStatus, message: &str) -> TitleEngineStatus {
    let message = message.trim();
    if let Some(raw) = message.strip_prefix("switch scheduled tier=") {
        let Some(tier) = TitleTier::parse(raw) else {
            return current;
        };
        return if tier == TitleTier::Extractive {
            TitleEngineStatus {
                tier,
                phase: EnginePhase::Idle,
                reason: Some(FallbackReason::Extractive),
            }
        } else {
            TitleEngineStatus {
                tier,
                phase: EnginePhase::Loading,
                reason: None,
            }
        };
    }
    if message.starts_with("weights resolved") || message == "hash ok" {
        if matches!(current.phase, EnginePhase::Loading | EnginePhase::Hashing) {
            return TitleEngineStatus {
                tier: current.tier,
                phase: EnginePhase::Hashing,
                reason: None,
            };
        }
        return current;
    }
    if message == "model loaded" {
        if current.tier == TitleTier::Extractive {
            return TitleEngineStatus {
                tier: TitleTier::Extractive,
                phase: EnginePhase::Idle,
                reason: Some(FallbackReason::Extractive),
            };
        }
        return TitleEngineStatus {
            tier: current.tier,
            phase: EnginePhase::Ready,
            reason: None,
        };
    }
    let Some(raw) = message.strip_prefix("fallback reason=") else {
        return current;
    };
    match raw {
        "extractive" => TitleEngineStatus {
            tier: TitleTier::Extractive,
            phase: EnginePhase::Idle,
            reason: Some(FallbackReason::Extractive),
        },
        "missing_weights" => TitleEngineStatus {
            tier: current.tier,
            phase: EnginePhase::Missing,
            reason: Some(FallbackReason::MissingWeights),
        },
        "bad_hash" | "timeout" | "unreadable" => {
            if current.phase == EnginePhase::Ready {
                return current;
            }
            let reason = match raw {
                "bad_hash" => FallbackReason::BadHash,
                "timeout" => FallbackReason::Timeout,
                _ => FallbackReason::Unreadable,
            };
            TitleEngineStatus {
                tier: current.tier,
                phase: EnginePhase::Failed,
                reason: Some(reason),
            }
        }
        _ => current,
    }
}

pub fn observe_diag(message: &str) {
    let current = current_status();
    let next = apply_diag(current, message);
    if next != current {
        publish(next);
    }
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
