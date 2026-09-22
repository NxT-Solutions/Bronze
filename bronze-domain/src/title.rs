//! Extractive item titles from captured copy.

// 400px panel − 20×2 chrome − 16×2 card ≈ 328px; 15px SF ~8px/Latin glyph.
const TITLE_MAX_CHARS: usize = 40;

pub fn compact_title(body: &str) -> String {
    let plain = strip_markup(body);
    let sentences = split_sentences(&plain);
    let picked = best_sentence(&sentences).unwrap_or("");
    clamp_title(picked)
}

pub fn clamp_title(text: &str) -> String {
    clamp_chars(text, TITLE_MAX_CHARS)
}

pub fn strip_markup(body: &str) -> String {
    let mut out = String::with_capacity(body.len());
    for ch in body.chars() {
        match ch {
            '*' | '`' | '#' => {}
            '\r' => {}
            '\n' | '\t' => out.push(' '),
            _ => out.push(ch),
        }
    }
    let collapsed: String = out.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed
}

fn split_sentences(plain: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let bytes = plain.as_bytes();
    for (idx, ch) in plain.char_indices() {
        let end = matches!(ch, '.' | '!' | '?' | ';' | '\n');
        if end {
            let piece = plain[start..idx].trim();
            if !piece.is_empty() {
                out.push(piece);
            }
            start = idx + ch.len_utf8();
            while start < bytes.len() && bytes[start].is_ascii_whitespace() {
                start += 1;
            }
        }
    }
    let tail = plain[start..].trim();
    if !tail.is_empty() {
        out.push(tail);
    }
    if out.is_empty() && !plain.trim().is_empty() {
        out.push(plain.trim());
    }
    out
}

fn best_sentence<'a>(sentences: &[&'a str]) -> Option<&'a str> {
    if sentences.is_empty() {
        return None;
    }
    if sentences.len() == 1 {
        return Some(sentences[0]);
    }
    let doc = document_terms(sentences);
    let mut best = sentences[0];
    let mut best_score = f64::MIN;
    for sentence in sentences {
        let score = sentence_score(sentence, &doc);
        if score > best_score {
            best_score = score;
            best = sentence;
        }
    }
    Some(best)
}

fn document_terms(sentences: &[&str]) -> std::collections::HashMap<String, f64> {
    let mut counts = std::collections::HashMap::new();
    let mut total = 0.0;
    for sentence in sentences {
        for term in significant_terms(sentence) {
            *counts.entry(term).or_insert(0.0) += 1.0;
            total += 1.0;
        }
    }
    if total > 0.0 {
        for value in counts.values_mut() {
            *value /= total;
        }
    }
    counts
}

fn sentence_score(sentence: &str, doc: &std::collections::HashMap<String, f64>) -> f64 {
    let terms = significant_terms(sentence);
    if terms.is_empty() {
        return f64::MIN / 2.0;
    }
    let mut score = 0.0;
    for term in &terms {
        score += doc.get(term).copied().unwrap_or(0.0);
    }
    let len = sentence.chars().count() as f64;
    if len < 12.0 {
        score *= 0.25;
    }
    if len > 160.0 {
        score *= 0.6;
    }
    score
}

fn significant_terms(sentence: &str) -> Vec<String> {
    sentence
        .split(|ch: char| !ch.is_alphanumeric())
        .filter(|part| part.len() > 2)
        .map(|part| part.to_ascii_lowercase())
        .filter(|part| !STOP.contains(&part.as_str()))
        .collect()
}

fn clamp_chars(text: &str, max: usize) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= max {
        return trimmed.to_string();
    }
    let taken: String = trimmed.chars().take(max).collect();
    match taken.rfind(char::is_whitespace) {
        Some(idx) if idx > 0 => taken[..idx].trim_end().to_string(),
        _ => taken,
    }
}

const STOP: &[&str] = &[
    "the", "and", "for", "that", "with", "this", "from", "have", "was", "are", "but", "not", "you",
    "your", "all", "can", "has", "had", "were", "they", "their", "them", "its",
];

#[cfg(test)]
mod title_tests {
    use super::*;

    #[test]
    fn compact_title_picks_the_content_bearing_sentence() {
        let body = "Thanks for the note.\nThe migration timeout is the real bug in persist.\nPlease take a look when you can.";
        let title = compact_title(body);
        assert!(title.to_ascii_lowercase().contains("migration"));
        assert!(!title.to_ascii_lowercase().starts_with("thanks"));
        assert!(title.chars().count() <= TITLE_MAX_CHARS);
    }

    #[test]
    fn compact_title_strips_markup_and_clamps() {
        let title = compact_title("**Bold heading** that keeps going and going and going past the limit of a tray label and then some extra words");
        assert!(!title.contains('*'));
        assert!(title.chars().count() <= TITLE_MAX_CHARS);
        assert!(!title.contains('…'));
        assert!(!title.ends_with(' '));
    }

    #[test]
    fn compact_title_fits_one_card_line_without_ellipsis() {
        let title = compact_title(&"word ".repeat(30));
        assert!(title.chars().count() <= TITLE_MAX_CHARS);
        assert!(!title.contains('…'));
        assert_eq!(TITLE_MAX_CHARS, 40);
    }

    #[test]
    fn compact_title_hard_cuts_a_single_overlong_token() {
        let title = compact_title(&"a".repeat(80));
        assert_eq!(title.chars().count(), TITLE_MAX_CHARS);
        assert!(!title.contains('…'));
    }

    #[test]
    fn clamp_title_matches_compact_title_limit() {
        let clamped = clamp_title("  Migration timeout is the real persist bug and more words  ");
        assert!(clamped.chars().count() <= TITLE_MAX_CHARS);
        assert!(!clamped.contains('…'));
        assert_eq!(clamped, clamp_title(&clamped));
    }
}
