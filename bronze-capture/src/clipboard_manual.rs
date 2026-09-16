//! Explicit Create from Clipboard (story 3.7, CAP-003, CAP-005).
//!
//! Text is read only from this command. Synthetic fallback stays off.
//! Stale `changeCount` is `clipboard_changed`. No restore. No files/images.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClipboardKind {
    PlainText,
    File,
    Image,
    Other,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClipboardOutcome {
    Imported { len: usize },
    ClipboardChanged,
    ClipboardUnsupportedType,
    NoSelection,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ClipboardSettings {
    pub synthetic_fallback: bool,
}

pub trait Pasteboard {
    fn generation(&self) -> u64;
    fn inspect_kind(&self) -> ClipboardKind;
    fn read_plain_text(&self) -> Option<String>;
}

pub fn create_from_clipboard<B: Pasteboard>(
    board: &B,
    settings: &ClipboardSettings,
) -> ClipboardOutcome {
    debug_assert!(!settings.synthetic_fallback);
    if settings.synthetic_fallback {
        return ClipboardOutcome::ClipboardUnsupportedType;
    }
    let c0 = board.generation();
    match board.inspect_kind() {
        ClipboardKind::File | ClipboardKind::Image | ClipboardKind::Other => {
            if board.generation() != c0 {
                return ClipboardOutcome::ClipboardChanged;
            }
            return ClipboardOutcome::ClipboardUnsupportedType;
        }
        ClipboardKind::PlainText => {}
    }
    let text = board.read_plain_text();
    if board.generation() != c0 {
        return ClipboardOutcome::ClipboardChanged;
    }
    match text {
        None => ClipboardOutcome::NoSelection,
        Some(body) if body.is_empty() => ClipboardOutcome::NoSelection,
        Some(body) => ClipboardOutcome::Imported { len: body.len() },
    }
}

#[cfg(test)]
mod clipboard_manual_tests {
    use super::*;
    use std::cell::Cell;

    struct FakeBoard {
        generation: Cell<u64>,
        kind: ClipboardKind,
        text: Option<String>,
        reads: Cell<u32>,
        bump_on_read: bool,
    }

    impl FakeBoard {
        fn text(body: &str) -> Self {
            Self {
                generation: Cell::new(1),
                kind: ClipboardKind::PlainText,
                text: Some(body.into()),
                reads: Cell::new(0),
                bump_on_read: false,
            }
        }
    }

    impl Pasteboard for FakeBoard {
        fn generation(&self) -> u64 {
            self.generation.get()
        }

        fn inspect_kind(&self) -> ClipboardKind {
            self.kind
        }

        fn read_plain_text(&self) -> Option<String> {
            self.reads.set(self.reads.get() + 1);
            if self.bump_on_read {
                self.generation.set(self.generation.get() + 1);
            }
            self.text.clone()
        }
    }

    #[test]
    fn clipboard_manual_reads_only_after_explicit_action() {
        let board = FakeBoard::text("hello");
        let settings = ClipboardSettings::default();
        assert_eq!(board.reads.get(), 0);
        let outcome = create_from_clipboard(&board, &settings);
        assert_eq!(outcome, ClipboardOutcome::Imported { len: 5 });
        assert_eq!(board.reads.get(), 1);
        assert!(!settings.synthetic_fallback);
    }

    #[test]
    fn clipboard_manual_synthetic_fallback_stays_off() {
        let mut settings = ClipboardSettings::default();
        assert!(!settings.synthetic_fallback);
        let board = FakeBoard::text("x");
        let _ = create_from_clipboard(&board, &settings);
        assert!(!settings.synthetic_fallback);
        settings.synthetic_fallback = false;
        assert!(!settings.synthetic_fallback);
    }

    #[test]
    fn clipboard_manual_stale_generation_is_clipboard_changed() {
        let mut board = FakeBoard::text("stale");
        board.bump_on_read = true;
        let settings = ClipboardSettings::default();
        assert_eq!(
            create_from_clipboard(&board, &settings),
            ClipboardOutcome::ClipboardChanged
        );
    }

    #[test]
    fn clipboard_manual_rejects_files_and_images_without_restore() {
        let files = FakeBoard {
            generation: Cell::new(3),
            kind: ClipboardKind::File,
            text: None,
            reads: Cell::new(0),
            bump_on_read: false,
        };
        let settings = ClipboardSettings::default();
        assert_eq!(
            create_from_clipboard(&files, &settings),
            ClipboardOutcome::ClipboardUnsupportedType
        );
        assert_eq!(files.reads.get(), 0);

        let images = FakeBoard {
            generation: Cell::new(4),
            kind: ClipboardKind::Image,
            text: None,
            reads: Cell::new(0),
            bump_on_read: false,
        };
        assert_eq!(
            create_from_clipboard(&images, &settings),
            ClipboardOutcome::ClipboardUnsupportedType
        );
        assert_eq!(images.reads.get(), 0);
    }
}
