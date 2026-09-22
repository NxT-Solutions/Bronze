use crate::prompt::{clean_title, format_prompt, title_is_grounded, MAX_NEW_TOKENS};
use crate::weights::{verified_weights_path, WeightsError};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;
use llama_cpp_2::{send_logs_to_tracing, LogOptions};
use std::num::NonZeroU32;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::OnceLock;
use std::time::Duration;

const GENERATE_TIMEOUT: Duration = Duration::from_millis(8000);
const LOAD_TIMEOUT: Duration = Duration::from_secs(45);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FallbackReason {
    MissingWeights,
    BadHash,
    Unreadable,
    Timeout,
    ShortBody,
    Ungrounded,
    Empty,
}

impl FallbackReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MissingWeights => "missing_weights",
            Self::BadHash => "bad_hash",
            Self::Unreadable => "unreadable",
            Self::Timeout => "timeout",
            Self::ShortBody => "short_body",
            Self::Ungrounded => "ungrounded",
            Self::Empty => "empty",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RefineOutcome {
    Title(String),
    Fallback(FallbackReason),
}

enum Job {
    Infer {
        body: String,
        reply: Sender<InferReply>,
    },
}

enum InferReply {
    Title(String),
    Fallback(FallbackReason),
}

static JOBS: OnceLock<Sender<Job>> = OnceLock::new();
static ENGINE_READY: AtomicBool = AtomicBool::new(false);

pub fn classify_weights_error(err: WeightsError) -> FallbackReason {
    match err {
        WeightsError::Missing => FallbackReason::MissingWeights,
        WeightsError::HashMismatch => FallbackReason::BadHash,
        WeightsError::Unreadable => FallbackReason::Unreadable,
    }
}

pub fn should_attempt_refine(
    body: &str,
    weights: Result<(), WeightsError>,
) -> Result<(), FallbackReason> {
    if body.trim().chars().count() <= 40 {
        return Err(FallbackReason::ShortBody);
    }
    weights.map_err(classify_weights_error)
}

pub fn refine_title(body: &str) -> Option<String> {
    match refine_outcome(body) {
        RefineOutcome::Title(title) => Some(title),
        RefineOutcome::Fallback(_) => None,
    }
}

pub fn refine_outcome(body: &str) -> RefineOutcome {
    if let Err(reason) = should_attempt_refine(body, crate::weights::weights_present()) {
        crate::emit_diag(&format!("fallback reason={}", reason.as_str()));
        return RefineOutcome::Fallback(reason);
    }
    crate::emit_diag("refine attempted");
    let (reply_tx, reply_rx) = mpsc::channel();
    if sender()
        .send(Job::Infer {
            body: body.to_string(),
            reply: reply_tx,
        })
        .is_err()
    {
        crate::emit_diag("fallback reason=unreadable");
        return RefineOutcome::Fallback(FallbackReason::Unreadable);
    }
    let wait = if ENGINE_READY.load(Ordering::Relaxed) {
        GENERATE_TIMEOUT
    } else {
        LOAD_TIMEOUT.saturating_add(GENERATE_TIMEOUT)
    };
    match reply_rx.recv_timeout(wait) {
        Ok(InferReply::Title(title)) => {
            crate::emit_diag("refine generated");
            RefineOutcome::Title(title)
        }
        Ok(InferReply::Fallback(reason)) => {
            crate::emit_diag(&format!("fallback reason={}", reason.as_str()));
            RefineOutcome::Fallback(reason)
        }
        Err(_) => {
            crate::emit_diag("fallback reason=timeout");
            RefineOutcome::Fallback(FallbackReason::Timeout)
        }
    }
}

pub fn warmup() {
    if crate::weights::any_candidate_file() {
        let _ = sender();
    } else {
        crate::emit_diag("fallback reason=missing_weights");
    }
}

fn sender() -> Sender<Job> {
    JOBS.get_or_init(|| {
        let (tx, rx) = mpsc::channel();
        let _ = std::thread::Builder::new()
            .name("bronze-title-model".into())
            .spawn(move || worker_loop(rx));
        tx
    })
    .clone()
}

fn worker_loop(rx: Receiver<Job>) {
    let mut engine = match load_engine() {
        Ok(loaded) => {
            crate::emit_diag("model loaded");
            ENGINE_READY.store(true, Ordering::Relaxed);
            Some(loaded)
        }
        Err(_) => None,
    };
    while let Ok(job) = rx.recv() {
        match job {
            Job::Infer { body, reply } => {
                if engine.is_none() {
                    match load_engine() {
                        Ok(loaded) => {
                            crate::emit_diag("model loaded");
                            ENGINE_READY.store(true, Ordering::Relaxed);
                            engine = Some(loaded);
                        }
                        Err(reason) => {
                            let _ = reply.send(InferReply::Fallback(reason));
                            continue;
                        }
                    }
                }
                let Some(loaded) = engine.as_ref() else {
                    let _ = reply.send(InferReply::Fallback(FallbackReason::Unreadable));
                    continue;
                };
                let out = match infer_once(loaded, &body) {
                    Ok(title) => InferReply::Title(title),
                    Err(reason) => InferReply::Fallback(reason),
                };
                let _ = reply.send(out);
            }
        }
    }
}

fn load_engine() -> Result<Engine, FallbackReason> {
    let path = verified_weights_path().map_err(classify_weights_error)?;
    send_logs_to_tracing(LogOptions::default().with_logs_enabled(false));
    let backend = LlamaBackend::init().map_err(|_| FallbackReason::Unreadable)?;
    // CPU only: do not silently add Metal/JIT entitlements. llama-cpp-2 still
    // compiles Metal on Apple Silicon; n_gpu_layers(0) keeps inference on CPU.
    let params = LlamaModelParams::default().with_n_gpu_layers(0);
    let model = LlamaModel::load_from_file(&backend, &path, &params)
        .map_err(|_| FallbackReason::Unreadable)?;
    Ok(Engine { backend, model })
}

struct Engine {
    backend: LlamaBackend,
    model: LlamaModel,
}

fn infer_once(engine: &Engine, body: &str) -> Result<String, FallbackReason> {
    let prompt = format_prompt(body);
    let ctx_params = LlamaContextParams::default()
        .with_n_ctx(NonZeroU32::new(1024))
        .with_n_threads(2)
        .with_n_threads_batch(2);
    let mut ctx = engine
        .model
        .new_context(&engine.backend, ctx_params)
        .map_err(|_| FallbackReason::Unreadable)?;
    let tokens = engine
        .model
        .str_to_token(&prompt, AddBos::Never)
        .map_err(|_| FallbackReason::Empty)?;
    if tokens.is_empty() || tokens.len() > 1000 {
        return Err(FallbackReason::Empty);
    }
    let mut batch = LlamaBatch::new(1024, 1);
    let last = i32::try_from(tokens.len())
        .map_err(|_| FallbackReason::Empty)?
        .saturating_sub(1);
    for (i, token) in (0_i32..).zip(tokens) {
        batch
            .add(token, i, &[0], i == last)
            .map_err(|_| FallbackReason::Empty)?;
    }
    ctx.decode(&mut batch).map_err(|_| FallbackReason::Empty)?;
    let mut sampler = LlamaSampler::greedy();
    let start = batch.n_tokens();
    let mut raw = String::new();
    let mut decoder = encoding_rs::UTF_8.new_decoder();
    for offset in 0..MAX_NEW_TOKENS {
        let token = sampler.sample(&ctx, batch.n_tokens() - 1);
        sampler.accept(token);
        if engine.model.is_eog_token(token) {
            break;
        }
        let piece = engine
            .model
            .token_to_piece(token, &mut decoder, true, None)
            .map_err(|_| FallbackReason::Empty)?;
        if piece.contains('\n') {
            raw.push_str(piece.split('\n').next().unwrap_or(""));
            break;
        }
        raw.push_str(&piece);
        batch.clear();
        batch
            .add(token, start + offset, &[0], true)
            .map_err(|_| FallbackReason::Empty)?;
        ctx.decode(&mut batch).map_err(|_| FallbackReason::Empty)?;
    }
    let Some(title) = clean_title(&raw) else {
        return Err(FallbackReason::Empty);
    };
    if title_is_grounded(body, &title) {
        Ok(title)
    } else {
        Err(FallbackReason::Ungrounded)
    }
}

#[cfg(test)]
mod infer_tests {
    use super::*;

    const LONG_BODY: &str = "Thanks for the note.\nThe migration timeout is the real bug in persist.\nPlease take a look when you can.";

    #[test]
    fn missing_weights_records_reason_and_keeps_compact_title() {
        let missing = std::env::temp_dir().join("bronze-absent-title.gguf");
        let _ = std::fs::remove_file(&missing);
        assert_eq!(
            crate::weights::verify_weights(&missing),
            Err(crate::weights::WeightsError::Missing)
        );
        assert_eq!(
            should_attempt_refine(LONG_BODY, Err(crate::weights::WeightsError::Missing)),
            Err(FallbackReason::MissingWeights)
        );
        assert_eq!(refine_from_unverified(&missing, LONG_BODY), None);
        let compact = bronze_domain::compact_title(LONG_BODY);
        assert!(compact.to_ascii_lowercase().contains("migration"));
        assert!(compact.chars().count() <= 40);
    }

    #[test]
    fn vendor_or_fixture_path_attempts_refine() {
        let path = crate::weights::vendor_weights_path();
        let weights = crate::weights::verify_weights(&path);
        if path.is_file() {
            assert_eq!(weights, Ok(()));
            assert_eq!(should_attempt_refine(LONG_BODY, weights), Ok(()));
            assert_eq!(
                should_attempt_refine("Park me", weights),
                Err(FallbackReason::ShortBody)
            );
        } else {
            assert_eq!(weights, Err(crate::weights::WeightsError::Missing));
            assert_eq!(
                should_attempt_refine(LONG_BODY, weights),
                Err(FallbackReason::MissingWeights)
            );
        }
    }

    #[test]
    fn short_body_is_skipped() {
        assert_eq!(
            should_attempt_refine("Park me", Ok(())),
            Err(FallbackReason::ShortBody)
        );
        assert_eq!(
            classify_weights_error(crate::weights::WeightsError::HashMismatch),
            FallbackReason::BadHash
        );
    }

    fn refine_from_unverified(path: &std::path::Path, body: &str) -> Option<String> {
        if crate::weights::verify_weights(path).is_err() {
            return None;
        }
        refine_title(body)
    }

    #[test]
    #[ignore]
    fn spike_smollm2_title_quality() {
        warmup();
        std::thread::sleep(Duration::from_secs(2));
        let samples = [
            "Thanks for the note.\nThe migration timeout is the real bug in persist.\nPlease take a look when you can.",
            "We should move the invoice review to Thursday so finance can close the books before Friday.",
            "fn persist_selection() { let title = compact_title(body); store.insert(title); }",
            "Park me",
            "The local title model is in. Capture still saves compact_title immediately; a Rust worker then tries SmolLM2 offline and only replaces the title if the body is unchanged and the title is grounded.",
        ];
        for body in samples {
            let started = std::time::Instant::now();
            let outcome = refine_outcome(body);
            println!(
                "chars={} elapsed_ms={} outcome={:?} compact={:?}\n",
                body.chars().count(),
                started.elapsed().as_millis(),
                outcome,
                bronze_domain::compact_title(body)
            );
        }
    }

    #[test]
    fn worker_source_has_no_network() {
        let infer = include_str!("infer.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("prod");
        let weights = include_str!("weights.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("prod");
        let prompt = include_str!("prompt.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("prod");
        let lib = include_str!("lib.rs");
        for src in [infer, weights, prompt, lib] {
            let lower = src.to_ascii_lowercase();
            for needle in [
                "huggingface.co",
                "hf-hub",
                "hf_hub",
                "reqwest",
                "ureq",
                "openai",
                "download(",
            ] {
                assert!(!lower.contains(needle), "title worker mentions {needle}");
            }
        }
    }
}
