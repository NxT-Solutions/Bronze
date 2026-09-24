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

fn marker_gap(ch: char) -> bool {
    ch.is_whitespace() && ch != '\n' && ch != '\r' && ch != '\u{2028}' && ch != '\u{2029}'
}

fn glued_list_marker(chars: &[char], index: usize) -> Option<usize> {
    // A bold marker sits against its number (`**1. Title**`). That is
    // markup, not a list token glued to the previous word.
    if index == 0 || chars[index - 1].is_whitespace() || chars[index - 1] == '*' {
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

#[derive(Clone, Debug)]
enum LineKind {
    Blank,
    Prose,
    Ol {
        n: u32,
        text: String,
        source_bold: bool,
    },
    Ul {
        text: String,
    },
}

struct Line {
    raw: String,
    indent: usize,
    kind: LineKind,
    depth: usize,
    heading: bool,
}

enum Block {
    Prose(String),
    List(ListBlock),
}

struct ListBlock {
    ordered: bool,
    items: Vec<Item>,
}

struct Item {
    number: Option<u32>,
    heading: bool,
    text: String,
    children: Vec<Block>,
}

fn leading_indent(raw: &str) -> (usize, usize) {
    let mut spaces = 0usize;
    let mut bytes = 0usize;
    for ch in raw.chars() {
        if ch == ' ' {
            spaces += 1;
            bytes += 1;
        } else if ch == '\t' {
            spaces += 4;
            bytes += 1;
        } else {
            break;
        }
    }
    (spaces, bytes)
}

fn depth_from_indent(indent: usize) -> usize {
    if indent < 2 {
        0
    } else {
        (indent / 4).max(1)
    }
}

fn strip_wrapping_bold(s: &str) -> Option<&str> {
    let inner = s.strip_prefix("**")?.strip_suffix("**")?;
    if inner.is_empty() || inner.contains("**") {
        None
    } else {
        Some(inner)
    }
}

fn numbered_marker(s: &str) -> Option<(u32, String)> {
    let digits = s.bytes().take_while(u8::is_ascii_digit).count();
    if digits == 0 || digits > 9 {
        return None;
    }
    let after = &s[digits..];
    let rest = after.strip_prefix('.')?;
    if !rest.chars().next().is_some_and(marker_gap) {
        return None;
    }
    let n = s[..digits].parse().ok()?;
    Some((n, rest.trim_start().to_string()))
}

fn bullet_marker(s: &str) -> Option<String> {
    for prefix in ["- ", "+ ", "• ", "* "] {
        if let Some(body) = s.strip_prefix(prefix) {
            return Some(body.to_string());
        }
    }
    None
}

fn classify_line(raw_line: &str) -> Line {
    let raw = raw_line.strip_suffix('\r').unwrap_or(raw_line).to_string();
    let (indent, bytes) = leading_indent(&raw);
    let rest = &raw[bytes..];
    if rest.trim().is_empty() {
        return Line {
            raw,
            indent,
            kind: LineKind::Blank,
            depth: 0,
            heading: false,
        };
    }
    let trimmed = rest.trim();
    if let Some(inner) = strip_wrapping_bold(trimmed) {
        if let Some((n, text)) = numbered_marker(inner) {
            return Line {
                raw,
                indent,
                kind: LineKind::Ol {
                    n,
                    text,
                    source_bold: true,
                },
                depth: 0,
                heading: false,
            };
        }
    }
    if let Some((n, text)) = numbered_marker(trimmed) {
        return Line {
            raw,
            indent,
            kind: LineKind::Ol {
                n,
                text,
                source_bold: false,
            },
            depth: 0,
            heading: false,
        };
    }
    if let Some(text) = bullet_marker(trimmed) {
        return Line {
            raw,
            indent,
            kind: LineKind::Ul { text },
            depth: 0,
            heading: false,
        };
    }
    Line {
        raw,
        indent,
        kind: LineKind::Prose,
        depth: 0,
        heading: false,
    }
}

fn ol_number(line: &Line) -> Option<u32> {
    match line.kind {
        LineKind::Ol { n, .. } => Some(n),
        _ => None,
    }
}

fn is_ol(line: &Line) -> bool {
    matches!(line.kind, LineKind::Ol { .. })
}

fn is_ul(line: &Line) -> bool {
    matches!(line.kind, LineKind::Ul { .. })
}

/// A number greater than the open item stays on that level. A restart at 1
/// nests. A smaller number returns to the ancestor it continues.
fn push_number(stack: &mut Vec<u32>, n: u32) -> usize {
    if stack.is_empty() {
        stack.push(n);
        return 0;
    }
    if n > stack[stack.len() - 1] {
        let last = stack.len() - 1;
        stack[last] = n;
        return last;
    }
    if n == 1 {
        stack.push(1);
        return stack.len() - 1;
    }
    while stack.len() > 1 {
        stack.pop();
        if n > stack[stack.len() - 1] {
            let last = stack.len() - 1;
            stack[last] = n;
            return last;
        }
        if n == 1 {
            stack.push(1);
            return stack.len() - 1;
        }
    }
    stack[0] = n;
    0
}

fn mark_numbered_headings(lines: &mut [Line]) {
    let ol_idx: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| is_ol(line))
        .map(|(idx, _)| idx)
        .collect();
    for (pos, &i) in ol_idx.iter().enumerate() {
        if matches!(
            lines[i].kind,
            LineKind::Ol {
                source_bold: true,
                ..
            }
        ) {
            lines[i].heading = true;
        }
        let depth = lines[i].depth;
        let mut has_child = false;
        for &j in &ol_idx[pos + 1..] {
            if lines[j].depth > depth {
                has_child = true;
                break;
            }
            if lines[j].depth <= depth {
                break;
            }
        }
        if has_child {
            lines[i].heading = true;
            continue;
        }
        let Some(&prev) = ol_idx[..pos]
            .iter()
            .rev()
            .find(|&&j| lines[j].depth == depth)
        else {
            continue;
        };
        let shallower = ol_idx[..pos]
            .iter()
            .any(|&j| j > prev && lines[j].depth < depth);
        if shallower || !lines[prev].heading {
            continue;
        }
        if ol_number(&lines[i]) == ol_number(&lines[prev]).map(|n| n.saturating_add(1)) {
            lines[i].heading = true;
        }
    }
}

fn lift_flush_under_bold(lines: &mut [Line]) {
    let mut open: Option<usize> = None;
    for line in lines.iter_mut() {
        if matches!(line.kind, LineKind::Blank) {
            continue;
        }
        if matches!(
            line.kind,
            LineKind::Ol {
                source_bold: true,
                ..
            }
        ) {
            open = Some(line.depth);
            line.heading = true;
            continue;
        }
        let Some(h) = open else {
            continue;
        };
        if line.depth > h {
            continue;
        }
        line.depth = h + 1;
        line.heading = false;
    }
}

fn attach_followers(lines: &mut [Line]) {
    let mut open: Option<usize> = None;
    for line in lines.iter_mut() {
        match line.kind {
            LineKind::Blank => {}
            LineKind::Ol { .. } if line.heading => {
                open = Some(line.depth);
            }
            LineKind::Ol { .. } => {
                if let Some(h) = open {
                    if line.depth <= h {
                        open = None;
                    }
                }
            }
            LineKind::Ul { .. } | LineKind::Prose => {
                if let Some(h) = open {
                    if line.depth <= h {
                        line.depth = h + 1;
                    }
                }
            }
        }
    }
}

fn assign_outline(lines: &mut [Line]) {
    let source_lists = lines
        .iter()
        .any(|line| line.indent >= 2 && (is_ol(line) || is_ul(line)));
    let source_bold = lines.iter().any(|line| {
        matches!(
            line.kind,
            LineKind::Ol {
                source_bold: true,
                ..
            }
        )
    });
    if source_lists {
        for line in lines.iter_mut() {
            if !matches!(line.kind, LineKind::Blank) {
                line.depth = depth_from_indent(line.indent);
            }
        }
    } else {
        let mut stack = Vec::new();
        for line in lines.iter_mut() {
            if let LineKind::Ol { n, .. } = line.kind {
                line.depth = push_number(&mut stack, n);
            }
        }
    }
    if source_bold {
        lift_flush_under_bold(lines);
    }
    mark_numbered_headings(lines);
    attach_followers(lines);
}

fn prose_text(line: &Line) -> String {
    if line.depth == 0 {
        line.raw.clone()
    } else {
        line.raw.trim().to_string()
    }
}

fn item_text(line: &Line) -> String {
    match &line.kind {
        LineKind::Ol { text, .. } | LineKind::Ul { text } => text.clone(),
        _ => String::new(),
    }
}

fn parse_blocks(lines: &[Line], i: &mut usize, min_depth: usize) -> Vec<Block> {
    let mut blocks = Vec::new();
    while *i < lines.len() {
        if matches!(lines[*i].kind, LineKind::Blank) {
            if min_depth == 0 {
                blocks.push(Block::Prose(String::new()));
                *i += 1;
                continue;
            }
            let mut k = *i;
            while k < lines.len() && matches!(lines[k].kind, LineKind::Blank) {
                k += 1;
            }
            if k >= lines.len() || lines[k].depth < min_depth {
                break;
            }
            blocks.push(Block::Prose(String::new()));
            *i += 1;
            continue;
        }
        if lines[*i].depth < min_depth {
            break;
        }
        if is_ol(&lines[*i]) || is_ul(&lines[*i]) {
            let depth = lines[*i].depth;
            let ordered = is_ol(&lines[*i]);
            let mut items = Vec::new();
            while *i < lines.len() {
                let cur = &lines[*i];
                let same_kind = if ordered { is_ol(cur) } else { is_ul(cur) };
                if cur.depth != depth || !same_kind || matches!(cur.kind, LineKind::Blank) {
                    break;
                }
                let number = ol_number(cur);
                let heading = cur.heading;
                let text = item_text(cur);
                *i += 1;
                let children = parse_blocks(lines, i, depth + 1);
                items.push(Item {
                    number,
                    heading,
                    text,
                    children,
                });
            }
            if items.is_empty() {
                break;
            }
            blocks.push(Block::List(ListBlock { ordered, items }));
            continue;
        }
        blocks.push(Block::Prose(prose_text(&lines[*i])));
        *i += 1;
    }
    blocks
}

fn outline_blocks(text: &str) -> Vec<Block> {
    let mut lines: Vec<Line> = text.split('\n').map(classify_line).collect();
    assign_outline(&mut lines);
    let mut i = 0;
    parse_blocks(&lines, &mut i, 0)
}

fn render_list(list: &ListBlock, out: &mut String, nested: bool) {
    let tag = if list.ordered { "ol" } else { "ul" };
    if nested {
        out.push_str(&format!(
            "<{tag} style=\"padding-inline-start:1.25em;font-weight:400\">"
        ));
    } else {
        out.push_str(&format!("<{tag}>"));
    }
    for item in &list.items {
        if let Some(n) = item.number {
            if item.heading {
                out.push_str(&format!("<li value=\"{n}\" style=\"font-weight:650\">"));
                out.push_str("<strong>");
                render_inline_markdown(&item.text, out);
                out.push_str("</strong>");
            } else {
                out.push_str(&format!("<li value=\"{n}\">"));
                render_inline_markdown(&item.text, out);
            }
        } else {
            out.push_str("<li>");
            render_inline_markdown(&item.text, out);
        }
        render_children(&item.children, out);
        out.push_str("</li>");
    }
    out.push_str(&format!("</{tag}>"));
}

fn render_children(blocks: &[Block], out: &mut String) {
    for block in blocks {
        match block {
            Block::Prose(text) if text.is_empty() => {}
            Block::Prose(text) => {
                out.push_str("<p style=\"margin:0;padding-inline-start:1.25em;font-weight:400\">");
                render_inline_markdown(text, out);
                out.push_str("</p>");
            }
            Block::List(list) => render_list(list, out, true),
        }
    }
}

fn render_html_blocks(blocks: &[Block]) -> String {
    let mut body = String::new();
    for (idx, block) in blocks.iter().enumerate() {
        match block {
            Block::Prose(text) => {
                render_inline_markdown(text, &mut body);
                if idx + 1 < blocks.len() {
                    body.push_str("<br>");
                }
            }
            Block::List(list) => render_list(list, &mut body, false),
        }
    }
    format!("<div style=\"white-space:pre-wrap\">{body}</div>")
}

fn write_plain(blocks: &[Block], depth: usize, out: &mut String) {
    let indent = "    ".repeat(depth);
    for block in blocks {
        match block {
            Block::Prose(text) if text.is_empty() => out.push('\n'),
            Block::Prose(text) => {
                if depth > 0 {
                    out.push_str(&indent);
                    out.push_str(text.trim());
                } else {
                    out.push_str(text);
                }
                out.push('\n');
            }
            Block::List(list) => {
                for item in &list.items {
                    out.push_str(&indent);
                    if list.ordered {
                        let n = item.number.unwrap_or(1);
                        if item.heading {
                            out.push_str(&format!("**{n}. {}**", item.text));
                        } else {
                            out.push_str(&format!("{n}. {}", item.text));
                        }
                    } else {
                        out.push_str("- ");
                        out.push_str(&item.text);
                    }
                    out.push('\n');
                    write_plain(&item.children, depth + 1, out);
                }
            }
        }
    }
}

pub fn outline_captured_text(md: &str) -> String {
    let normalized = restore_smashed_structure(md);
    let blocks = outline_blocks(&normalized);
    let mut out = String::new();
    write_plain(&blocks, 0, &mut out);
    while out.ends_with('\n') {
        out.pop();
    }
    out
}

pub fn html_from_constrained_markdown(md: &str) -> String {
    let normalized = restore_smashed_structure(md);
    render_html_blocks(&outline_blocks(&normalized))
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

    #[test]
    fn outline_restart_is_a_strong_heading_and_children_keep_numbers() {
        let src = "Jij kiest welk pad1. DataForSEO (klaar om te knippen)1. Grant roles2. Disable Windmill3. Zet op prod4. Eén dry_run5. Check login6. Unpause snapshot2. Asana (Rutger: low risk)\n--dry-run op prod → FE=357, FT=26, DEV=64 scheduler.3. OpenRouter\nNiet. Eerst management key. Schedule blijft paused.Niet alle drie";
        let html = html_from_constrained_markdown(src);
        assert!(html.contains("<strong>DataForSEO (klaar om te knippen)</strong>"));
        assert!(html.contains("<strong>Asana (Rutger: low risk)</strong>"));
        assert!(html.contains("<strong>OpenRouter</strong>"));
        assert!(html.contains("<li value=\"1\" style=\"font-weight:650\">"));
        assert!(html.contains("<li value=\"2\" style=\"font-weight:650\">"));
        assert!(html.contains("<li value=\"3\" style=\"font-weight:650\">"));
        assert!(html.contains("<li value=\"1\">Grant roles</li>"));
        assert!(html.contains("<li value=\"6\">Unpause snapshot</li>"));
        assert!(html.contains("padding-inline-start:1.25em"));
        assert!(html.contains("--dry-run op prod → FE=357, FT=26, DEV=64 scheduler."));
        assert!(html.contains("Niet. Eerst management key. Schedule blijft paused."));
        assert!(html.contains("Niet alle drie"));
        assert!(!html.contains("<strong>Grant"));
        assert!(!html.contains("value=\"7\""));
        assert!(!html.contains("<script"));
        let plain = outline_captured_text(src);
        assert!(plain.contains("**1. DataForSEO (klaar om te knippen)**"));
        assert!(plain.contains("\n    1. Grant roles"));
        assert!(plain.contains("**2. Asana (Rutger: low risk)**"));
        assert!(plain.contains("\n    --dry-run op prod"));
        assert!(plain.contains("**3. OpenRouter**"));
        assert_eq!(outline_captured_text(&plain), plain);
        let again = html_from_constrained_markdown(&plain);
        assert!(again.contains("<strong>DataForSEO (klaar om te knippen)</strong>"));
        assert!(again.contains("<li value=\"1\">Grant roles</li>"));
        assert!(!again.contains("<strong>2. Asana"));
        assert!(again.contains("<strong>Asana (Rutger: low risk)</strong>"));

        let flat = html_from_constrained_markdown("1. first\n2. second");
        assert!(flat.contains("<li value=\"1\">first</li>"));
        assert!(flat.contains("<li value=\"2\">second</li>"));
        assert!(!flat.contains("<strong>"));
        assert!(!flat.contains("font-weight:650"));
        assert_eq!(
            outline_captured_text("1. first\n2. second\n- bullet"),
            "1. first\n2. second\n- bullet"
        );
        let prose = html_from_constrained_markdown("see section 2. Next stays.");
        assert!(!prose.contains("<ol"));
        assert!(!prose.contains("<strong>"));
        assert!(prose.contains("see section 2. Next stays."));
        assert!(html_from_constrained_markdown("version 1.2 and FE=357").contains("version 1.2"));
        assert!(!html_from_constrained_markdown("Hello. World").contains("<ol"));
        let kept = html_from_constrained_markdown("**1. Title**\nbody line\n**2. Next**\nmore");
        assert!(kept.contains("<strong>Title</strong>"));
        assert!(kept.contains("<strong>Next</strong>"));
        assert!(kept.contains("padding-inline-start:1.25em"));
        assert!(kept.contains(">body line</p>"));
        assert!(kept.contains(">more</p>"));
        assert!(
            kept.find("<strong>Title</strong>").unwrap() < kept.find(">body line</p>").unwrap()
        );
        assert!(kept.find(">body line</p>").unwrap() < kept.find("<strong>Next</strong>").unwrap());
    }
}
