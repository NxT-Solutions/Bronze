//! Copy to pasteboard (story 6.4, QUE-005). No synthetic paste.

use bronze_domain::{
    format_items, html_from_constrained_markdown, lifecycle_after_copy, Lifecycle, OutputProfile,
};

pub const SYNTHETIC_PASTE: bool = false;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CopyError {
    Pasteboard,
    SyntheticPasteForbidden,
}

pub trait Pasteboard {
    fn write_text(&mut self, text: &str) -> Result<(), CopyError>;
    fn write_plain_and_html(&mut self, plain: &str, html: &str) -> Result<(), CopyError>;
}

#[derive(Default)]
pub struct FakePasteboard {
    pub last: Option<String>,
    pub last_html: Option<String>,
    pub fail: bool,
}

impl Pasteboard for FakePasteboard {
    fn write_text(&mut self, text: &str) -> Result<(), CopyError> {
        if self.fail {
            return Err(CopyError::Pasteboard);
        }
        self.last = Some(text.to_string());
        Ok(())
    }

    fn write_plain_and_html(&mut self, plain: &str, html: &str) -> Result<(), CopyError> {
        if self.fail {
            return Err(CopyError::Pasteboard);
        }
        self.last = Some(plain.to_string());
        self.last_html = Some(html.to_string());
        Ok(())
    }
}

fn format_items_html(items: &[&str]) -> String {
    items
        .iter()
        .map(|item| html_from_constrained_markdown(item))
        .collect()
}

pub fn copy_items(
    items: &[&str],
    profile: &OutputProfile,
    board: &mut impl Pasteboard,
    lifecycle: &mut Lifecycle,
) -> Result<String, CopyError> {
    if SYNTHETIC_PASTE {
        return Err(CopyError::SyntheticPasteForbidden);
    }
    let text = format_items(profile, items);
    let html = format_items_html(items);
    board.write_plain_and_html(&text, &html)?;
    if let Some(next) = lifecycle_after_copy(profile.post_copy_action) {
        *lifecycle = next;
    }
    Ok(text)
}

#[cfg(test)]
mod copy_tests {
    use super::*;
    use bronze_domain::{default_output_profile, PostCopyAction};

    #[test]
    fn copy_lifecycle_only_after_pasteboard_success() {
        assert!(!SYNTHETIC_PASTE);
        let profile = default_output_profile();
        assert_eq!(profile.post_copy_action, PostCopyAction::Copied);
        let mut life = Lifecycle::Queued;
        let mut fail = FakePasteboard {
            fail: true,
            last: None,
            last_html: None,
        };
        assert_eq!(
            copy_items(&["x"], &profile, &mut fail, &mut life).unwrap_err(),
            CopyError::Pasteboard
        );
        assert_eq!(life, Lifecycle::Queued);
        let mut ok = FakePasteboard::default();
        copy_items(&["x"], &profile, &mut ok, &mut life).expect("copy");
        assert_eq!(life, Lifecycle::Copied);
        assert_eq!(ok.last.as_deref(), Some("x"));
        let html = ok.last_html.as_deref().expect("html");
        assert!(!html.contains("<strong>"));
        assert!(html.contains("white-space:pre-wrap"));
        assert!(html.contains('x'));
    }

    #[test]
    fn copy_writes_sanitized_html_not_raw_body() {
        let profile = default_output_profile();
        let mut life = Lifecycle::Queued;
        let mut board = FakePasteboard::default();
        copy_items(
            &["**Hello**", "<script>alert(1)</script>"],
            &profile,
            &mut board,
            &mut life,
        )
        .expect("copy");
        assert_eq!(
            board.last.as_deref(),
            Some("**Hello**\n<script>alert(1)</script>")
        );
        let html = board.last_html.as_deref().expect("html");
        assert!(html.contains("<strong>Hello</strong>"));
        assert!(!html.contains("<script"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains("white-space:pre-wrap"));
        let tap = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../native/macos/BronzeNative/Sources/BronzeNative/EventTapEngine.swift"
        ));
        assert!(!tap.contains("bronze_native_pasteboard_write"));
    }
}
