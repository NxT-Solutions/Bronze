//! Local SmolLM2 title refine. Loads a hash-pinned GGUF only. No hub, no network.

mod infer;
mod prompt;
mod weights;

pub use infer::{
    classify_weights_error, refine_outcome, refine_title, should_attempt_refine, warmup,
    FallbackReason, RefineOutcome,
};
pub use prompt::{
    clean_title, format_prompt, title_is_grounded, MAX_INPUT_CHARS, MAX_NEW_TOKENS, PROMPT_VERSION,
};
pub use weights::{
    any_candidate_file, candidate_paths, set_weights_path, vendor_weights_path, verified_weights,
    verified_weights_path, verify_weights, weights_present, WeightsError, WeightsSource, FILENAME,
    HF_BASE_REPO, HF_GGUF_REPO, HF_REVISION, SHA256_HEX,
};

pub fn emit_diag(message: &str) {
    eprintln!("bronze-title: {message}");
}
