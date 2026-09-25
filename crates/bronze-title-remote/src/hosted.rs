use bronze_title_model::{accept_refined_title, truncate_input, SYSTEM_PROMPT};
use serde::Deserialize;

use crate::host::parse_hosted_base;
use crate::keystore::{slot_for_provider, SecretStore};
use crate::{http_agent, RemoteError};

pub const PAYLOAD_CLASS: &str = "truncated_capture_2048";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostedProvider {
    Openai,
    Anthropic,
    Openrouter,
}

impl HostedProvider {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim() {
            "hosted-openai" | "openai" => Some(Self::Openai),
            "hosted-anthropic" | "anthropic" => Some(Self::Anthropic),
            "hosted-openrouter" | "openrouter" => Some(Self::Openrouter),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Openai => "hosted-openai",
            Self::Anthropic => "hosted-anthropic",
            Self::Openrouter => "hosted-openrouter",
        }
    }

    pub fn default_base(self) -> &'static str {
        match self {
            Self::Openai => "https://api.openai.com",
            Self::Anthropic => "https://api.anthropic.com",
            Self::Openrouter => "https://openrouter.ai/api",
        }
    }

    pub fn default_host(self) -> &'static str {
        match self {
            Self::Openai => "api.openai.com",
            Self::Anthropic => "api.anthropic.com",
            Self::Openrouter => "openrouter.ai",
        }
    }

    pub fn default_model(self) -> &'static str {
        match self {
            Self::Openai => "gpt-4o-mini",
            Self::Anthropic => "claude-3-5-haiku-latest",
            Self::Openrouter => "openai/gpt-4o-mini",
        }
    }
}

pub fn disclosure(
    provider: HostedProvider,
    custom_base: &str,
) -> Result<(String, &'static str), RemoteError> {
    if custom_base.trim().is_empty() {
        return Ok((provider.default_host().into(), PAYLOAD_CLASS));
    }
    let (_, host) = parse_hosted_base(custom_base)?;
    Ok((host, PAYLOAD_CLASS))
}

pub fn refine(
    provider: HostedProvider,
    custom_base: &str,
    body: &str,
    secrets: &dyn SecretStore,
) -> Result<String, RemoteError> {
    let slot = slot_for_provider(provider.as_str()).ok_or(RemoteError::MissingKey)?;
    let key = secrets
        .get(slot)?
        .filter(|k| !k.is_empty())
        .ok_or(RemoteError::MissingKey)?;
    let raw = match provider {
        HostedProvider::Anthropic => {
            let url = "https://api.anthropic.com/v1/messages";
            anthropic_complete(url, &key, body)?
        }
        HostedProvider::Openai | HostedProvider::Openrouter => {
            let base = if custom_base.trim().is_empty() {
                provider.default_base().to_string()
            } else {
                parse_hosted_base(custom_base)?.0
            };
            openai_complete(&format!("{base}/v1/chat/completions"), &key, provider, body)?
        }
    };
    accept_refined_title(body, &raw).ok_or(RemoteError::Ungrounded)
}

fn openai_complete(
    url: &str,
    key: &str,
    provider: HostedProvider,
    body: &str,
) -> Result<String, RemoteError> {
    if url.contains("127.0.0.1") || url.contains("localhost") {
        return Err(RemoteError::BlockedHost);
    }
    let payload = serde_json::json!({
        "model": provider.default_model(),
        "temperature": 0,
        "max_tokens": 16,
        "messages": [
            { "role": "system", "content": SYSTEM_PROMPT },
            { "role": "user", "content": user_text(body) }
        ]
    });
    let resp = http_agent()
        .post(url)
        .set("Authorization", &format!("Bearer {key}"))
        .send_json(payload)
        .map_err(map_http)?;
    let parsed: OpenaiResponse = resp.into_json().map_err(|_| RemoteError::Empty)?;
    parsed
        .choices
        .first()
        .map(|choice| choice.message.content.clone())
        .filter(|text| !text.is_empty())
        .ok_or(RemoteError::Empty)
}

fn anthropic_complete(url: &str, key: &str, body: &str) -> Result<String, RemoteError> {
    let payload = serde_json::json!({
        "model": HostedProvider::Anthropic.default_model(),
        "max_tokens": 16,
        "temperature": 0,
        "system": SYSTEM_PROMPT,
        "messages": [ { "role": "user", "content": user_text(body) } ]
    });
    let resp = http_agent()
        .post(url)
        .set("x-api-key", key)
        .set("anthropic-version", "2023-06-01")
        .send_json(payload)
        .map_err(map_http)?;
    let parsed: AnthropicResponse = resp.into_json().map_err(|_| RemoteError::Empty)?;
    parsed
        .content
        .into_iter()
        .find_map(|block| {
            if block.kind == "text" && !block.text.is_empty() {
                Some(block.text)
            } else {
                None
            }
        })
        .ok_or(RemoteError::Empty)
}

fn map_http(err: ureq::Error) -> RemoteError {
    match err {
        ureq::Error::Status(401, _) | ureq::Error::Status(403, _) => RemoteError::Refused,
        _ => RemoteError::Unreachable,
    }
}

fn user_text(body: &str) -> String {
    format!(
        "Selected text:\n{}\n\nTopic headline. Name the subject and the change or problem.",
        truncate_input(body)
    )
}

#[derive(Deserialize)]
struct OpenaiResponse {
    #[serde(default)]
    choices: Vec<OpenaiChoice>,
}

#[derive(Deserialize)]
struct OpenaiChoice {
    #[serde(default)]
    message: OpenaiMessage,
}

#[derive(Default, Deserialize)]
struct OpenaiMessage {
    #[serde(default)]
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    #[serde(default)]
    content: Vec<AnthropicBlock>,
}

#[derive(Deserialize)]
struct AnthropicBlock {
    #[serde(default, rename = "type")]
    kind: String,
    #[serde(default)]
    text: String,
}

#[cfg(test)]
mod hosted_tests {
    use super::*;
    use crate::host::parse_hosted_base;
    use crate::keystore::MemorySecrets;

    #[test]
    fn missing_key_and_blocked_base_do_not_send() {
        let secrets = MemorySecrets::new();
        let err = refine(
            HostedProvider::Openai,
            "",
            "The migration timeout is the real bug in persist and needs a number.",
            &secrets,
        )
        .expect_err("missing");
        assert_eq!(err, RemoteError::MissingKey);
        assert_eq!(
            parse_hosted_base("https://172.16.0.4"),
            Err(RemoteError::BlockedHost)
        );
        let (host, class) = disclosure(HostedProvider::Openai, "").expect("disc");
        assert_eq!(host, "api.openai.com");
        assert_eq!(class, PAYLOAD_CLASS);
        let src = include_str!("hosted.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("prod");
        assert!(src.contains("/v1/chat/completions"));
        assert!(src.contains("/v1/messages"));
        assert!(src.contains("BlockedHost"));
        assert!(!src.contains("11434"));
    }
}
