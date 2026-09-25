// Score bands match `packages/ui/src/lib/fuzzy-match.ts`.

pub const FUZZY_EXACT: i32 = 1_000_000;
pub const FUZZY_PREFIX: i32 = 800_000;
pub const FUZZY_SUBSTRING: i32 = 600_000;
pub const FUZZY_SUBSEQUENCE: i32 = 400_000;
pub const FUZZY_TYPO: i32 = 200_000;

const SUBSEQUENCE_MIN: usize = 3;
const MAX_GAP: usize = 1;
const TYPO_MIN: usize = 4;

pub fn locale_lowercase(text: &str, locale: &str) -> String {
    if turkic_locale(locale) {
        let mut out = String::new();
        for ch in text.chars() {
            match ch {
                'I' => out.push('ı'),
                'İ' => out.push('i'),
                other => {
                    for lower in other.to_lowercase() {
                        out.push(lower);
                    }
                }
            }
        }
        return out;
    }
    text.to_lowercase()
}

fn turkic_locale(locale: &str) -> bool {
    let primary = locale
        .trim()
        .split(['-', '_'])
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    primary == "tr" || primary == "az"
}

fn closeness(distance: usize) -> i32 {
    1000 - distance.min(999) as i32
}

fn starts_with(hay: &[char], needle: &[char]) -> bool {
    needle.len() <= hay.len() && hay[..needle.len()] == needle[..]
}

fn index_of_slice(hay: &[char], needle: &[char]) -> Option<usize> {
    if needle.is_empty() || needle.len() > hay.len() {
        return None;
    }
    let last = hay.len() - needle.len();
    for start in 0..=last {
        if hay[start..start + needle.len()] == needle[..] {
            return Some(start);
        }
    }
    None
}

fn subsequence_bonus(hay: &[char], needle: &[char]) -> Option<i32> {
    if needle.len() < SUBSEQUENCE_MIN {
        return None;
    }
    let mut from = 0;
    let mut prev: Option<usize> = None;
    let mut first = 0;
    let mut gaps = 0usize;
    for ch in needle {
        let mut found = None;
        let mut index = from;
        while index < hay.len() {
            if hay[index] == *ch {
                found = Some(index);
                break;
            }
            index += 1;
        }
        let at = found?;
        if let Some(previous) = prev {
            let gap = at - previous - 1;
            if gap > MAX_GAP {
                return None;
            }
            gaps += gap;
        } else {
            first = at;
        }
        prev = Some(at);
        from = at + 1;
    }
    let penalty = (first * 2 + gaps).min(999) as i32;
    Some(999 - penalty)
}

fn within_one_ascii_edit(left: &[char], right: &[char]) -> bool {
    if left == right {
        return true;
    }
    if left.len().abs_diff(right.len()) > 1 {
        return false;
    }
    if left.len() == right.len() {
        let mut index = 0;
        let mut edits = 0;
        while index < left.len() {
            if left[index] == right[index] {
                index += 1;
                continue;
            }
            edits += 1;
            if edits > 1 {
                return false;
            }
            if index + 1 < left.len()
                && left[index] == right[index + 1]
                && left[index + 1] == right[index]
                && left[index].is_ascii_lowercase()
                && left[index + 1].is_ascii_lowercase()
                && right[index].is_ascii_lowercase()
                && right[index + 1].is_ascii_lowercase()
            {
                index += 2;
                continue;
            }
            if !left[index].is_ascii_lowercase() || !right[index].is_ascii_lowercase() {
                return false;
            }
            index += 1;
        }
        return edits == 1;
    }
    let (shorter, longer) = if left.len() < right.len() {
        (left, right)
    } else {
        (right, left)
    };
    let mut i = 0;
    let mut j = 0;
    let mut skips = 0;
    while i < shorter.len() && j < longer.len() {
        if shorter[i] == longer[j] {
            i += 1;
            j += 1;
            continue;
        }
        if !longer[j].is_ascii_lowercase() {
            return false;
        }
        skips += 1;
        if skips > 1 {
            return false;
        }
        j += 1;
    }
    if i != shorter.len() {
        return false;
    }
    if j < longer.len() {
        return skips == 0 && longer.len() - j == 1 && longer[j].is_ascii_lowercase();
    }
    skips <= 1
}

fn words(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start: Option<usize> = None;
    for (index, ch) in text.char_indices() {
        if ch.is_alphanumeric() {
            if start.is_none() {
                start = Some(index);
            }
        } else if let Some(begin) = start.take() {
            out.push(&text[begin..index]);
        }
    }
    if let Some(begin) = start {
        out.push(&text[begin..]);
    }
    out
}

fn typo_bonus(haystack: &str, query: &str) -> Option<i32> {
    let query_chars: Vec<char> = query.chars().collect();
    if query_chars.len() < TYPO_MIN {
        return None;
    }
    let hay_chars: Vec<char> = haystack.chars().collect();
    if within_one_ascii_edit(&hay_chars, &query_chars) {
        return Some(closeness(0));
    }
    let mut best = None;
    let mut from = 0usize;
    for word in words(haystack) {
        let at = haystack[from..]
            .find(word)
            .map(|pos| from + pos)
            .unwrap_or(from);
        from = at + word.len();
        let word_chars: Vec<char> = word.chars().collect();
        if word_chars.len().abs_diff(query_chars.len()) > 1 {
            continue;
        }
        if !within_one_ascii_edit(&word_chars, &query_chars) {
            continue;
        }
        let char_start = haystack[..at].chars().count();
        let bonus = closeness(char_start);
        best = Some(best.map_or(bonus, |current: i32| current.max(bonus)));
    }
    best
}

pub fn fuzzy_match_score(haystack: &str, query: &str, locale: &str) -> Option<i32> {
    let folded_query = locale_lowercase(query.trim(), locale);
    if folded_query.is_empty() {
        return Some(0);
    }
    let folded = locale_lowercase(haystack, locale);
    if folded.is_empty() {
        return None;
    }
    if folded == folded_query {
        return Some(FUZZY_EXACT);
    }
    let hay: Vec<char> = folded.chars().collect();
    let needle: Vec<char> = folded_query.chars().collect();
    if starts_with(&hay, &needle) {
        return Some(FUZZY_PREFIX + closeness(hay.len() - needle.len()));
    }
    if let Some(index) = index_of_slice(&hay, &needle) {
        return Some(FUZZY_SUBSTRING + closeness(index));
    }
    if let Some(bonus) = subsequence_bonus(&hay, &needle) {
        return Some(FUZZY_SUBSEQUENCE + bonus);
    }
    typo_bonus(&folded, &folded_query).map(|bonus| FUZZY_TYPO + bonus)
}

pub fn fuzzy_best_score<'a, I>(parts: I, query: &str, locale: &str) -> Option<i32>
where
    I: IntoIterator<Item = &'a str>,
{
    if locale_lowercase(query.trim(), locale).is_empty() {
        return Some(0);
    }
    let mut best = None;
    for part in parts {
        if let Some(score) = fuzzy_match_score(part, query, locale) {
            if score > 0 && best.is_none_or(|current| score > current) {
                best = Some(score);
            }
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scores_rank_exact_prefix_substring_fuzzy_then_typo() {
        assert_eq!(fuzzy_match_score("Bronze", "", "en"), Some(0));
        assert_eq!(fuzzy_match_score("Bronze", "   ", "en"), Some(0));
        assert_eq!(
            fuzzy_match_score("Bronze", "Bronze", "en"),
            Some(FUZZY_EXACT)
        );
        assert_eq!(
            fuzzy_match_score("Bronze", "BRONZE", "en"),
            Some(FUZZY_EXACT)
        );
        assert_eq!(
            fuzzy_match_score("Bronze", "Bron", "en"),
            Some(FUZZY_PREFIX + 998)
        );
        assert_eq!(
            fuzzy_match_score("xxBronze", "bronze", "en"),
            Some(FUZZY_SUBSTRING + 998)
        );
        assert_eq!(
            fuzzy_match_score("Bronze", "brn", "en"),
            Some(FUZZY_SUBSEQUENCE + 998)
        );
        assert_eq!(
            fuzzy_match_score("Bronze", "Bronxe", "en"),
            Some(FUZZY_TYPO + 1000)
        );
        assert_eq!(
            fuzzy_match_score("Bronze", "Bronez", "en"),
            Some(FUZZY_TYPO + 1000)
        );
        assert_eq!(fuzzy_match_score("Bronze", "zzzz", "en"), None);
        assert_eq!(fuzzy_match_score("Café token", "cafe", "en"), None);
        assert_eq!(
            fuzzy_match_score("Café token", "CAFÉ", "en"),
            Some(FUZZY_PREFIX + 994)
        );
    }

    #[test]
    fn turkish_i_follows_the_ui_locale() {
        assert_eq!(locale_lowercase("I", "tr"), "ı");
        assert_eq!(locale_lowercase("İ", "tr"), "i");
        assert_eq!(locale_lowercase("İ", "en"), "i\u{307}");
        assert_eq!(
            fuzzy_match_score("Istanbul", "ıstanbul", "tr"),
            Some(FUZZY_EXACT)
        );
        assert_eq!(fuzzy_match_score("Istanbul", "istanbul", "tr"), None);
        assert_eq!(
            fuzzy_match_score("Istanbul", "istanbul", "en"),
            Some(FUZZY_EXACT)
        );
        assert_eq!(
            fuzzy_match_score("İstanbul", "istanbul", "tr"),
            Some(FUZZY_EXACT)
        );
        assert_ne!(
            fuzzy_match_score("İstanbul", "istanbul", "en"),
            Some(FUZZY_EXACT)
        );
        assert_eq!(
            fuzzy_match_score("Istanbul", "ıstanbul", "tr-TR"),
            Some(FUZZY_EXACT)
        );
    }

    #[test]
    fn best_score_and_empty_query_keep_every_part() {
        assert_eq!(
            fuzzy_best_score(["Notes", "Bronze"], "brn", "en"),
            Some(FUZZY_SUBSEQUENCE + 998)
        );
        assert_eq!(fuzzy_best_score(["Notes"], "", "en"), Some(0));
        assert_eq!(fuzzy_best_score(["Notes"], "zzzz", "en"), None);
    }
}
