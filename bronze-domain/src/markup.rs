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

fn list_item_body(line: &str) -> Option<(&str, bool)> {
    let trimmed = line.trim_start();
    if let Some(rest) = trimmed.strip_prefix("- ") {
        return Some((rest, false));
    }
    if let Some(rest) = trimmed.strip_prefix("+ ") {
        return Some((rest, false));
    }
    if let Some(rest) = trimmed.strip_prefix("• ") {
        return Some((rest, false));
    }
    if let Some(rest) = trimmed.strip_prefix("* ") {
        return Some((rest, false));
    }
    let digits = trimmed.bytes().take_while(u8::is_ascii_digit).count();
    if digits > 0 {
        let after = &trimmed[digits..];
        if let Some(rest) = after.strip_prefix(". ") {
            return Some((rest, true));
        }
    }
    None
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
    let mut body = String::new();
    let lines: Vec<&str> = md.split('\n').collect();
    let mut i = 0;
    while i < lines.len() {
        if let Some((_, ordered)) = list_item_body(lines[i]) {
            let tag = if ordered { "ol" } else { "ul" };
            body.push_str(&format!("<{tag}>"));
            while i < lines.len() {
                let Some((item, item_ordered)) = list_item_body(lines[i]) else {
                    break;
                };
                if item_ordered != ordered {
                    break;
                }
                body.push_str("<li>");
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
    }
}
