//! Local SmolLM2 title refine. Loads a hash-pinned GGUF only. No hub, no network.

mod infer;
mod prompt;
mod weights;

pub use infer::{refine_title, warmup};
pub use prompt::{
    clean_title, format_prompt, title_is_grounded, MAX_INPUT_CHARS, MAX_NEW_TOKENS, PROMPT_VERSION,
};
pub use weights::{
    set_weights_path, vendor_weights_path, verified_weights_path, verify_weights, WeightsError,
    FILENAME, HF_BASE_REPO, HF_GGUF_REPO, HF_REVISION, SHA256_HEX,
};
