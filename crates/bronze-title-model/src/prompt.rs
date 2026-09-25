use crate::tiers::TitleTier;
use bronze_domain::clamp_title;

pub const MAX_INPUT_CHARS: usize = 2048;
pub const MAX_NEW_TOKENS: i32 = 16;
pub const PROMPT_VERSION: &str = "v4";
pub const SYSTEM_PROMPT: &str = "Summarize the selected text as a 5-8 word topic headline. Name the subject and the change or problem. Reuse concrete nouns already in the text, especially product names, commit ids, and metrics. Do not copy or clip the first sentence. Do not write Overview, Notes, or Text. Output only the headline. No Title: prefix, no quotes, no hyphen slugs, no trailing period.";
pub const QWEN_CLOSER: &str =
    "Topic headline. Start with a later noun or commit id from the text, not the opening words:";
const SMOL_CLOSER: &str = "Topic headline for the whole selection (not the first sentence):";

pub fn truncate_input(body: &str) -> String {
    let trimmed = body.trim();
    if trimmed.chars().count() <= MAX_INPUT_CHARS {
        return trimmed.to_string();
    }
    trimmed.chars().take(MAX_INPUT_CHARS).collect()
}

pub fn format_prompt(body: &str) -> String {
    format_prompt_for(TitleTier::Smol360, body)
}

pub fn format_prompt_for(tier: TitleTier, body: &str) -> String {
    let body = truncate_input(body);
    let closer = match tier {
        TitleTier::Qwen05 => QWEN_CLOSER,
        TitleTier::Extractive | TitleTier::Smol135 | TitleTier::Smol360 | TitleTier::Custom => {
            SMOL_CLOSER
        }
    };
    format!(
        "<|im_start|>system\n{SYSTEM_PROMPT}<|im_end|>\n<|im_start|>user\nSelected text:\n{body}\n\n{closer}<|im_end|>\n<|im_start|>assistant\n"
    )
}

pub fn take_generated_piece(raw: &mut String, piece: &str) -> bool {
    let stripped = strip_chat_markup(piece);
    if stripped.is_empty() {
        return chat_stop_markup(piece) || !raw.is_empty();
    }
    let text = if raw.is_empty() {
        stripped.trim_start_matches(['\n', '\r', ' ', '\t'])
    } else {
        stripped.as_str()
    };
    if text.is_empty() {
        return false;
    }
    if let Some((head, _)) = text.split_once('\n') {
        raw.push_str(head);
        return true;
    }
    raw.push_str(text);
    chat_stop_markup(piece)
}

pub fn raw_preview(raw: &str) -> String {
    raw.chars().filter(|ch| !ch.is_control()).take(40).collect()
}

fn strip_chat_markup(piece: &str) -> String {
    piece
        .replace("<|im_end|>", "")
        .replace("<|im_start|>", "")
        .replace("<|endoftext|>", "")
}

fn chat_stop_markup(piece: &str) -> bool {
    piece.contains("<|im_end|>") || piece.contains("<|endoftext|>")
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
    let stripped = strip_headline_punct(stripped);
    let stripped = expand_hyphen_slug(stripped);
    let stripped = strip_dangling_tail(&stripped);
    if is_generic_headline(stripped) {
        return None;
    }
    let clamped = clamp_title(stripped);
    if clamped.is_empty() || clamped.contains('…') || is_generic_headline(&clamped) {
        None
    } else {
        Some(clamped)
    }
}

pub fn accept_refined_title(body: &str, raw: &str) -> Option<String> {
    let title = clean_title(raw)?;
    if title_is_grounded(body, &title) && !title_echoes_opening(body, &title) {
        Some(title)
    } else {
        None
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

fn strip_headline_punct(text: &str) -> &str {
    text.trim_end_matches(['.', '!', '?', ',', ';']).trim()
}

fn expand_hyphen_slug(text: &str) -> String {
    if text.contains(' ') || text.matches('-').count() < 2 {
        text.to_string()
    } else {
        text.replace('-', " ")
    }
}

fn strip_dangling_tail(text: &str) -> &str {
    let mut slice = text.trim_end();
    loop {
        let Some(last) = slice.split_whitespace().last() else {
            return "";
        };
        if !is_dangling_word(last) {
            return slice;
        }
        slice = slice[..slice.len() - last.len()].trim_end();
    }
}

fn is_dangling_word(word: &str) -> bool {
    matches!(
        word.to_ascii_lowercase().as_str(),
        "by" | "of"
            | "the"
            | "a"
            | "an"
            | "to"
            | "for"
            | "with"
            | "in"
            | "on"
            | "and"
            | "or"
            | "from"
            | "vs"
    )
}

fn is_generic_headline(title: &str) -> bool {
    matches!(
        title.to_ascii_lowercase().as_str(),
        "overview" | "notes" | "text"
    )
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

pub fn title_echoes_opening(body: &str, title: &str) -> bool {
    let title_words = opening_words(title);
    let opening = opening_words(opening_span(body));
    if title_words.len() < 3 || opening.len() < 3 {
        return false;
    }
    if title_words.len() >= 4 && title_words[..3] == opening[..3] {
        return true;
    }
    title_words.len() >= 5
        && (opening.starts_with(title_words.as_slice())
            || title_words.starts_with(opening.as_slice()))
}

fn opening_span(body: &str) -> &str {
    let trimmed = body.trim();
    let end = trimmed.find(['.', '!', '?', '\n']).unwrap_or(trimmed.len());
    trimmed[..end].trim()
}

fn opening_words(text: &str) -> Vec<String> {
    text.split(|ch: char| !ch.is_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(|part| part.to_ascii_lowercase())
        .filter(|part| !matches!(part.as_str(), "the" | "a" | "an"))
        .collect()
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

    const FOLLO_BODY: &str = "The landing page change did most of the work for the Follo billing investigation. Query performance impact from D7CEC1E versus the previous plan still needs a number before we sign off. Invoice review on Thursday is the remaining close item.";

    #[test]
    fn prompt_is_fixed_english_v4() {
        let prompt = format_prompt("The migration timeout is the real bug.");
        assert!(prompt.contains(SYSTEM_PROMPT));
        assert!(prompt.contains("Summarize the selected text as a 5-8 word topic headline"));
        assert!(prompt.contains("Name the subject and the change or problem"));
        assert!(prompt.contains("Do not copy or clip the first sentence"));
        assert!(prompt.contains("Selected text:"));
        assert!(prompt.contains("Topic headline for the whole selection (not the first sentence):"));
        assert!(prompt.contains("product names, commit ids, and metrics"));
        assert!(prompt.contains("The migration timeout is the real bug."));
        assert!(!prompt.contains("Write a 3-8 word title"));
        assert!(!prompt.contains("Write a title"));
        assert_eq!(PROMPT_VERSION, "v4");
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
        assert_eq!(
            clean_title("Title: The landing page change did most of the.").as_deref(),
            Some("The landing page change did most")
        );
        assert_eq!(
            clean_title("Follo billing query D7CEC1E.").as_deref(),
            Some("Follo billing query D7CEC1E")
        );
        assert_eq!(
            clean_title("Migrate-Timeout-Is-Real-Bug-In-Persist").as_deref(),
            Some("Migrate Timeout Is Real Bug In Persist")
        );
        assert_eq!(
            clean_title("Finance should review invoices by.").as_deref(),
            Some("Finance should review invoices")
        );
        assert_eq!(clean_title("Overview"), None);
        assert_eq!(clean_title("Notes"), None);
        assert_eq!(clean_title("Text"), None);
        let inner = clean_title("The topic is \"Persistence of Selection").expect("inner");
        assert!(!inner.contains('"'));
        assert!(quoted.to_ascii_lowercase().contains("migration"));
        let long = clean_title(&"word ".repeat(40)).expect("clamped");
        assert!(long.chars().count() <= 40);
        assert!(!long.contains('…'));
    }

    #[test]
    fn accept_refined_title_summarizes_instead_of_opening_echo() {
        assert_eq!(
            accept_refined_title(
                FOLLO_BODY,
                "Title: The landing page change did most of the."
            ),
            None
        );
        assert_eq!(
            accept_refined_title(FOLLO_BODY, "The landing page change did most of the"),
            None
        );
        assert_eq!(
            accept_refined_title(FOLLO_BODY, "Landing page change significantly"),
            None
        );
        assert_eq!(accept_refined_title(FOLLO_BODY, "Overview"), None);
        assert_eq!(
            accept_refined_title(FOLLO_BODY, "Quantum photon lattice"),
            None
        );
        assert_eq!(
            accept_refined_title(FOLLO_BODY, "Follo billing query D7CEC1E."),
            Some("Follo billing query D7CEC1E".into())
        );
        assert_eq!(
            accept_refined_title(FOLLO_BODY, "Query performance impact D7CEC1E"),
            Some("Query performance impact D7CEC1E".into())
        );
        assert!(title_echoes_opening(
            FOLLO_BODY,
            "The landing page change did most of the"
        ));
        assert!(title_echoes_opening(
            FOLLO_BODY,
            "Landing page change significantly"
        ));
        assert!(!title_echoes_opening(
            FOLLO_BODY,
            "Follo billing query D7CEC1E"
        ));
    }

    #[test]
    fn qwen_prompt_asks_for_later_noun_not_opening_words() {
        let prompt = format_prompt_for(TitleTier::Qwen05, FOLLO_BODY);
        assert!(prompt.contains("<|im_start|>system"));
        assert!(prompt.contains("<|im_start|>user"));
        assert!(prompt.contains("<|im_start|>assistant\n"));
        assert!(prompt.contains(QWEN_CLOSER));
        assert!(prompt.contains("later noun or commit id"));
        assert!(prompt.contains("not the opening words"));
        assert!(prompt.contains(FOLLO_BODY));
        assert!(!prompt.contains(SMOL_CLOSER));
        let smol = format_prompt_for(TitleTier::Smol360, FOLLO_BODY);
        assert!(smol.contains(SMOL_CLOSER));
        assert!(!smol.contains(QWEN_CLOSER));
    }

    #[test]
    fn leading_newline_does_not_empty_the_title() {
        let mut raw = String::new();
        assert!(!take_generated_piece(&mut raw, "\n"));
        assert!(raw.is_empty());
        assert!(!take_generated_piece(&mut raw, "Follo billing query"));
        assert_eq!(raw, "Follo billing query");
        assert!(take_generated_piece(&mut raw, " D7CEC1E\nmore"));
        assert_eq!(raw, "Follo billing query D7CEC1E");
    }

    #[test]
    fn chat_end_token_stops_without_being_kept() {
        let mut raw = String::new();
        assert!(take_generated_piece(&mut raw, "<|im_end|>"));
        assert!(raw.is_empty());
        assert!(!take_generated_piece(&mut raw, "Follo billing"));
        assert!(take_generated_piece(&mut raw, "<|im_end|>"));
        assert_eq!(raw, "Follo billing");
        let mut mixed = String::new();
        assert!(take_generated_piece(&mut mixed, "Follo billing<|im_end|>"));
        assert_eq!(mixed, "Follo billing");
    }

    #[test]
    fn raw_preview_is_short_and_strips_controls() {
        let preview =
            raw_preview("Landing page change impact on Follo billing investigation\nsecret");
        assert_eq!(preview.chars().count(), 40);
        assert!(!preview.contains('\n'));
        assert_eq!(
            raw_preview("Follo\u{0007} billing").as_str(),
            "Follo billing"
        );
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
        assert!(title_is_grounded(FOLLO_BODY, "Follo billing query D7CEC1E"));
        assert!(!title_is_grounded("Park me", "The Power of the Sun"));
    }
}
