use crate::live_session::{physical_ram_bytes, LiveSession};
use crate::title_engines::secret_store;
use bronze_settings::{SettingsV1, TitleModelId};
use bronze_title_model::{
    auto_pick_title_tier, current_status, present_gguf_tiers, refine_outcome, refine_tier,
    subscribe_status, RefineOutcome, TitleEngineStatus, TitleTier,
};
use bronze_title_remote::{hosted_refine, ollama_refine, HostedProvider};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

pub const TITLE_ENGINE_STATUS_EVENT: &str = "title-engine-status";

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TitleEngineStatusDto {
    pub tier: String,
    pub phase: String,
    pub reason: Option<String>,
}

impl From<TitleEngineStatus> for TitleEngineStatusDto {
    fn from(status: TitleEngineStatus) -> Self {
        Self {
            tier: status.tier.as_str().into(),
            phase: status.phase.as_str().into(),
            reason: status.reason.map(|reason| reason.as_str().into()),
        }
    }
}

pub fn attach_title_engine_status(app: &AppHandle) {
    let handle = app.clone();
    subscribe_status(move |status| {
        let _ = handle.emit(
            TITLE_ENGINE_STATUS_EVENT,
            TitleEngineStatusDto::from(status),
        );
    });
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn title_engine_status() -> TitleEngineStatusDto {
    TitleEngineStatusDto::from(current_status())
}

pub fn schedule(app: &AppHandle, item_id: String, body: String) {
    if item_id.is_empty() || body.trim().is_empty() {
        bronze_title_model::emit_diag("fallback reason=empty");
        return;
    }
    bronze_title_model::emit_diag(&format!("refine scheduled chars={}", body.chars().count()));
    let handle = app.clone();
    let _ = std::thread::Builder::new()
        .name("bronze-item-title".into())
        .spawn(move || {
            let settings = handle
                .try_state::<std::sync::Mutex<LiveSession>>()
                .map(|state| {
                    state
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .settings()
                });
            let Some(settings) = settings else {
                return;
            };
            let RefineOutcome::Title(title) = resolve_title(&settings, &body) else {
                return;
            };
            let Some(state) = handle.try_state::<std::sync::Mutex<LiveSession>>() else {
                return;
            };
            let applied = {
                let mut session = state
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                session
                    .apply_refined_title(&item_id, &body, &title)
                    .unwrap_or(false)
            };
            if applied {
                bronze_title_model::emit_diag("refine applied");
                let _ = handle.emit("queue-changed", ());
            } else {
                bronze_title_model::emit_diag("fallback reason=stale_body");
            }
        });
}

pub fn warmup() {
    bronze_title_model::warmup();
}

fn resolve_title(settings: &SettingsV1, body: &str) -> RefineOutcome {
    match settings.general.title_model {
        TitleModelId::Unset | TitleModelId::Extractive => {
            RefineOutcome::Fallback(bronze_title_model::FallbackReason::Extractive)
        }
        TitleModelId::Smol135
        | TitleModelId::Smol360
        | TitleModelId::Qwen05
        | TitleModelId::Custom => refine_outcome(body),
        TitleModelId::Ollama => match ollama_refine(&settings.general.title_ollama_model, body) {
            Ok(title) => RefineOutcome::Title(title),
            Err(_) => bundled_fallback(body),
        },
        TitleModelId::HostedOpenai
        | TitleModelId::HostedAnthropic
        | TitleModelId::HostedOpenrouter => {
            if !settings.general.title_hosted_confirmed {
                return bundled_fallback(body);
            }
            let Some(provider) = HostedProvider::parse(settings.general.title_model.as_str())
            else {
                return bundled_fallback(body);
            };
            match hosted_refine(
                provider,
                &settings.general.title_hosted_base,
                body,
                secret_store().as_ref(),
            ) {
                Ok(title) => RefineOutcome::Title(title),
                Err(_) => bundled_fallback(body),
            }
        }
    }
}

fn bundled_fallback(body: &str) -> RefineOutcome {
    let tier = auto_pick_title_tier(physical_ram_bytes(), &present_gguf_tiers());
    if tier == TitleTier::Extractive {
        return RefineOutcome::Fallback(bronze_title_model::FallbackReason::Extractive);
    }
    refine_tier(tier, body)
}

#[cfg(test)]
mod title_refine_tests {
    use super::TitleEngineStatusDto;
    use bronze_title_remote::{parse_hosted_base, resolve_ollama_base, RemoteError};

    #[test]
    fn schedule_is_after_persist_not_in_tap() {
        let lib = include_str!("lib.rs");
        let persist = lib
            .split("fn persist_capture_request")
            .nth(1)
            .expect("persist");
        let persist_end = persist.find("\nmod ").unwrap_or(persist.len());
        let persist = &persist[..persist_end];
        let emit_at = persist.find("capture-result").expect("emit");
        let spawn_at = persist.find("title_refine::schedule").expect("schedule");
        assert!(emit_at < spawn_at);
        let pump = lib.split("fn start_capture_pump").nth(1).expect("pump");
        let pump_end = pump.find("\nfn ").unwrap_or(pump.len());
        assert!(!pump[..pump_end].contains("title_refine"));
        assert!(!pump[..pump_end].contains("refine_title"));
        assert!(!include_str!(
            "../../../../native/macos/BronzeNative/Sources/BronzeNative/EventTapEngine.swift"
        )
        .contains("refine_title"));
        assert!(!include_str!(
            "../../../../native/macos/BronzeNative/Sources/BronzeNative/EventTapEngine.swift"
        )
        .contains("ollama"));
    }

    #[test]
    fn schedule_logs_visible_fallback_reasons() {
        let src = include_str!("title_refine.rs");
        assert!(src.contains("bronze_title_model::emit_diag"));
        assert!(src.contains("refine scheduled chars="));
        assert!(src.contains("fallback reason=empty"));
        assert!(src.contains("refine applied"));
        let lib = include_str!("lib.rs");
        assert!(lib.contains("title_engine_status"));
        assert!(lib.contains("attach_title_engine_status"));
        assert!(src.contains("title-engine-status"));
        assert!(!include_str!(
            "../../../../native/macos/BronzeNative/Sources/BronzeNative/EventTapEngine.swift"
        )
        .contains("title-engine-status"));
        assert!(
            include_str!("../permissions/used-permissions.toml").contains("title_engine_status")
        );
    }

    #[test]
    fn resolver_never_hosts_on_local_failure() {
        let src = include_str!("title_refine.rs");
        let local = src.split("TitleModelId::Custom").nth(1).expect("custom");
        let local_end = local.find("TitleModelId::Ollama").expect("ollama arm");
        assert!(!local[..local_end].contains("hosted_refine"));
        assert!(!local[..local_end].contains("ollama_refine"));
        assert_eq!(
            resolve_ollama_base(Some("http://example.com:11434")),
            "http://127.0.0.1:11434"
        );
        assert_eq!(
            parse_hosted_base("https://10.0.0.8"),
            Err(RemoteError::BlockedHost)
        );
    }

    #[test]
    fn title_engine_status_dto_maps_lifecycle() {
        use bronze_title_model::{apply_diag, EnginePhase, TitleEngineStatus};
        let loading = apply_diag(TitleEngineStatus::idle(), "switch scheduled tier=qwen-05");
        let dto = TitleEngineStatusDto::from(loading);
        assert_eq!(dto.tier, "qwen-05");
        assert_eq!(dto.phase, "loading");
        assert_eq!(dto.reason, None);
        assert_eq!(
            TitleEngineStatusDto::from(apply_diag(loading, "hash ok")).phase,
            "hashing"
        );
        let ready = apply_diag(loading, "model loaded");
        assert_eq!(ready.phase, EnginePhase::Ready);
        assert_eq!(TitleEngineStatusDto::from(ready).phase, "ready");
        assert_eq!(
            TitleEngineStatusDto::from(apply_diag(loading, "fallback reason=missing_weights"))
                .reason
                .as_deref(),
            Some("missing_weights")
        );
        assert_eq!(
            TitleEngineStatusDto::from(apply_diag(ready, "fallback reason=timeout")).phase,
            "ready"
        );
        assert!(!dto.tier.contains('/'));
    }
}
