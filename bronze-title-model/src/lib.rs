//! Local title refine. Loads a hash-pinned GGUF allow-list only. No hub, no network.

mod infer;
mod prompt;
mod status;
mod tiers;
mod weights;

pub use infer::{
    classify_refine_reject, classify_weights_error, desired_tier, refine_outcome, refine_title,
    request_tier, should_attempt_refine, warmup, FallbackReason, RefineOutcome,
};
pub use prompt::{
    clean_title, format_prompt, format_prompt_for, raw_preview, take_generated_piece,
    title_echoes_opening, title_is_grounded, MAX_INPUT_CHARS, MAX_NEW_TOKENS, PROMPT_VERSION,
};
pub use status::{
    apply_diag, current_status, observe_diag, subscribe_status, EnginePhase, TitleEngineStatus,
};
pub use tiers::{auto_pick_title_tier, TierSpec, TitleTier, GGUF_TIERS};
pub use weights::{
    any_candidate_file, candidate_paths, present_gguf_tiers, set_weights_dir, set_weights_path,
    vendor_weights_path, verified_weights, verified_weights_for, verified_weights_path,
    verify_weights, weights_present, weights_present_for, WeightsError, WeightsSource, FILENAME,
    HF_BASE_REPO, HF_GGUF_REPO, HF_REVISION, SHA256_HEX,
};

pub fn emit_diag(message: &str) {
    eprintln!("bronze-title: {message}");
    observe_diag(message);
}
