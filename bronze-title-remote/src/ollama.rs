use bronze_title_model::{accept_refined_title, truncate_input, SYSTEM_PROMPT};
use serde::Deserialize;

use crate::host::resolve_ollama_base;
use crate::{http_agent, RemoteError};

#[derive(Deserialize)]
struct TagsResponse {
    #[serde(default)]
    models: Vec<TagModel>,
}

#[derive(Deserialize)]
struct TagModel {
    #[serde(default)]
    name: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    #[serde(default)]
    message: ChatMessage,
}

#[derive(Default, Deserialize)]
struct ChatMessage {
    #[serde(default)]
    content: String,
}

pub fn list_models() -> Result<Vec<String>, RemoteError> {
    let base = resolve_ollama_base(std::env::var("OLLAMA_HOST").ok().as_deref());
    let url = format!("{base}/api/tags");
    let resp = http_agent()
        .get(&url)
        .call()
        .map_err(|_| RemoteError::Unreachable)?;
    let parsed: TagsResponse = resp.into_json().map_err(|_| RemoteError::Empty)?;
    Ok(parsed
        .models
        .into_iter()
        .map(|model| model.name)
        .filter(|name| !name.is_empty())
        .collect())
}

pub fn refine(model: &str, body: &str) -> Result<String, RemoteError> {
    if model.trim().is_empty() {
        return Err(RemoteError::Empty);
    }
    let base = resolve_ollama_base(std::env::var("OLLAMA_HOST").ok().as_deref());
    let url = format!("{base}/api/chat");
    let payload = serde_json::json!({
        "model": model,
        "stream": false,
        "options": { "num_predict": 16, "temperature": 0 },
        "messages": [
            { "role": "system", "content": SYSTEM_PROMPT },
            { "role": "user", "content": user_text(body) }
        ]
    });
    let resp = http_agent()
        .post(&url)
        .send_json(payload)
        .map_err(|_| RemoteError::Unreachable)?;
    let parsed: ChatResponse = resp.into_json().map_err(|_| RemoteError::Empty)?;
    accept_refined_title(body, &parsed.message.content).ok_or(RemoteError::Ungrounded)
}

fn user_text(body: &str) -> String {
    format!(
        "Selected text:\n{}\n\nTopic headline. Name the subject and the change or problem.",
        truncate_input(body)
    )
}

#[cfg(test)]
mod ollama_tests {
    use crate::host::{resolve_ollama_base, OLLAMA_DEFAULT_BASE};

    #[test]
    fn list_and_refine_use_loopback_only() {
        assert_eq!(
            resolve_ollama_base(Some("https://ollama.com")),
            OLLAMA_DEFAULT_BASE
        );
        let ollama = include_str!("ollama.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("prod");
        assert!(ollama.contains("/api/chat"));
        assert!(ollama.contains("stream"));
        assert!(!ollama.contains("/api/pull"));
        assert!(ollama.contains("num_predict"));
    }
}
