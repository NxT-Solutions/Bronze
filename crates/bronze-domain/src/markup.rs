//! Constrained markdown for captured attributed text.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StyleRun {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub code: bool,
}

/// One attributed run, measured in Unicode scalars. `background` is an opaque
/// color id: equal ids are the same color. `highlight` is `AXHighlight`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MarkSpan {
    pub len: usize,
    pub highlight: bool,
    pub background: Option<u32>,
}

/// `AXHighlight`, or a background that is not the selection's shared
/// background, is an inline mark. A uniform background stays plain.
pub fn inline_code_flags(spans: &[MarkSpan]) -> Vec<bool> {
    let shared = shared_background(spans);
    spans
        .iter()
        .map(|span| {
            span.highlight
                || match shared {
                    SharedBackground::One(body) => {
                        span.background.is_some() && span.background != body
                    }
                    SharedBackground::Ambiguous => false,
                }
        })
        .collect()
}

#[derive(Clone, Copy)]
enum SharedBackground {
    One(Option<u32>),
    Ambiguous,
}

fn shared_background(spans: &[MarkSpan]) -> SharedBackground {
    let mut counts: Vec<(Option<u32>, usize)> = Vec::new();
    for span in spans {
        if span.len == 0 {
            continue;
        }
        if let Some(slot) = counts
            .iter_mut()
            .find(|(color, _)| *color == span.background)
        {
            slot.1 += span.len;
        } else {
            counts.push((span.background, span.len));
        }
    }
    let max = counts.iter().map(|(_, n)| *n).max().unwrap_or(0);
    if max == 0 {
        return SharedBackground::One(None);
    }
    let leaders: Vec<Option<u32>> = counts
        .into_iter()
        .filter(|(_, n)| *n == max)
        .map(|(color, _)| color)
        .collect();
    if leaders.len() == 1 {
        SharedBackground::One(leaders[0])
    } else if leaders.contains(&None) {
        SharedBackground::One(None)
    } else {
        SharedBackground::Ambiguous
    }
}

pub fn markdown_from_runs(runs: &[StyleRun]) -> String {
    let mut out = String::new();
    let mut pending: Option<StyleRun> = None;
    for run in runs {
        if run.text.is_empty() {
            continue;
        }
        if let Some(acc) = pending.as_mut() {
            if acc.bold == run.bold && acc.italic == run.italic && acc.code == run.code {
                acc.text.push_str(&run.text);
                continue;
            }
            push_run(&mut out, acc);
        }
        pending = Some(run.clone());
    }
    if let Some(acc) = pending {
        push_run(&mut out, &acc);
    }
    out
}

fn push_run(out: &mut String, run: &StyleRun) {
    let code = run.code && !run.text.chars().all(char::is_whitespace);
    let escaped = escape_inline(&run.text);
    let body = if code {
        format!("`{escaped}`")
    } else {
        escaped
    };
    match (run.bold, run.italic) {
        (true, true) => {
            out.push_str("***");
            out.push_str(&body);
            out.push_str("***");
        }
        (true, false) => {
            out.push_str("**");
            out.push_str(&body);
            out.push_str("**");
        }
        (false, true) => {
            out.push('*');
            out.push_str(&body);
            out.push('*');
        }
        (false, false) => out.push_str(&body),
    }
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

const CODE_OPEN: &str =
    "<code style=\"background-color:#f4f4f5;border-radius:4px;padding:0 0.2em\">";

fn render_inline_markdown(md: &str, body: &mut String) {
    let chars: Vec<char> = md.chars().collect();
    let mut i = 0;
    let mut bold = false;
    let mut italic = false;
    let mut code = false;
    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            let next = chars[i + 1];
            if next == '\\' || next == '*' || next == '`' {
                append_text(body, next);
                i += 2;
                continue;
            }
        }
        if chars[i] == '`' && !code {
            if let Some(close) = find_unescaped_backtick(&chars, i + 1) {
                if close > i + 1 {
                    let keep_bold = bold;
                    let keep_italic = italic;
                    set_style(
                        body,
                        &mut bold,
                        &mut italic,
                        &mut code,
                        keep_bold,
                        keep_italic,
                        true,
                    );
                    let mut inner = i + 1;
                    while inner < close {
                        if chars[inner] == '\\'
                            && inner + 1 < close
                            && matches!(chars[inner + 1], '\\' | '*' | '`')
                        {
                            append_text(body, chars[inner + 1]);
                            inner += 2;
                            continue;
                        }
                        append_text(body, chars[inner]);
                        inner += 1;
                    }
                    set_style(
                        body,
                        &mut bold,
                        &mut italic,
                        &mut code,
                        keep_bold,
                        keep_italic,
                        false,
                    );
                    i = close + 1;
                    continue;
                }
            }
        }
        if !code && chars[i] == '*' {
            if matches!(chars.get(i..i + 3), Some(['*', '*', '*'])) {
                let want_bold = !bold;
                let want_italic = !italic;
                let keep_code = code;
                set_style(
                    body,
                    &mut bold,
                    &mut italic,
                    &mut code,
                    want_bold,
                    want_italic,
                    keep_code,
                );
                i += 3;
                continue;
            }
            if matches!(chars.get(i..i + 2), Some(['*', '*'])) {
                let want_bold = !bold;
                let keep_italic = italic;
                let keep_code = code;
                set_style(
                    body,
                    &mut bold,
                    &mut italic,
                    &mut code,
                    want_bold,
                    keep_italic,
                    keep_code,
                );
                i += 2;
                continue;
            }
            let keep_bold = bold;
            let want_italic = !italic;
            let keep_code = code;
            set_style(
                body,
                &mut bold,
                &mut italic,
                &mut code,
                keep_bold,
                want_italic,
                keep_code,
            );
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
    set_style(body, &mut bold, &mut italic, &mut code, false, false, false);
}

fn find_unescaped_backtick(chars: &[char], start: usize) -> Option<usize> {
    let mut i = start;
    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() && matches!(chars[i + 1], '\\' | '*' | '`') {
            i += 2;
            continue;
        }
        if chars[i] == '`' {
            return Some(i);
        }
        i += 1;
    }
    None
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
    code: &mut bool,
    want_bold: bool,
    want_italic: bool,
    want_code: bool,
) {
    if *bold == want_bold && *italic == want_italic && *code == want_code {
        return;
    }
    if *code && (*code != want_code || *italic != want_italic || *bold != want_bold) {
        out.push_str("</code>");
        *code = false;
    }
    if *italic && (*italic != want_italic || *bold != want_bold) {
        out.push_str("</em>");
        *italic = false;
    }
    if *bold && *bold != want_bold {
        out.push_str("</strong>");
        *bold = false;
    }
    if want_bold && !*bold {
        out.push_str("<strong>");
        *bold = true;
    }
    if want_italic && !*italic {
        out.push_str("<em>");
        *italic = true;
    }
    if want_code && !*code {
        out.push_str(CODE_OPEN);
        *code = true;
    }
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

const CLIPBOARD_MARKUP_MAX_BYTES: usize = 1 << 20;

#[derive(Clone, Copy, Default, Eq, PartialEq)]
struct InlineStyle {
    bold: bool,
    italic: bool,
    code: bool,
}

/// Unescaped `**` or `` ` `` means attributed marks, not a lone `*`.
pub fn markdown_has_style_marks(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            i += 2;
            continue;
        }
        if chars[i] == '`' {
            return true;
        }
        if chars[i] == '*' && chars.get(i + 1) == Some(&'*') {
            return true;
        }
        i += 1;
    }
    false
}

pub fn markdown_from_clipboard_types(
    html: Option<&str>,
    rtf: Option<&str>,
    plain: Option<&str>,
) -> Option<String> {
    if let Some(html) = html.filter(|s| !s.is_empty()) {
        let md = constrained_markdown_from_html(html);
        if !md.is_empty() {
            return Some(md);
        }
    }
    if let Some(rtf) = rtf.filter(|s| !s.is_empty()) {
        let md = constrained_markdown_from_rtf(rtf);
        if !md.is_empty() {
            return Some(md);
        }
    }
    plain.filter(|s| !s.is_empty()).map(str::to_string)
}

pub fn capture_body_preferring_ax_marks(ax_text: &str, clipboard_md: Option<&str>) -> String {
    if markdown_has_style_marks(ax_text) {
        return ax_text.to_string();
    }
    match clipboard_md {
        Some(md) if markdown_has_style_marks(md) => md.to_string(),
        _ => ax_text.to_string(),
    }
}

pub fn constrained_markdown_from_html(html: &str) -> String {
    if html.is_empty() || html.len() > CLIPBOARD_MARKUP_MAX_BYTES {
        return String::new();
    }
    markdown_from_runs(&html_style_runs(html))
}

pub fn constrained_markdown_from_rtf(rtf: &str) -> String {
    if rtf.is_empty() || rtf.len() > CLIPBOARD_MARKUP_MAX_BYTES {
        return String::new();
    }
    markdown_from_runs(&rtf_style_runs(rtf))
}

fn html_style_runs(html: &str) -> Vec<StyleRun> {
    let bytes = html.as_bytes();
    let mut i = 0;
    let mut style = InlineStyle::default();
    let mut stack = vec![style];
    let mut skip = 0u32;
    let mut pre = 0u32;
    let mut lists: Vec<(bool, u32)> = Vec::new();
    let mut runs = Vec::new();
    let mut pending = String::new();
    let mut pending_style = InlineStyle::default();
    let mut last_was_ws = true;
    while i < bytes.len() {
        if bytes[i] == b'<' {
            if bytes.get(i + 1..i + 4) == Some(b"!--") {
                i += 4;
                while i + 2 < bytes.len() && &bytes[i..i + 3] != b"-->" {
                    i += 1;
                }
                i = (i + 3).min(bytes.len());
                continue;
            }
            flush_html_run(&mut runs, &mut pending, pending_style);
            let (next, tag) = parse_html_tag(bytes, i);
            if next <= i {
                i += 1;
                continue;
            }
            i = next;
            match tag {
                HtmlTag::Start {
                    name,
                    class,
                    style_attr,
                    self_closing,
                } => {
                    apply_html_start(
                        &name,
                        &class,
                        &style_attr,
                        self_closing,
                        &mut style,
                        &mut stack,
                        &mut skip,
                        &mut pre,
                        &mut lists,
                        &mut runs,
                        &mut pending,
                        &mut pending_style,
                        &mut last_was_ws,
                    );
                }
                HtmlTag::End { name } => {
                    apply_html_end(
                        &name,
                        &mut style,
                        &mut stack,
                        &mut skip,
                        &mut pre,
                        &mut lists,
                        &mut last_was_ws,
                    );
                }
            }
            continue;
        }
        let start = i;
        while i < bytes.len() && bytes[i] != b'<' {
            i += 1;
        }
        if skip == 0 {
            if let Ok(raw) = std::str::from_utf8(&bytes[start..i]) {
                push_html_text(
                    &mut runs,
                    &mut pending,
                    &mut pending_style,
                    &decode_entities(raw),
                    style,
                    pre > 0,
                    &mut last_was_ws,
                );
            }
        }
    }
    flush_html_run(&mut runs, &mut pending, pending_style);
    runs
}

enum HtmlTag {
    Start {
        name: String,
        class: String,
        style_attr: String,
        self_closing: bool,
    },
    End {
        name: String,
    },
}

fn is_html_name_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b':'
}

fn parse_html_tag(bytes: &[u8], start: usize) -> (usize, HtmlTag) {
    let mut i = start + 1;
    let end_tag = bytes.get(i) == Some(&b'/');
    if end_tag {
        i += 1;
    }
    let name_start = i;
    while i < bytes.len() && is_html_name_byte(bytes[i]) {
        i += 1;
    }
    let name = std::str::from_utf8(&bytes[name_start..i])
        .unwrap_or("")
        .to_ascii_lowercase();
    let mut class = String::new();
    let mut style_attr = String::new();
    let mut self_closing = false;
    while i < bytes.len() && bytes[i] != b'>' {
        if bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'>') {
            self_closing = true;
            i += 1;
            break;
        }
        if bytes[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        let key_start = i;
        while i < bytes.len() && is_html_name_byte(bytes[i]) {
            i += 1;
        }
        if i == key_start {
            i += 1;
            continue;
        }
        let key = std::str::from_utf8(&bytes[key_start..i])
            .unwrap_or("")
            .to_ascii_lowercase();
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let mut value = String::new();
        if bytes.get(i) == Some(&b'=') {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if let Some(&q) = bytes.get(i) {
                if q == b'"' || q == b'\'' {
                    i += 1;
                    let v0 = i;
                    while i < bytes.len() && bytes[i] != q {
                        i += 1;
                    }
                    value = std::str::from_utf8(&bytes[v0..i]).unwrap_or("").to_string();
                    if i < bytes.len() {
                        i += 1;
                    }
                } else {
                    let v0 = i;
                    while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'>' {
                        i += 1;
                    }
                    value = std::str::from_utf8(&bytes[v0..i]).unwrap_or("").to_string();
                }
            }
        }
        if key == "class" {
            class = value;
        } else if key == "style" {
            style_attr = value;
        }
    }
    if i < bytes.len() && bytes[i] == b'>' {
        i += 1;
    }
    if matches!(
        name.as_str(),
        "br" | "img" | "hr" | "meta" | "link" | "input" | "source" | "wbr" | "col"
    ) {
        self_closing = true;
    }
    if end_tag {
        (i, HtmlTag::End { name })
    } else {
        (
            i,
            HtmlTag::Start {
                name,
                class,
                style_attr,
                self_closing,
            },
        )
    }
}

fn html_skip_tag(name: &str) -> bool {
    matches!(
        name,
        "script"
            | "style"
            | "noscript"
            | "iframe"
            | "object"
            | "embed"
            | "svg"
            | "template"
            | "head"
            | "link"
            | "meta"
    )
}

fn html_void_drop(name: &str) -> bool {
    matches!(name, "img" | "source" | "track" | "input" | "hr")
}

fn html_code_hint(name: &str, class: &str, style_attr: &str) -> bool {
    if matches!(name, "code" | "pre" | "kbd" | "samp" | "tt") {
        return true;
    }
    let class_l = class.to_ascii_lowercase();
    for token in class_l.split(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '-') {
        if matches!(
            token,
            "code" | "pre" | "kbd" | "monospace" | "hljs" | "token" | "prettyprint"
        ) || token.contains("mrkdwn__code")
            || token.contains("mrkdwn__pre")
        {
            return true;
        }
    }
    font_family_is_mono(style_attr)
}

fn font_family_is_mono(style_attr: &str) -> bool {
    let style_l = style_attr.to_ascii_lowercase();
    let Some(fam) = style_prop(&style_l, "font-family") else {
        return false;
    };
    fam.contains("mono")
        || fam.contains("courier")
        || fam.contains("menlo")
        || fam.contains("monaco")
        || fam.contains("consolas")
        || fam.contains("cascadia")
        || fam.contains("fira")
        || fam.contains("jetbrains")
        || fam.contains("source code")
        || fam.contains("sf mono")
        || fam.contains("sfmono")
}

fn style_prop<'a>(style: &'a str, name: &str) -> Option<&'a str> {
    let needle = format!("{name}:");
    let start = style.find(&needle)?;
    let rest = style[start + needle.len()..].trim_start();
    let end = rest.find(';').unwrap_or(rest.len());
    Some(rest[..end].trim())
}

#[allow(clippy::too_many_arguments)]
fn apply_html_start(
    name: &str,
    class: &str,
    style_attr: &str,
    self_closing: bool,
    style: &mut InlineStyle,
    stack: &mut Vec<InlineStyle>,
    skip: &mut u32,
    pre: &mut u32,
    lists: &mut Vec<(bool, u32)>,
    runs: &mut Vec<StyleRun>,
    pending: &mut String,
    pending_style: &mut InlineStyle,
    last_was_ws: &mut bool,
) {
    if html_skip_tag(name) {
        if !self_closing {
            *skip = skip.saturating_add(1);
        }
        return;
    }
    if *skip > 0 {
        if !self_closing && html_skip_tag(name) {
            *skip = skip.saturating_add(1);
        }
        return;
    }
    if html_void_drop(name) {
        return;
    }
    if name == "br" {
        push_html_text(
            runs,
            pending,
            pending_style,
            "\n",
            InlineStyle::default(),
            true,
            last_was_ws,
        );
        *last_was_ws = true;
        return;
    }
    if matches!(
        name,
        "p" | "div" | "tr" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "blockquote"
    ) {
        push_html_block_break(runs, pending, pending_style, last_was_ws);
    }
    if name == "ul" || name == "ol" {
        push_html_block_break(runs, pending, pending_style, last_was_ws);
        lists.push((name == "ol", 1));
    }
    if name == "li" {
        push_html_block_break(runs, pending, pending_style, last_was_ws);
        let depth = lists.len().saturating_sub(1);
        let indent = "    ".repeat(depth);
        let marker = match lists.last_mut() {
            Some((true, n)) => {
                let cur = *n;
                *n = n.saturating_add(1);
                format!("{indent}{cur}. ")
            }
            Some((false, _)) => format!("{indent}- "),
            None => format!("{indent}- "),
        };
        push_html_text(
            runs,
            pending,
            pending_style,
            &marker,
            InlineStyle::default(),
            true,
            last_was_ws,
        );
    }
    if self_closing {
        return;
    }
    let mut next = *style;
    if matches!(name, "strong" | "b") {
        next.bold = true;
    }
    if matches!(name, "em" | "i") {
        next.italic = true;
    }
    if html_code_hint(name, class, style_attr) {
        next.code = true;
        if name == "pre" {
            *pre = pre.saturating_add(1);
        }
    }
    stack.push(next);
    *style = next;
}

fn apply_html_end(
    name: &str,
    style: &mut InlineStyle,
    stack: &mut Vec<InlineStyle>,
    skip: &mut u32,
    pre: &mut u32,
    lists: &mut Vec<(bool, u32)>,
    last_was_ws: &mut bool,
) {
    if html_skip_tag(name) {
        *skip = skip.saturating_sub(1);
        return;
    }
    if *skip > 0 {
        return;
    }
    if name == "pre" {
        *pre = pre.saturating_sub(1);
    }
    if name == "ul" || name == "ol" {
        lists.pop();
        *last_was_ws = true;
    }
    if stack.len() > 1 {
        stack.pop();
    }
    *style = stack.last().copied().unwrap_or_default();
}

fn push_html_block_break(
    runs: &mut Vec<StyleRun>,
    pending: &mut String,
    pending_style: &mut InlineStyle,
    last_was_ws: &mut bool,
) {
    if pending.ends_with('\n') {
        *last_was_ws = true;
        return;
    }
    if pending.is_empty() && runs.last().is_some_and(|run| run.text.ends_with('\n')) {
        *last_was_ws = true;
        return;
    }
    if pending.is_empty() && runs.is_empty() {
        *last_was_ws = true;
        return;
    }
    push_html_text(
        runs,
        pending,
        pending_style,
        "\n",
        InlineStyle::default(),
        true,
        last_was_ws,
    );
    *last_was_ws = true;
}

fn push_html_text(
    runs: &mut Vec<StyleRun>,
    pending: &mut String,
    pending_style: &mut InlineStyle,
    text: &str,
    style: InlineStyle,
    pre: bool,
    last_was_ws: &mut bool,
) {
    let mut buf = String::new();
    if pre {
        buf.push_str(text);
        if !text.is_empty() {
            *last_was_ws = text.chars().last().is_some_and(char::is_whitespace);
        }
    } else {
        for ch in text.chars() {
            if ch == '\u{00a0}' {
                buf.push(' ');
                *last_was_ws = true;
            } else if ch.is_whitespace() {
                if !*last_was_ws {
                    buf.push(' ');
                    *last_was_ws = true;
                }
            } else {
                buf.push(ch);
                *last_was_ws = false;
            }
        }
    }
    if buf.is_empty() {
        return;
    }
    if pending.is_empty() {
        *pending_style = style;
        pending.push_str(&buf);
        return;
    }
    if *pending_style == style {
        pending.push_str(&buf);
        return;
    }
    flush_html_run(runs, pending, *pending_style);
    *pending_style = style;
    pending.push_str(&buf);
}

fn flush_html_run(runs: &mut Vec<StyleRun>, pending: &mut String, style: InlineStyle) {
    if pending.is_empty() {
        return;
    }
    runs.push(StyleRun {
        text: std::mem::take(pending),
        bold: style.bold,
        italic: style.italic,
        code: style.code,
    });
}

fn decode_entities(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        rest = &rest[amp..];
        let Some(end) = rest.find(';') else {
            out.push('&');
            rest = &rest[1..];
            continue;
        };
        let ent = &rest[1..end];
        match ent {
            "amp" => out.push('&'),
            "lt" => out.push('<'),
            "gt" => out.push('>'),
            "quot" => out.push('"'),
            "apos" | "#39" | "#x27" => out.push('\''),
            "nbsp" => out.push('\u{00a0}'),
            _ if ent.starts_with('#') => {
                if let Some(ch) = decode_numeric_entity(ent) {
                    out.push(ch);
                }
            }
            _ => {}
        }
        rest = &rest[end + 1..];
    }
    out.push_str(rest);
    out
}

fn decode_numeric_entity(ent: &str) -> Option<char> {
    let digits = ent.strip_prefix('#')?;
    let value = if let Some(hex) = digits
        .strip_prefix('x')
        .or_else(|| digits.strip_prefix('X'))
    {
        u32::from_str_radix(hex, 16).ok()?
    } else {
        digits.parse().ok()?
    };
    char::from_u32(value)
}

fn rtf_style_runs(rtf: &str) -> Vec<StyleRun> {
    let bytes = rtf.as_bytes();
    let mut i = 0;
    let mut fonts = Vec::<(i32, bool)>::new();
    let mut font = 0i32;
    let mut bold = false;
    let mut italic = false;
    let mut highlight = 0i32;
    let mut skip = 0i32;
    let mut stack: Vec<(i32, bool, bool, i32)> = Vec::new();
    let mut runs = Vec::new();
    let mut pending = String::new();
    let mut pending_style = InlineStyle::default();
    let mut last_was_ws = true;
    let mut in_fonttbl = false;
    let mut fonttbl_depth = 0i32;
    let mut dest_skip = 0i32;
    while i < bytes.len() {
        let ch = bytes[i];
        if ch == b'{' {
            stack.push((font, bold, italic, highlight));
            i += 1;
            if in_fonttbl {
                fonttbl_depth += 1;
            }
            if bytes.get(i) == Some(&b'\\') && bytes.get(i + 1) == Some(&b'*') {
                dest_skip += 1;
            }
            continue;
        }
        if ch == b'}' {
            if dest_skip > 0 {
                dest_skip -= 1;
            }
            if in_fonttbl {
                fonttbl_depth -= 1;
                if fonttbl_depth <= 0 {
                    in_fonttbl = false;
                }
            }
            if let Some((f, b, it, h)) = stack.pop() {
                font = f;
                bold = b;
                italic = it;
                highlight = h;
            }
            i += 1;
            continue;
        }
        if ch == b'\\' {
            i += 1;
            if i >= bytes.len() {
                break;
            }
            let next = bytes[i];
            if next == b'\\' || next == b'{' || next == b'}' {
                if dest_skip == 0 && !in_fonttbl {
                    let style = rtf_inline(bold, italic, font, highlight, &fonts);
                    push_html_text(
                        &mut runs,
                        &mut pending,
                        &mut pending_style,
                        std::str::from_utf8(&[next]).unwrap_or(""),
                        style,
                        true,
                        &mut last_was_ws,
                    );
                }
                i += 1;
                continue;
            }
            if next == b'\'' {
                i += 1;
                if i + 1 < bytes.len() {
                    let hex = &rtf[i..i + 2];
                    if dest_skip == 0 && !in_fonttbl {
                        if let Ok(byte) = u8::from_str_radix(hex, 16) {
                            let style = rtf_inline(bold, italic, font, highlight, &fonts);
                            push_html_text(
                                &mut runs,
                                &mut pending,
                                &mut pending_style,
                                &String::from_utf8_lossy(&[byte]),
                                style,
                                true,
                                &mut last_was_ws,
                            );
                        }
                    }
                    i += 2;
                }
                continue;
            }
            let word_start = i;
            while i < bytes.len() && bytes[i].is_ascii_alphabetic() {
                i += 1;
            }
            let word = std::str::from_utf8(&bytes[word_start..i]).unwrap_or("");
            let mut num: Option<i32> = None;
            let num_start = i;
            if bytes.get(i) == Some(&b'-') {
                i += 1;
            }
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            if i > num_start {
                num = std::str::from_utf8(&bytes[num_start..i])
                    .ok()
                    .and_then(|s| s.parse().ok());
            }
            if bytes.get(i) == Some(&b' ') {
                i += 1;
            }
            if dest_skip > 0 {
                continue;
            }
            match word {
                "fonttbl" => {
                    in_fonttbl = true;
                    fonttbl_depth = 1;
                }
                "colortbl" | "stylesheet" | "info" | "pict" | "themedata"
                | "colorschememapping" | "latentstyles" | "datastore" => {
                    dest_skip += 1;
                }
                "f" if in_fonttbl => {
                    if let Some(id) = num {
                        font = id;
                    }
                }
                "fmodern" if in_fonttbl => {
                    upsert_font(&mut fonts, font, true);
                }
                "froman" | "fswiss" | "fscript" | "fdecor" | "ftech" if in_fonttbl => {
                    upsert_font(&mut fonts, font, false);
                }
                "f" => {
                    if let Some(id) = num {
                        font = id;
                    }
                }
                "b" => bold = num.unwrap_or(1) != 0,
                "i" => italic = num.unwrap_or(1) != 0,
                "highlight" => highlight = num.unwrap_or(0),
                "par" | "line" => {
                    let style = rtf_inline(bold, italic, font, highlight, &fonts);
                    push_html_text(
                        &mut runs,
                        &mut pending,
                        &mut pending_style,
                        "\n",
                        style,
                        true,
                        &mut last_was_ws,
                    );
                    last_was_ws = true;
                }
                "tab" => {
                    let style = rtf_inline(bold, italic, font, highlight, &fonts);
                    push_html_text(
                        &mut runs,
                        &mut pending,
                        &mut pending_style,
                        "\t",
                        style,
                        true,
                        &mut last_was_ws,
                    );
                }
                "u" => {
                    if let Some(code) = num {
                        let scalar = if code < 0 {
                            (code as u16) as u32
                        } else {
                            code as u32
                        };
                        if let Some(ch) = char::from_u32(scalar) {
                            let style = rtf_inline(bold, italic, font, highlight, &fonts);
                            let mut tmp = [0u8; 4];
                            push_html_text(
                                &mut runs,
                                &mut pending,
                                &mut pending_style,
                                ch.encode_utf8(&mut tmp),
                                style,
                                true,
                                &mut last_was_ws,
                            );
                        }
                    }
                    skip = 1;
                }
                "bin" => {
                    if let Some(n) = num {
                        i = i.saturating_add(n.max(0) as usize).min(bytes.len());
                    }
                }
                _ => {
                    if in_fonttbl && !word.is_empty() {
                        if let Some(name) = rtf[word_start..].split(';').next() {
                            if font_name_is_mono(name) {
                                upsert_font(&mut fonts, font, true);
                            }
                        }
                    }
                }
            }
            continue;
        }
        if dest_skip > 0 {
            i += 1;
            continue;
        }
        if in_fonttbl {
            let start = i;
            while i < bytes.len() && !matches!(bytes[i], b'\\' | b'{' | b'}' | b';') {
                i += 1;
            }
            if let Ok(name) = std::str::from_utf8(&bytes[start..i]) {
                if font_name_is_mono(name) {
                    upsert_font(&mut fonts, font, true);
                }
            }
            if bytes.get(i) == Some(&b';') {
                i += 1;
            }
            continue;
        }
        if skip > 0 {
            skip -= 1;
            i += 1;
            continue;
        }
        let start = i;
        while i < bytes.len() && !matches!(bytes[i], b'\\' | b'{' | b'}') {
            i += 1;
        }
        if let Ok(text) = std::str::from_utf8(&bytes[start..i]) {
            let style = rtf_inline(bold, italic, font, highlight, &fonts);
            push_html_text(
                &mut runs,
                &mut pending,
                &mut pending_style,
                text,
                style,
                true,
                &mut last_was_ws,
            );
        }
    }
    flush_html_run(&mut runs, &mut pending, pending_style);
    runs
}

fn upsert_font(fonts: &mut Vec<(i32, bool)>, id: i32, mono: bool) {
    if let Some(slot) = fonts.iter_mut().find(|(fid, _)| *fid == id) {
        slot.1 = slot.1 || mono;
    } else {
        fonts.push((id, mono));
    }
}

fn rtf_inline(
    bold: bool,
    italic: bool,
    font: i32,
    highlight: i32,
    fonts: &[(i32, bool)],
) -> InlineStyle {
    let mono = fonts
        .iter()
        .find(|(id, _)| *id == font)
        .is_some_and(|(_, mono)| *mono);
    InlineStyle {
        bold,
        italic,
        code: mono || highlight > 0,
    }
}

fn font_name_is_mono(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.contains("courier")
        || lower.contains("mono")
        || lower.contains("menlo")
        || lower.contains("consolas")
        || lower.contains("monaco")
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
                code: false,
            },
            StyleRun {
                text: "\n".into(),
                bold: false,
                italic: false,
                code: false,
            },
            StyleRun {
                text: "world".into(),
                bold: false,
                italic: true,
                code: false,
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

    #[test]
    fn inline_marks_keep_bold_code_and_plain_prose() {
        let gray = 0x00e4_e4e7_ff;
        let flags = inline_code_flags(&[
            MarkSpan {
                len: 20,
                highlight: false,
                background: None,
            },
            MarkSpan {
                len: 8,
                highlight: false,
                background: Some(gray),
            },
            MarkSpan {
                len: 6,
                highlight: true,
                background: None,
            },
        ]);
        assert_eq!(flags, vec![false, true, true]);
        let uniform = inline_code_flags(&[
            MarkSpan {
                len: 4,
                highlight: false,
                background: Some(gray),
            },
            MarkSpan {
                len: 9,
                highlight: false,
                background: Some(gray),
            },
        ]);
        assert_eq!(uniform, vec![false, false]);
        let tied = inline_code_flags(&[
            MarkSpan {
                len: 4,
                highlight: false,
                background: Some(1),
            },
            MarkSpan {
                len: 4,
                highlight: false,
                background: Some(2),
            },
        ]);
        assert_eq!(tied, vec![false, false]);

        let md = markdown_from_runs(&[
            StyleRun {
                text: "Nieuwe ".into(),
                bold: true,
                italic: false,
                code: false,
            },
            StyleRun {
                text: "openrouter_activity_daily".into(),
                bold: false,
                italic: false,
                code: true,
            },
        ]);
        assert_eq!(md, "**Nieuwe **`openrouter_activity_daily`");
        assert_eq!(
            markdown_from_runs(&[StyleRun {
                text: "a`b".into(),
                bold: false,
                italic: false,
                code: true,
            }]),
            "`a\\`b`"
        );

        let html = html_from_constrained_markdown(
            "1. **Nieuwe OpenRouter management-key** uses `openrouter_activity_daily`",
        );
        assert!(html.contains("<strong>Nieuwe OpenRouter management-key</strong>"));
        assert!(html.contains("<code style=\"background-color:#f4f4f5;border-radius:4px;padding:0 0.2em\">openrouter_activity_daily</code>"));
        assert!(html.contains("<li value=\"1\">"));
        assert!(!html.contains("font-weight:650"));
        assert!(!html.contains("<script"));
        let both = html_from_constrained_markdown("**`v_fact_cost_openrouter`**");
        assert!(both.contains("<strong><code"));
        assert!(both.contains("v_fact_cost_openrouter</code></strong>"));
        let leaked = html_from_constrained_markdown("`<script>alert(1)</script>`");
        assert!(!leaked.contains("<script"));
        assert!(leaked.contains("&lt;script&gt;"));

        assert!(!html_from_constrained_markdown("see section 2. Next").contains("<ol"));
        assert!(!html_from_constrained_markdown("version 1.2").contains("<ol"));
        assert!(!html_from_constrained_markdown("Hello. World").contains("<ol"));
        assert_eq!(
            restore_smashed_structure("see section 2. Next"),
            "see section 2. Next"
        );
        assert_eq!(restore_smashed_structure("version 1.2"), "version 1.2");
        assert_eq!(restore_smashed_structure("Hello. World"), "Hello. World");
    }

    #[test]
    fn outline_keeps_lead_and_paragraph_after_period() {
        let smashed = "Jij kiest welk pad je overzet — één tegelijk1. DataForSEO (klaar om te knippen)1. Grant2. Disable3. Zet4. Eén5. Check6. Unpause2. Asana (Rutger: low risk)\n--dry-run op prod\n3. OpenRouter\nNiet. Eerst management key + finance. Schedule blijft paused.Niet alle drie tegelijk.";
        assert_cursor_outline(smashed);
        let restored = "Jij kiest welk pad je overzet — één tegelijk\n1. DataForSEO (klaar om te knippen)\n1. Grant\n2. Disable\n3. Zet\n4. Eén\n5. Check\n6. Unpause\n2. Asana (Rutger: low risk)\n--dry-run op prod\n3. OpenRouter\nNiet. Eerst management key + finance. Schedule blijft paused.\nNiet alle drie tegelijk.";
        assert_cursor_outline(restored);
        assert_eq!(
            restore_smashed_structure("see section 2. Next"),
            "see section 2. Next"
        );
        assert_eq!(restore_smashed_structure("version 1.2"), "version 1.2");
        assert_eq!(restore_smashed_structure("Hello. World"), "Hello. World");
        assert!(!html_from_constrained_markdown("see section 2. Next").contains("<ol"));
        assert!(!html_from_constrained_markdown("version 1.2").contains("<ol"));
        assert!(!html_from_constrained_markdown("Hello. World").contains("<ol"));
    }

    fn assert_cursor_outline(src: &str) {
        let html = html_from_constrained_markdown(src);
        let plain = outline_captured_text(src);
        for needle in [
            "Jij kiest welk pad je overzet — één tegelijk",
            "DataForSEO (klaar om te knippen)",
            "Asana (Rutger: low risk)",
            "--dry-run",
            "OpenRouter",
            "Niet. Eerst management key + finance. Schedule blijft paused.",
            "Niet alle drie tegelijk.",
        ] {
            assert!(html.contains(needle), "display missing {needle}: {html}");
            assert!(plain.contains(needle), "copy missing {needle}: {plain}");
        }
        assert!(html.contains("<li value=\"1\">Grant</li>"), "{html}");
        assert!(html.contains("<li value=\"6\">Unpause</li>"), "{html}");
        assert!(plain.contains("1. Grant"), "{plain}");
        assert!(plain.contains("6. Unpause"), "{plain}");
        assert!(!html.contains("value=\"7\""));
        assert!(plain.contains("**1. DataForSEO (klaar om te knippen)**"));
        assert!(plain.contains("\n    1. Grant"));
        assert!(plain.contains("\n    6. Unpause"));
        assert!(plain.contains("**2. Asana (Rutger: low risk)**"));
        assert!(plain.contains("\n    --dry-run"));
        assert!(plain.contains("**3. OpenRouter**"));
        let lead = plain.find("Jij kiest").expect("lead");
        let first = plain.find("**1. DataForSEO").expect("first heading");
        let paused = plain.find("paused.").expect("paused");
        let closing = plain.find("Niet alle drie tegelijk.").expect("closing");
        assert!(lead < first);
        assert!(paused < closing);
        assert_eq!(outline_captured_text(&plain), plain);
        let again = html_from_constrained_markdown(&plain);
        assert!(again.contains("Jij kiest welk pad je overzet — één tegelijk"));
        assert!(again.contains("Niet alle drie tegelijk."));
        assert!(again.contains("<li value=\"6\">"));
        assert!(!again.contains("value=\"7\""));
    }

    #[test]
    fn html_and_rtf_become_constrained_markdown_without_fetch() {
        assert_eq!(
            constrained_markdown_from_html("<p><strong>Hello</strong> <b>world</b></p>"),
            "**Hello** **world**"
        );
        assert_eq!(
            constrained_markdown_from_html("<div><code>openrouter</code> <pre>bin</pre></div>"),
            "`openrouter` `bin`"
        );
        assert_eq!(
            constrained_markdown_from_html(
                "<span class=\"c-mrkdwn__code\">chip</span> \
                 <span style=\"font-family: Menlo, monospace\">mono</span>"
            ),
            "`chip` `mono`"
        );
        assert_eq!(
            constrained_markdown_from_html(
                "<span data-qa=\"code\" class=\"c-mrkdwn__code\">fn</span>"
            ),
            "`fn`"
        );
        assert_eq!(
            constrained_markdown_from_html("<?xml version=\"1.0\"?><p>hi</p>"),
            "hi"
        );
        assert_eq!(
            constrained_markdown_from_html("<div aria-label=\"n\"><b>x</b></div>"),
            "**x**"
        );
        assert_eq!(
            constrained_markdown_from_html("<!DOCTYPE html><p>ok</p>"),
            "ok"
        );
        assert_eq!(
            constrained_markdown_from_html("<ul><li>one</li><li>two</li></ul>"),
            "- one\n- two"
        );
        assert_eq!(
            constrained_markdown_from_html("<ol><li>one</li><li>two</li></ol>"),
            "1. one\n2. two"
        );
        let dirty = constrained_markdown_from_html(
            "<p>keep<script>alert(1)</script><img src=\"https://evil.example/x.png\">\
             <a href=\"https://evil.example\">link</a></p>",
        );
        assert_eq!(dirty, "keeplink");
        assert!(!dirty.contains("<script"));
        assert!(!dirty.contains("https:"));
        assert!(!dirty.contains("evil.example"));
        assert!(!dirty.contains("alert"));
        let src = include_str!("markup.rs");
        assert!(!src.contains(&["URL", "Session"].concat()));
        assert!(!src.contains(&["req", "west"].concat()));
        assert!(!src.contains(&["u", "req"].concat()));
        assert_eq!(
            constrained_markdown_from_rtf("{\\rtf1\\ansi\\b bold\\b0}"),
            "**bold**"
        );
        assert_eq!(
            constrained_markdown_from_rtf(
                "{\\rtf1\\ansi{\\fonttbl{\\f0\\fswiss Helvetica;}{\\f1\\fmodern Courier New;}}\\f1 code}"
            ),
            "`code`"
        );
        assert!(markdown_has_style_marks("**bold**"));
        assert!(markdown_has_style_marks("`code`"));
        assert!(!markdown_has_style_marks("2 * 3"));
        assert!(!markdown_has_style_marks("plain"));
        assert_eq!(
            capture_body_preferring_ax_marks("**ax**", Some("`clip`")),
            "**ax**"
        );
        assert_eq!(
            capture_body_preferring_ax_marks("plain", Some("`code`")),
            "`code`"
        );
        assert_eq!(
            capture_body_preferring_ax_marks("plain", Some("still plain")),
            "plain"
        );
        assert_eq!(
            markdown_from_clipboard_types(Some("<code>x</code>"), Some("{\\rtf1\\b y}"), Some("z")),
            Some("`x`".into())
        );
        assert_eq!(
            markdown_from_clipboard_types(None, Some("{\\rtf1\\ansi\\b y\\b0}"), Some("z")),
            Some("**y**".into())
        );
        assert_eq!(
            markdown_from_clipboard_types(None, None, Some("z")),
            Some("z".into())
        );
    }
}
