//! Composer submit policy (story 6.1, QUE-002, CAP-003, I18N-003).

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComposerChord {
    Enter,
    CmdEnter,
}

pub const COMPOSER_LABEL_KEY: &str = "composer.add.label";
pub const COMPOSER_SUBMIT_KEY: &str = "composer.add.submit";
pub const COMPOSER_ERROR_KEY: &str = "composer.add.error";

pub fn composer_should_add(chord: ComposerChord, is_composing: bool) -> bool {
    matches!(chord, ComposerChord::CmdEnter) && !is_composing
}

#[cfg(test)]
mod composer_tests {
    use super::*;

    #[test]
    fn composer_cmd_enter_adds_and_ime_enter_does_not() {
        assert!(composer_should_add(ComposerChord::CmdEnter, false));
        assert!(!composer_should_add(ComposerChord::Enter, false));
        assert!(!composer_should_add(ComposerChord::Enter, true));
        assert!(!composer_should_add(ComposerChord::CmdEnter, true));
        assert_eq!(COMPOSER_LABEL_KEY, "composer.add.label");
        assert_eq!(COMPOSER_SUBMIT_KEY, "composer.add.submit");
        assert_eq!(COMPOSER_ERROR_KEY, "composer.add.error");
    }
}
