use crate::custom::gguf_magic_ok;
use crate::prompt::{
    accept_refined_title, clean_title, format_prompt_for, raw_preview, take_generated_piece,
    title_echoes_opening, title_is_grounded, MAX_NEW_TOKENS,
};
use crate::tiers::TitleTier;
use crate::weights::{verified_weights_for, WeightsError};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;
use llama_cpp_2::{send_logs_to_tracing, LogOptions};
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Mutex, OnceLock};
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
    FirstSentence,
    Empty,
    Extractive,
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
            Self::FirstSentence => "first_sentence",
            Self::Empty => "empty",
            Self::Extractive => "extractive",
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
    InferTier {
        tier: TitleTier,
        body: String,
        reply: Sender<InferReply>,
    },
    SetModel {
        tier: TitleTier,
    },
}

enum InferReply {
    Title(String),
    Fallback(FallbackReason),
}

static JOBS: OnceLock<Sender<Job>> = OnceLock::new();
static ENGINE_READY: AtomicBool = AtomicBool::new(false);
static DESIRED: Mutex<TitleTier> = Mutex::new(TitleTier::Extractive);

pub fn classify_weights_error(err: WeightsError) -> FallbackReason {
    match err {
        WeightsError::Missing => FallbackReason::MissingWeights,
        WeightsError::HashMismatch => FallbackReason::BadHash,
        WeightsError::Unreadable => FallbackReason::Unreadable,
    }
}

pub fn desired_tier() -> TitleTier {
    *DESIRED
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn request_tier(tier: TitleTier) {
    if tier != TitleTier::Custom {
        crate::custom::set_custom_path(None);
    }
    {
        let mut desired = DESIRED
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *desired = tier;
    }
    crate::emit_diag(&format!("switch scheduled tier={}", tier.as_str()));
    enqueue_set_model(tier);
}

fn enqueue_set_model(tier: TitleTier) {
    if sender().send(Job::SetModel { tier }).is_err() {
        crate::emit_diag("fallback reason=unreadable");
    }
}

pub fn request_custom(path: PathBuf) {
    crate::custom::set_custom_path(Some(path));
    request_tier(TitleTier::Custom);
}

pub fn should_attempt_refine(
    body: &str,
    weights: Result<(), WeightsError>,
) -> Result<(), FallbackReason> {
    should_attempt_refine_for(desired_tier(), body, weights)
}

fn should_attempt_refine_for(
    tier: TitleTier,
    body: &str,
    weights: Result<(), WeightsError>,
) -> Result<(), FallbackReason> {
    if tier == TitleTier::Extractive {
        return Err(FallbackReason::Extractive);
    }
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
    let tier = desired_tier();
    dispatch_infer(body, tier, InferKind::Desired)
}

pub fn refine_tier(tier: TitleTier, body: &str) -> RefineOutcome {
    if tier == TitleTier::Extractive || tier == TitleTier::Custom {
        crate::emit_diag("fallback reason=extractive");
        return RefineOutcome::Fallback(FallbackReason::Extractive);
    }
    dispatch_infer(body, tier, InferKind::Bundled(tier))
}

enum InferKind {
    Desired,
    Bundled(TitleTier),
}

fn dispatch_infer(body: &str, tier: TitleTier, kind: InferKind) -> RefineOutcome {
    if let Err(reason) =
        should_attempt_refine_for(tier, body, crate::weights::weights_present_for(tier))
    {
        crate::emit_diag(&format!("fallback reason={}", reason.as_str()));
        return RefineOutcome::Fallback(reason);
    }
    crate::emit_diag("refine attempted");
    let (reply_tx, reply_rx) = mpsc::channel();
    let job = match kind {
        InferKind::Desired => Job::Infer {
            body: body.to_string(),
            reply: reply_tx,
        },
        InferKind::Bundled(tier) => Job::InferTier {
            tier,
            body: body.to_string(),
            reply: reply_tx,
        },
    };
    if sender().send(job).is_err() {
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
    let tier = desired_tier();
    if tier == TitleTier::Extractive {
        crate::emit_diag("fallback reason=extractive");
        return;
    }
    if crate::weights::weights_present_for(tier).is_ok() {
        enqueue_set_model(tier);
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

struct WorkerState {
    backend: Option<LlamaBackend>,
    model: Option<LlamaModel>,
    loaded: Option<TitleTier>,
}

fn worker_loop(rx: Receiver<Job>) {
    let mut state = WorkerState {
        backend: None,
        model: None,
        loaded: None,
    };
    while let Ok(job) = rx.recv() {
        let ran = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run_job(&mut state, job);
        }));
        if ran.is_err() {
            state.backend = None;
            state.model = None;
            state.loaded = None;
            ENGINE_READY.store(false, Ordering::Relaxed);
            crate::emit_diag("fallback reason=unreadable");
        }
    }
}

fn run_job(state: &mut WorkerState, job: Job) {
    match job {
        Job::SetModel { tier } => {
            let _ = apply_tier(state, tier);
        }
        Job::Infer { body, reply } => {
            run_infer_job(state, desired_tier(), body, reply);
        }
        Job::InferTier { tier, body, reply } => {
            run_infer_job(state, tier, body, reply);
        }
    }
}

fn run_infer_job(
    state: &mut WorkerState,
    tier: TitleTier,
    body: String,
    reply: Sender<InferReply>,
) {
    if apply_tier(state, tier).is_err() {
        let reason =
            match should_attempt_refine_for(tier, &body, crate::weights::weights_present_for(tier))
            {
                Err(reason) => reason,
                Ok(()) => FallbackReason::Unreadable,
            };
        let _ = reply.send(InferReply::Fallback(reason));
        return;
    }
    let Some(model) = state.model.as_ref() else {
        let _ = reply.send(InferReply::Fallback(FallbackReason::Extractive));
        return;
    };
    let Some(backend) = state.backend.as_ref() else {
        let _ = reply.send(InferReply::Fallback(FallbackReason::Unreadable));
        return;
    };
    let loaded = state.loaded.unwrap_or(tier);
    let out = match infer_once(backend, model, loaded, &body) {
        Ok(title) => InferReply::Title(title),
        Err(reason) => InferReply::Fallback(reason),
    };
    let _ = reply.send(out);
}

fn apply_tier(state: &mut WorkerState, tier: TitleTier) -> Result<(), FallbackReason> {
    if state.loaded == Some(tier) && (tier == TitleTier::Extractive || state.model.is_some()) {
        if tier == TitleTier::Extractive {
            crate::emit_diag("fallback reason=extractive");
        } else {
            crate::emit_diag("model loaded");
        }
        return Ok(());
    }
    state.model = None;
    state.loaded = Some(tier);
    ENGINE_READY.store(false, Ordering::Relaxed);
    if tier == TitleTier::Extractive {
        crate::emit_diag("fallback reason=extractive");
        return Ok(());
    }
    match load_model(state, tier) {
        Ok(()) => {
            crate::emit_diag("model loaded");
            ENGINE_READY.store(true, Ordering::Relaxed);
            Ok(())
        }
        Err(reason) => {
            state.model = None;
            state.loaded = Some(TitleTier::Extractive);
            crate::emit_diag(&format!("fallback reason={}", reason.as_str()));
            Err(reason)
        }
    }
}

fn load_model(state: &mut WorkerState, tier: TitleTier) -> Result<(), FallbackReason> {
    let path = if tier == TitleTier::Custom {
        let path = crate::custom::custom_weights_path().ok_or(FallbackReason::MissingWeights)?;
        if !gguf_magic_ok(&path) {
            return Err(FallbackReason::Unreadable);
        }
        crate::weights::read_file_with_progress(&path).map_err(classify_weights_error)?;
        crate::emit_diag("weights resolved source=custom");
        path
    } else {
        verified_weights_for(tier)
            .map_err(classify_weights_error)?
            .0
    };
    send_logs_to_tracing(LogOptions::default().with_logs_enabled(false));
    if state.backend.is_none() {
        state.backend = Some(LlamaBackend::init().map_err(|_| FallbackReason::Unreadable)?);
    }
    let backend = state.backend.as_ref().ok_or(FallbackReason::Unreadable)?;
    // CPU only: do not silently add Metal/JIT entitlements. llama-cpp-2 still
    // compiles Metal on Apple Silicon; n_gpu_layers(0) keeps inference on CPU.
    let params = LlamaModelParams::default().with_n_gpu_layers(0);
    let model = LlamaModel::load_from_file(backend, &path, &params)
        .map_err(|_| FallbackReason::Unreadable)?;
    state.model = Some(model);
    state.loaded = Some(tier);
    Ok(())
}

pub fn classify_refine_reject(body: &str, raw: &str) -> FallbackReason {
    match clean_title(raw) {
        None => FallbackReason::Empty,
        Some(title) if !title_is_grounded(body, &title) => FallbackReason::Ungrounded,
        Some(title) if title_echoes_opening(body, &title) => FallbackReason::FirstSentence,
        Some(_) => FallbackReason::Empty,
    }
}

fn emit_raw_preview(raw: &str) {
    crate::emit_diag(&format!(
        "fallback raw_len={} raw_preview={}",
        raw.chars().count(),
        raw_preview(raw)
    ));
}

fn infer_once(
    backend: &LlamaBackend,
    model: &LlamaModel,
    tier: TitleTier,
    body: &str,
) -> Result<String, FallbackReason> {
    let prompt = format_prompt_for(tier, body);
    let ctx_params = LlamaContextParams::default()
        .with_n_ctx(NonZeroU32::new(1024))
        .with_n_threads(2)
        .with_n_threads_batch(2);
    let mut ctx = model
        .new_context(backend, ctx_params)
        .map_err(|_| FallbackReason::Unreadable)?;
    let tokens = model
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
        if model.is_eog_token(token) {
            break;
        }
        let piece = model
            .token_to_piece(token, &mut decoder, true, None)
            .map_err(|_| FallbackReason::Empty)?;
        if take_generated_piece(&mut raw, &piece) {
            break;
        }
        batch.clear();
        batch
            .add(token, start + offset, &[0], true)
            .map_err(|_| FallbackReason::Empty)?;
        ctx.decode(&mut batch).map_err(|_| FallbackReason::Empty)?;
    }
    match accept_refined_title(body, &raw) {
        Some(title) => Ok(title),
        None => {
            let reason = classify_refine_reject(body, &raw);
            emit_raw_preview(&raw);
            Err(reason)
        }
    }
}

#[cfg(test)]
mod infer_tests {
    use super::*;

    const LONG_BODY: &str = "Thanks for the note.\nThe migration timeout is the real bug in persist.\nPlease take a look when you can.";

    fn hold_desired_tier() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: Mutex<()> = Mutex::new(());
        LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[test]
    fn missing_weights_records_reason_and_keeps_compact_title() {
        let _guard = hold_desired_tier();
        request_tier(TitleTier::Qwen05);
        let missing = std::env::temp_dir().join("bronze-absent-title.gguf");
        let _ = std::fs::remove_file(&missing);
        assert_eq!(
            crate::weights::verify_weights(&missing),
            Err(crate::weights::WeightsError::Missing)
        );
        assert_eq!(
            should_attempt_refine_for(
                TitleTier::Qwen05,
                LONG_BODY,
                Err(crate::weights::WeightsError::Missing)
            ),
            Err(FallbackReason::MissingWeights)
        );
        assert_eq!(refine_from_unverified(&missing, LONG_BODY), None);
        let compact = bronze_domain::compact_title(LONG_BODY);
        assert!(compact.to_ascii_lowercase().contains("migration"));
        assert!(compact.chars().count() <= 40);
    }

    #[test]
    fn extractive_skips_gguf_and_switch_schedules_reload() {
        let _guard = hold_desired_tier();
        request_tier(TitleTier::Extractive);
        assert_eq!(desired_tier(), TitleTier::Extractive);
        assert_eq!(
            should_attempt_refine(LONG_BODY, Ok(())),
            Err(FallbackReason::Extractive)
        );
        let infer = include_str!("infer.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("prod");
        assert!(infer.contains("Job::SetModel"));
        assert!(infer.contains("switch scheduled tier="));
        let request = infer
            .split("pub fn request_tier")
            .nth(1)
            .expect("request_tier");
        let request = request
            .split("fn enqueue_set_model")
            .next()
            .expect("enqueue");
        assert!(request.contains("enqueue_set_model(tier)"));
        assert!(!request.contains("JOBS.get()"));
        let enqueue = infer
            .split("fn enqueue_set_model")
            .nth(1)
            .expect("enqueue body");
        let enqueue = enqueue
            .split("pub fn request_custom")
            .next()
            .expect("custom");
        assert!(enqueue.contains("sender().send(Job::SetModel"));
        request_tier(TitleTier::Smol360);
        assert_eq!(desired_tier(), TitleTier::Smol360);
    }

    #[test]
    fn switch_without_prior_warmup_leaves_loading() {
        let _guard = hold_desired_tier();
        let missing =
            std::env::temp_dir().join(format!("bronze-absent-switch-{}.gguf", std::process::id()));
        let _ = std::fs::remove_file(&missing);
        request_custom(missing);
        let status = wait_for_settled_phase();
        assert_ne!(status.phase, crate::EnginePhase::Loading);
        assert_ne!(status.phase, crate::EnginePhase::Hashing);
        assert!(
            matches!(
                status.phase,
                crate::EnginePhase::Failed | crate::EnginePhase::Missing
            ),
            "{}",
            status.phase.as_str()
        );
        request_tier(TitleTier::Extractive);
        let idle = wait_for_settled_phase();
        assert_eq!(idle.phase, crate::EnginePhase::Idle);
        assert_eq!(desired_tier(), TitleTier::Extractive);
    }

    fn wait_for_settled_phase() -> crate::TitleEngineStatus {
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        loop {
            let status = crate::current_status();
            if !matches!(
                status.phase,
                crate::EnginePhase::Loading | crate::EnginePhase::Hashing
            ) {
                return status;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "title engine stayed on {}",
                status.phase.as_str()
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn vendor_or_fixture_path_attempts_refine() {
        let _guard = hold_desired_tier();
        request_tier(TitleTier::Smol360);
        let path = crate::weights::vendor_weights_path();
        let weights = crate::weights::verify_weights(&path);
        if path.is_file() {
            assert_eq!(weights, Ok(()));
            assert_eq!(
                should_attempt_refine_for(TitleTier::Smol360, LONG_BODY, weights),
                Ok(())
            );
            assert_eq!(
                should_attempt_refine_for(TitleTier::Smol360, "Park me", weights),
                Err(FallbackReason::ShortBody)
            );
        } else {
            assert_eq!(weights, Err(crate::weights::WeightsError::Missing));
            assert_eq!(
                should_attempt_refine_for(TitleTier::Smol360, LONG_BODY, weights),
                Err(FallbackReason::MissingWeights)
            );
        }
    }

    #[test]
    fn bundled_fallback_skips_extractive_and_custom() {
        assert_eq!(
            refine_tier(TitleTier::Extractive, LONG_BODY),
            RefineOutcome::Fallback(FallbackReason::Extractive)
        );
        assert_eq!(
            refine_tier(TitleTier::Custom, LONG_BODY),
            RefineOutcome::Fallback(FallbackReason::Extractive)
        );
    }

    #[test]
    fn short_body_is_skipped() {
        assert_eq!(
            should_attempt_refine_for(TitleTier::Smol360, "Park me", Ok(())),
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
    fn reject_reasons_distinguish_empty_echo_and_ungrounded() {
        const FOLLO: &str = "The landing page change did most of the work for the Follo billing investigation. Query performance impact from D7CEC1E versus the previous plan still needs a number before we sign off. Invoice review on Thursday is the remaining close item.";
        assert_eq!(classify_refine_reject(FOLLO, ""), FallbackReason::Empty);
        assert_eq!(
            classify_refine_reject(FOLLO, "   \n"),
            FallbackReason::Empty
        );
        assert_eq!(
            classify_refine_reject(FOLLO, "Overview"),
            FallbackReason::Empty
        );
        assert_eq!(
            classify_refine_reject(FOLLO, "The landing page change did most of the"),
            FallbackReason::FirstSentence
        );
        assert_eq!(
            classify_refine_reject(FOLLO, "Landing page change significantly"),
            FallbackReason::FirstSentence
        );
        assert_eq!(
            classify_refine_reject(
                FOLLO,
                "Landing page change impact on Follo billing investigation"
            ),
            FallbackReason::FirstSentence
        );
        assert_eq!(
            classify_refine_reject(FOLLO, "Quantum photon lattice"),
            FallbackReason::Ungrounded
        );
        assert_eq!(
            accept_refined_title(FOLLO, "Follo billing query D7CEC1E."),
            Some("Follo billing query D7CEC1E".into())
        );
    }

    #[test]
    #[ignore]
    fn spike_qwen_follo_raw() {
        let _guard = hold_desired_tier();
        request_tier(TitleTier::Qwen05);
        if crate::weights::weights_present_for(TitleTier::Qwen05).is_err() {
            eprintln!("bronze-title: spike skipped missing qwen weights");
            return;
        }
        warmup();
        std::thread::sleep(Duration::from_secs(2));
        let samples = [
            "The landing page change did most of the work for the Follo billing investigation. Query performance impact from D7CEC1E versus the previous plan still needs a number before we sign off. Invoice review on Thursday is the remaining close item.",
            "Landing page change did most of the work for the Follo billing investigation. Query performance impact from D7CEC1E versus the previous plan still needs a number before we sign off. Invoice review on Thursday is the remaining close item. Finance still needs the D7CEC1E query count versus last week's plan before anyone signs the Follo close.",
            "What actually did it change? A landing page change did most of the work: fewer queries fired per visit. The old v3 data source (d7cec1ec) has had zero queries since the fix. We should write down the response-size change after the config rollout before calling the Follo investigation done.",
        ];
        for body in samples {
            let started = std::time::Instant::now();
            let outcome = refine_outcome(body);
            eprintln!(
                "bronze-title: spike chars={} elapsed_ms={} outcome={:?} compact={:?}",
                body.chars().count(),
                started.elapsed().as_millis(),
                outcome,
                bronze_domain::compact_title(body)
            );
        }
    }

    #[test]
    #[ignore]
    fn spike_smollm2_title_quality() {
        let _guard = hold_desired_tier();
        request_tier(TitleTier::Smol360);
        warmup();
        std::thread::sleep(Duration::from_secs(2));
        let samples = [
            "Thanks for the note.\nThe migration timeout is the real bug in persist.\nPlease take a look when you can.",
            "We should move the invoice review to Thursday so finance can close the books before Friday.",
            "fn persist_selection() { let title = compact_title(body); store.insert(title); }",
            "Park me",
            "The local title model is in. Capture still saves compact_title immediately; a Rust worker then tries SmolLM2 offline and only replaces the title if the body is unchanged and the title is grounded.",
            "The landing page change did most of the work for the Follo billing investigation. Query performance impact from D7CEC1E versus the previous plan still needs a number before we sign off. Invoice review on Thursday is the remaining close item.",
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
        let tiers = include_str!("tiers.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("prod");
        let status = include_str!("status.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("prod");
        let bundle = include_str!("bundle.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("prod");
        let custom = include_str!("custom.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("prod");
        for src in [infer, weights, prompt, lib, tiers, status, bundle, custom] {
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
