use crate::live_session::LiveSession;
use bronze_title_model::RefineOutcome;
use tauri::{AppHandle, Emitter, Manager};

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
            let RefineOutcome::Title(title) = bronze_title_model::refine_outcome(&body) else {
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

#[cfg(test)]
mod title_refine_tests {
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
    }

    #[test]
    fn schedule_logs_visible_fallback_reasons() {
        let src = include_str!("title_refine.rs");
        assert!(src.contains("bronze_title_model::emit_diag"));
        assert!(src.contains("refine scheduled chars="));
        assert!(src.contains("fallback reason=empty"));
        assert!(src.contains("refine applied"));
    }
}
