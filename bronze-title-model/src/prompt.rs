use bronze_domain::clamp_title;

pub const MAX_INPUT_CHARS: usize = 2048;
pub const MAX_NEW_TOKENS: i32 = 16;
pub const PROMPT_VERSION: &str = "v3";

pub fn truncate_input(body: &str) -> String {
    let trimmed = body.trim();
    if trimmed.chars().count() <= MAX_INPUT_CHARS {
        return trimmed.to_string();
    }
    trimmed.chars().take(MAX_INPUT_CHARS).collect()
}

pub fn format_prompt(body: &str) -> String {
    let body = truncate_input(body);
    format!(
        "<|im_start|>system\nWrite a 3-8 word title naming the topic. Do not copy a sentence. No quotes. Do not prefix with Title:.<|im_end|>\n<|im_start|>user\n{body}<|im_end|>\n<|im_start|>assistant\n"
    )
}

pub fn clean_title(raw: &str) -> Option<String> {
    let line = raw.lines().next()?.trim();
    if line.is_empty() {
        return None;
    }
    let stripped = strip_title_prefix(line);
    let stripped = stripped.trim_matches(|ch| matches!(ch, '"' | '\'' | '`'));
    let stripped: String = stripped
        .chars()
        .filter(|ch| *ch != '"' && *ch != '`')
        .collect();
    let stripped = strip_title_prefix(stripped.trim());
    let clamped = clamp_title(stripped);
    if clamped.is_empty() || clamped.contains('…') {
        None
    } else {
        Some(clamped)
    }
}

fn strip_title_prefix(line: &str) -> &str {
    let trimmed = line.trim();
    let bytes = trimmed.as_bytes();
    if bytes.len() >= 6
        && bytes[5] == b':'
        && trimmed
            .get(..5)
            .is_some_and(|head| head.eq_ignore_ascii_case("title"))
    {
        trimmed[6..].trim_start()
    } else {
        trimmed
    }
}

pub fn title_is_grounded(body: &str, title: &str) -> bool {
    let body_terms = significant_terms(body);
    if body_terms.is_empty() {
        return false;
    }
    significant_terms(title)
        .iter()
        .any(|term| body_terms.iter().any(|body| terms_overlap(body, term)))
}

fn terms_overlap(body: &str, title: &str) -> bool {
    body == title
        || (body.len() >= 4 && title.len() >= 4 && (body.contains(title) || title.contains(body)))
}

fn significant_terms(text: &str) -> Vec<String> {
    text.split(|ch: char| !ch.is_alphanumeric())
        .filter(|part| part.len() > 2)
        .map(|part| part.to_ascii_lowercase())
        .collect()
}

#[cfg(test)]
mod prompt_tests {
    use super::*;

    #[test]
    fn prompt_is_fixed_english_v2() {
        let prompt = format_prompt("The migration timeout is the real bug.");
        assert!(prompt.contains("3-8 word title"));
        assert!(prompt.contains("Do not copy a sentence"));
        assert!(prompt.contains("Do not prefix with Title:"));
        assert!(prompt.contains("The migration timeout is the real bug."));
        assert_eq!(PROMPT_VERSION, "v3");
        assert_eq!(MAX_NEW_TOKENS, 16);
    }

    #[test]
    fn truncate_caps_input() {
        let long = "x".repeat(MAX_INPUT_CHARS + 80);
        assert_eq!(truncate_input(&long).chars().count(), MAX_INPUT_CHARS);
    }

    #[test]
    fn clean_title_clamps_and_drops_empty() {
        assert_eq!(clean_title("   "), None);
        assert_eq!(clean_title("\n"), None);
        let quoted = clean_title("\"Migration timeout\"\nmore").expect("title");
        assert!(!quoted.contains('"'));
        assert_eq!(
            clean_title("Title: Landing Page Change Hasimproved").as_deref(),
            Some("Landing Page Change Hasimproved")
        );
        assert_eq!(
            clean_title("title: Queue card heading").as_deref(),
            Some("Queue card heading")
        );
        assert_eq!(
            clean_title("\"Title: Landing Page Change Hasimproved\"").as_deref(),
            Some("Landing Page Change Hasimproved")
        );
        assert_eq!(clean_title("Title:Foo Bar").as_deref(), Some("Foo Bar"));
        let inner = clean_title("The topic is \"Persistence of Selection").expect("inner");
        assert!(!inner.contains('"'));
        assert!(quoted.to_ascii_lowercase().contains("migration"));
        let long = clean_title(&"word ".repeat(40)).expect("clamped");
        assert!(long.chars().count() <= 40);
        assert!(!long.contains('…'));
    }

    #[test]
    fn groundedness_rejects_hallucinated_titles() {
        assert!(title_is_grounded(
            "The migration timeout is the real bug in persist.",
            "Migration timeout"
        ));
        assert!(title_is_grounded(
            "We should move the invoice review to Thursday so finance can close the books before Friday.",
            "Financial close"
        ));
        assert!(!title_is_grounded("Park me", "The Power of the Sun"));
    }
}
