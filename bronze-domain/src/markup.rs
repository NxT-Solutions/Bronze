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
}
