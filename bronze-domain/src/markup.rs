//! Constrained markdown for captured attributed text.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StyleRun {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
}

pub fn markdown_from_runs(runs: &[StyleRun]) -> String {
    let mut out = String::new();
    for run in runs {
        if run.text.is_empty() {
            continue;
        }
        let escaped = escape_inline(&run.text);
        match (run.bold, run.italic) {
            (true, true) => {
                out.push_str("***");
                out.push_str(&escaped);
                out.push_str("***");
            }
            (true, false) => {
                out.push_str("**");
                out.push_str(&escaped);
                out.push_str("**");
            }
            (false, true) => {
                out.push('*');
                out.push_str(&escaped);
                out.push('*');
            }
            (false, false) => out.push_str(&escaped),
        }
    }
    out
}

pub fn font_name_traits(name: &str) -> (bool, bool) {
    let lower = name.to_ascii_lowercase();
    let bold = lower.contains("bold")
        || lower.contains("black")
        || lower.contains("heavy")
        || lower.contains("semibold")
        || lower.contains("demibold");
    let italic = lower.contains("italic") || lower.contains("oblique");
    (bold, italic)
}

fn escape_inline(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('*', "\\*")
        .replace('`', "\\`")
}

fn list_item_body(line: &str) -> Option<(Option<u32>, &str, bool)> {
    let trimmed = line.trim_start();
    if let Some(rest) = trimmed.strip_prefix("- ") {
        return Some((None, rest, false));
    }
    if let Some(rest) = trimmed.strip_prefix("+ ") {
        return Some((None, rest, false));
    }
    if let Some(rest) = trimmed.strip_prefix("• ") {
        return Some((None, rest, false));
    }
    if let Some(rest) = trimmed.strip_prefix("* ") {
        return Some((None, rest, false));
    }
    let digits = trimmed.bytes().take_while(u8::is_ascii_digit).count();
    if digits > 0 {
        let after = &trimmed[digits..];
        if let Some(rest) = after.strip_prefix(". ") {
            let number = trimmed[..digits].parse::<u32>().ok();
            return Some((number, rest, true));
        }
    }
    None
}

fn marker_gap(ch: char) -> bool {
    ch.is_whitespace() && ch != '\n' && ch != '\r' && ch != '\u{2028}' && ch != '\u{2029}'
}

fn glued_list_marker(chars: &[char], index: usize) -> Option<usize> {
    if index == 0 || chars[index - 1].is_whitespace() {
        return None;
    }
    let mut end = index;
    if end >= chars.len() || !chars[end].is_ascii_digit() {
        return None;
    }
    while end < chars.len() && chars[end].is_ascii_digit() {
        end += 1;
    }
    if end >= chars.len() || chars[end] != '.' {
        return None;
    }
    end += 1;
    if end >= chars.len() || !marker_gap(chars[end]) {
        return None;
    }
    while end < chars.len() && marker_gap(chars[end]) {
        end += 1;
    }
    Some(end - index)
}

fn jammed_sentence(chars: &[char], index: usize) -> bool {
    if chars.get(index) != Some(&'.') || index < 2 {
        return false;
    }
    let prev = chars[index - 1];
    let before = chars[index - 2];
    if !prev.is_lowercase() || !before.is_lowercase() {
        return false;
    }
    matches!(chars.get(index + 1), Some(next) if next.is_uppercase())
}

/// Accessibility selections often omit block separators. A list marker glued
/// to the previous word, or a sentence end jammed onto the next capital,
/// is restored. Spaced prose is left as captured.
pub fn restore_smashed_structure(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut index = 0;
    while index < chars.len() {
        if let Some(len) = glued_list_marker(&chars, index) {
            out.push('\n');
            for ch in &chars[index..index + len] {
                out.push(*ch);
            }
            index += len;
            continue;
        }
        if jammed_sentence(&chars, index) {
            out.push('.');
            out.push('\n');
            out.push('\n');
            index += 1;
            continue;
        }
        out.push(chars[index]);
        index += 1;
    }
    out
}

fn render_inline_markdown(md: &str, body: &mut String) {
    let chars: Vec<char> = md.chars().collect();
    let mut i = 0;
    let mut bold = false;
    let mut italic = false;
    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            let next = chars[i + 1];
            if next == '\\' || next == '*' || next == '`' {
                append_text(body, next);
                i += 2;
                continue;
            }
        }
        if chars[i] == '*' {
            if matches!(chars.get(i..i + 3), Some(['*', '*', '*'])) {
                let want_bold = !bold;
                let want_italic = !italic;
                set_style(body, &mut bold, &mut italic, want_bold, want_italic);
                i += 3;
                continue;
            }
            if matches!(chars.get(i..i + 2), Some(['*', '*'])) {
                let want_bold = !bold;
                let keep_italic = italic;
                set_style(body, &mut bold, &mut italic, want_bold, keep_italic);
                i += 2;
                continue;
            }
            let keep_bold = bold;
            let want_italic = !italic;
            set_style(body, &mut bold, &mut italic, keep_bold, want_italic);
            i += 1;
            continue;
        }
        if chars[i] == '\r' {
            body.push_str("<br>");
            i += 1;
            if chars.get(i) == Some(&'\n') {
                i += 1;
            }
            continue;
        }
        if chars[i] == '\n' {
            body.push_str("<br>");
            i += 1;
            continue;
        }
        append_text(body, chars[i]);
        i += 1;
    }
    set_style(body, &mut bold, &mut italic, false, false);
}

pub fn html_from_constrained_markdown(md: &str) -> String {
    let normalized = restore_smashed_structure(md);
    let mut body = String::new();
    let lines: Vec<&str> = normalized.split('\n').collect();
    let mut i = 0;
    while i < lines.len() {
        if let Some((_, _, ordered)) = list_item_body(lines[i]) {
            let tag = if ordered { "ol" } else { "ul" };
            body.push_str(&format!("<{tag}>"));
            while i < lines.len() {
                let Some((number, item, item_ordered)) = list_item_body(lines[i]) else {
                    break;
                };
                if item_ordered != ordered {
                    break;
                }
                if let Some(n) = number {
                    body.push_str(&format!("<li value=\"{n}\">"));
                } else {
                    body.push_str("<li>");
                }
                render_inline_markdown(item, &mut body);
                body.push_str("</li>");
                i += 1;
            }
            body.push_str(&format!("</{tag}>"));
            continue;
        }
        render_inline_markdown(lines[i], &mut body);
        i += 1;
        if i < lines.len() {
            body.push_str("<br>");
        }
    }
    format!("<div style=\"white-space:pre-wrap\">{body}</div>")
}

fn set_style(
    out: &mut String,
    bold: &mut bool,
    italic: &mut bool,
    want_bold: bool,
    want_italic: bool,
) {
    if *bold == want_bold && *italic == want_italic {
        return;
    }
    if *italic {
        out.push_str("</em>");
    }
    if *bold {
        out.push_str("</strong>");
    }
    if want_bold {
        out.push_str("<strong>");
    }
    if want_italic {
        out.push_str("<em>");
    }
    *bold = want_bold;
    *italic = want_italic;
}

fn append_text(out: &mut String, ch: char) {
    match ch {
        '&' => out.push_str("&amp;"),
        '<' => out.push_str("&lt;"),
        '>' => out.push_str("&gt;"),
        '"' => out.push_str("&quot;"),
        _ => out.push(ch),
    }
}

#[cfg(test)]
mod markup_tests {
    use super::*;

    #[test]
    fn markdown_from_runs_keeps_bold_italic_and_newlines() {
        let md = markdown_from_runs(&[
            StyleRun {
                text: "Hello".into(),
                bold: true,
                italic: false,
            },
            StyleRun {
                text: "\n".into(),
                bold: false,
                italic: false,
            },
            StyleRun {
                text: "world".into(),
                bold: false,
                italic: true,
            },
        ]);
        assert_eq!(md, "**Hello**\n*world*");
        assert_eq!(font_name_traits("Helvetica-Bold"), (true, false));
        assert_eq!(font_name_traits("Times-Italic"), (false, true));
    }

    #[test]
    fn html_from_constrained_markdown_marks_escapes_and_keeps_indents() {
        let bold = html_from_constrained_markdown("**Hello**");
        assert!(bold.contains("<strong>Hello</strong>"));
        assert!(bold.contains("white-space:pre-wrap"));

        let script = html_from_constrained_markdown("<script>alert(1)</script>");
        assert!(!script.contains("<script"));
        assert!(script.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(!script.contains("javascript:"));

        let js = html_from_constrained_markdown("javascript:alert(1)");
        assert!(js.contains("javascript:alert(1)"));
        assert!(!js.contains("<img"));
        assert!(!js.contains(" href="));

        let breaks = html_from_constrained_markdown("a\nb");
        assert!(breaks.contains("a<br>b"));

        let indented = html_from_constrained_markdown("    hello\tworld");
        assert!(indented.starts_with("<div style=\"white-space:pre-wrap\">"));
        assert!(indented.contains("    hello\tworld"));

        let both = html_from_constrained_markdown("***Hi***");
        assert!(both.contains("<strong><em>Hi</em></strong>"));
        let escaped = html_from_constrained_markdown("\\*not\\*");
        assert!(escaped.contains("*not*"));
        assert!(!escaped.contains("<em>"));

        let list = html_from_constrained_markdown("- one\n- **two**");
        assert!(list.contains("<ul>"));
        assert!(list.contains("<li>one</li>"));
        assert!(list.contains("<li><strong>two</strong></li>"));
        assert!(!list.contains("<script"));

        let numbered = html_from_constrained_markdown("tegelijk1. Data is)2. Disable");
        assert!(numbered.contains("<ol>"));
        assert!(numbered.contains("<li value=\"1\">Data is)</li>"));
        assert!(numbered.contains("<li value=\"2\">Disable</li>"));
        let smashed_script = html_from_constrained_markdown("tegelijk1. <script>alert(1)</script>");
        assert!(
            smashed_script.contains("<li value=\"1\">&lt;script&gt;alert(1)&lt;/script&gt;</li>")
        );
        assert!(!smashed_script.contains("<script"));
        let paragraph = html_from_constrained_markdown("Schedule blijft paused.Niet alle");
        assert!(paragraph.contains("paused.<br><br>Niet alle"));
    }

    #[test]
    fn restore_smashed_structure_splits_glued_markers_only() {
        let smashed = "één tegelijk1. DataForSEO (klaar)1. Grant is)2. Disable snapshot3. Zet op prod4. Eén dry_run5. Check login6. Unpause snapshot2. Asana\nscheduler.3. OpenRouter\nNiet. Schedule blijft paused.Niet alle";
        let restored = restore_smashed_structure(smashed);
        assert!(restored.contains("tegelijk\n1. DataForSEO"));
        assert!(restored.contains("(klaar)\n1. Grant"));
        assert!(restored.contains("is)\n2. Disable"));
        assert!(restored.contains("snapshot\n3. Zet"));
        assert!(restored.contains("prod\n4. Eén"));
        assert!(restored.contains("dry_run\n5. Check"));
        assert!(restored.contains("login\n6. Unpause"));
        assert!(restored.contains("snapshot\n2. Asana"));
        assert!(restored.contains("scheduler.\n3. OpenRouter"));
        assert!(restored.contains("paused.\n\nNiet alle"));
        assert!(restored.contains("Asana\nscheduler.\n3. OpenRouter"));
        assert!(!restored.contains("tegelijk1."));
        assert!(!restored.contains("paused.Niet"));
        assert_eq!(restore_smashed_structure(&restored), restored);

        assert_eq!(
            restore_smashed_structure("see section 2. Next stays."),
            "see section 2. Next stays."
        );
        assert_eq!(restore_smashed_structure("Hello. World"), "Hello. World");
        assert_eq!(
            restore_smashed_structure("Mr.Smith e.g.The i.e.Next Dr.Who"),
            "Mr.Smith e.g.The i.e.Next Dr.Who"
        );
        assert_eq!(
            restore_smashed_structure("version 1.2 and FE=357, FT=26, DEV=64"),
            "version 1.2 and FE=357, FT=26, DEV=64"
        );
        assert_eq!(
            restore_smashed_structure("1. first\n2. second\n- bullet"),
            "1. first\n2. second\n- bullet"
        );
    }
}
