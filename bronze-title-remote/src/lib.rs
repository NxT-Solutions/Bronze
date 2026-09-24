//! Opt-in title refine over loopback Ollama or a hosted API. Not used unless selected.

mod host;
mod hosted;
mod keystore;
mod ollama;

pub use host::{
    hosted_host_blocked, is_loopback_host, parse_hosted_base, resolve_ollama_base,
    OLLAMA_DEFAULT_BASE,
};
pub use hosted::refine as hosted_refine;
pub use hosted::{disclosure, HostedProvider, PAYLOAD_CLASS};
pub use keystore::{
    slot_for_provider, KeychainSecrets, MemorySecrets, SecretStore, SLOT_ANTHROPIC, SLOT_OPENAI,
    SLOT_OPENROUTER,
};
pub use ollama::{list_models as list_ollama_models, refine as ollama_refine};

use std::time::Duration;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RemoteError {
    NotLoopback,
    BlockedHost,
    InvalidBase,
    MissingKey,
    Unreachable,
    Refused,
    Empty,
    Ungrounded,
}

impl RemoteError {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotLoopback => "not_loopback",
            Self::BlockedHost => "blocked_host",
            Self::InvalidBase => "invalid_base",
            Self::MissingKey => "missing_key",
            Self::Unreachable => "unreachable",
            Self::Refused => "refused",
            Self::Empty => "empty",
            Self::Ungrounded => "ungrounded",
        }
    }
}

fn http_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(8))
        .build()
}

#[cfg(test)]
mod remote_tests {
    fn prod(src: &str) -> &str {
        src.split("#[cfg(test)]").next().expect("prod")
    }

    #[test]
    fn title_model_crate_still_has_no_http() {
        let infer = prod(include_str!("../../bronze-title-model/src/infer.rs"));
        let weights = prod(include_str!("../../bronze-title-model/src/weights.rs"));
        let custom = prod(include_str!("../../bronze-title-model/src/custom.rs"));
        let lib = include_str!("../../bronze-title-model/src/lib.rs");
        for src in [infer, weights, custom, lib] {
            let lower = src.to_ascii_lowercase();
            assert!(!lower.contains("reqwest"));
            assert!(!lower.contains("ureq"));
            assert!(!lower.contains("openai"));
        }
    }

    #[test]
    fn agent_builder_does_not_take_proxy_from_env() {
        let lib = prod(include_str!("lib.rs"));
        assert!(lib.contains("AgentBuilder::new()"));
        assert!(!lib.contains("proxy_from_env"));
        assert!(!lib.contains("HTTP_PROXY"));
        assert!(!lib.contains("ALL_PROXY"));
    }
}
