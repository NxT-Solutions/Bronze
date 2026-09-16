//! Copy to pasteboard (story 6.4, QUE-005). No synthetic paste.

use bronze_domain::{format_items, lifecycle_after_copy, Lifecycle, OutputProfile};

pub const SYNTHETIC_PASTE: bool = false;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CopyError {
    Pasteboard,
    SyntheticPasteForbidden,
}

pub trait Pasteboard {
    fn write_text(&mut self, text: &str) -> Result<(), CopyError>;
}

#[derive(Default)]
pub struct FakePasteboard {
    pub last: Option<String>,
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
    board.write_text(&text)?;
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
    }
}
