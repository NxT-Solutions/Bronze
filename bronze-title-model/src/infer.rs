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
use std::sync::mpsc::{self, Sender};
use std::sync::OnceLock;
use std::time::Duration;

const REFINE_TIMEOUT: Duration = Duration::from_millis(8000);

enum Job {
    Infer {
        body: String,
        reply: Sender<Option<String>>,
    },
}

static JOBS: OnceLock<Sender<Job>> = OnceLock::new();

struct Engine {
    backend: LlamaBackend,
    model: LlamaModel,
}

pub fn refine_title(body: &str) -> Option<String> {
    if body.trim().is_empty() || body.trim().chars().count() <= 40 {
        return None;
    }
    if verified_weights_path().is_err() {
        return None;
    }
    let (reply_tx, reply_rx) = mpsc::channel();
    sender()?
        .send(Job::Infer {
            body: body.to_string(),
            reply: reply_tx,
        })
        .ok()?;
    reply_rx.recv_timeout(REFINE_TIMEOUT).ok().flatten()
}

pub fn warmup() {
    if verified_weights_path().is_ok() {
        let _ = sender();
    }
}

fn sender() -> Option<Sender<Job>> {
    Some(
        JOBS.get_or_init(|| {
            let (tx, rx) = mpsc::channel();
            let _ = std::thread::Builder::new()
                .name("bronze-title-model".into())
                .spawn(move || {
                    let engine = load_engine();
                    while let Ok(job) = rx.recv() {
                        match job {
                            Job::Infer { body, reply } => {
                                let out = engine
                                    .as_ref()
                                    .ok()
                                    .and_then(|engine| infer_once(engine, &body));
                                let _ = reply.send(out);
                            }
                        }
                    }
                });
            tx
        })
        .clone(),
    )
}

fn load_engine() -> Result<Engine, WeightsError> {
    let path = verified_weights_path()?;
    send_logs_to_tracing(LogOptions::default().with_logs_enabled(false));
    let backend = LlamaBackend::init().map_err(|_| WeightsError::Unreadable)?;
    // CPU only: do not silently add Metal/JIT entitlements. llama-cpp-2 still
    // compiles Metal on Apple Silicon; n_gpu_layers(0) keeps inference on CPU.
    let params = LlamaModelParams::default().with_n_gpu_layers(0);
    let model = LlamaModel::load_from_file(&backend, &path, &params)
        .map_err(|_| WeightsError::Unreadable)?;
    Ok(Engine { backend, model })
}

fn infer_once(engine: &Engine, body: &str) -> Option<String> {
    let prompt = format_prompt(body);
    let ctx_params = LlamaContextParams::default()
        .with_n_ctx(NonZeroU32::new(1024))
        .with_n_threads(2)
        .with_n_threads_batch(2);
    let mut ctx = engine.model.new_context(&engine.backend, ctx_params).ok()?;
    let tokens = engine.model.str_to_token(&prompt, AddBos::Never).ok()?;
    if tokens.is_empty() || tokens.len() > 1000 {
        return None;
    }
    let mut batch = LlamaBatch::new(1024, 1);
    let last = i32::try_from(tokens.len()).ok()?.saturating_sub(1);
    for (i, token) in (0_i32..).zip(tokens) {
        batch.add(token, i, &[0], i == last).ok()?;
    }
    ctx.decode(&mut batch).ok()?;
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
            .ok()?;
        if piece.contains('\n') {
            raw.push_str(piece.split('\n').next().unwrap_or(""));
            break;
        }
        raw.push_str(&piece);
        batch.clear();
        batch.add(token, start + offset, &[0], true).ok()?;
        ctx.decode(&mut batch).ok()?;
    }
    let title = clean_title(&raw)?;
    title_is_grounded(body, &title).then_some(title)
}

#[cfg(test)]
mod infer_tests {
    use super::*;

    #[test]
    fn missing_or_bad_weights_return_none() {
        let missing = std::env::temp_dir().join("bronze-absent-title.gguf");
        let _ = std::fs::remove_file(&missing);
        assert_eq!(
            crate::weights::verify_weights(&missing),
            Err(crate::weights::WeightsError::Missing)
        );
        assert_eq!(
            refine_from_unverified(&missing, "The migration timeout is the real bug."),
            None
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
        std::thread::sleep(Duration::from_secs(12));
        let samples = [
            "Thanks for the note.\nThe migration timeout is the real bug in persist.\nPlease take a look when you can.",
            "We should move the invoice review to Thursday so finance can close the books before Friday.",
            "fn persist_selection() { let title = compact_title(body); store.insert(title); }",
            "Park me",
        ];
        for body in samples {
            let title = refine_title(body);
            println!(
                "body={:?}\nmodel={:?}\nfallback={}\n",
                body,
                title,
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
