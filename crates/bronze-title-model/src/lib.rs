//! Local title refine. Loads hash-pinned bundled GGUFs or one imported file. No hub, no HTTP.

mod bundle;
mod custom;
mod infer;
mod prompt;
mod status;
mod tiers;
mod weights;

pub use bundle::{stage_bundled_models, staged_model_filenames, vendor_dir, BUNDLE_MODELS_DIR};
pub use custom::{
    custom_gguf_path, custom_id_ok, custom_weights_path, display_name_from, gguf_magic_ok,
    import_custom_gguf, is_gguf_magic, set_custom_path, CustomGguf, CustomGgufError, GGUF_MAGIC,
    MAX_CUSTOM_BYTES,
};
pub use infer::{
    classify_refine_reject, classify_weights_error, desired_tier, refine_outcome, refine_tier,
    refine_title, request_custom, request_tier, should_attempt_refine, warmup, FallbackReason,
    RefineOutcome,
};
pub use prompt::{
    accept_refined_title, clean_title, format_prompt, format_prompt_for, raw_preview,
    take_generated_piece, title_echoes_opening, title_is_grounded, truncate_input, MAX_INPUT_CHARS,
    MAX_NEW_TOKENS, PROMPT_VERSION, SYSTEM_PROMPT,
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
