//! Section/item entities and closed lifecycle (story 4.1, QUE-001/002/003, I18N-003, DAT-001).

use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Lifecycle {
    Queued,
    Copied,
    Active,
    Done,
    Skipped,
    Trashed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidTransition {
    pub from: Lifecycle,
    pub to: Lifecycle,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LanguageError {
    InvalidTag,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentLanguage(String);

impl ContentLanguage {
    pub fn und() -> Self {
        Self("und".into())
    }

    pub fn parse(tag: Option<&str>) -> Result<Self, LanguageError> {
        match tag {
            None | Some("") => Ok(Self::und()),
            Some(raw) if well_formed_bcp47(raw) => Ok(Self(raw.into())),
            Some(_) => Err(LanguageError::InvalidTag),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for ContentLanguage {
    fn default() -> Self {
        Self::und()
    }
}

fn well_formed_bcp47(tag: &str) -> bool {
    let mut parts = tag.split('-');
    let Some(primary) = parts.next() else {
        return false;
    };
    if !subtag(primary, 1, 8, false) {
        return false;
    }
    parts.all(|part| subtag(part, 1, 8, true))
}

fn subtag(part: &str, min: usize, max: usize, alphanum: bool) -> bool {
    let len = part.len();
    if len < min || len > max {
        return false;
    }
    part.bytes()
        .all(|b| b.is_ascii_alphabetic() || (alphanum && b.is_ascii_digit()))
}

pub fn can_transition(from: Lifecycle, to: Lifecycle) -> bool {
    if from == to {
        return false;
    }
    matches!(
        (from, to),
        (
            Lifecycle::Queued,
            Lifecycle::Copied
                | Lifecycle::Active
                | Lifecycle::Done
                | Lifecycle::Skipped
                | Lifecycle::Trashed
        ) | (
            Lifecycle::Copied,
            Lifecycle::Queued
                | Lifecycle::Active
                | Lifecycle::Done
                | Lifecycle::Skipped
                | Lifecycle::Trashed
        ) | (
            Lifecycle::Active,
            Lifecycle::Queued
                | Lifecycle::Copied
                | Lifecycle::Done
                | Lifecycle::Skipped
                | Lifecycle::Trashed
        ) | (Lifecycle::Done, Lifecycle::Queued | Lifecycle::Trashed)
            | (Lifecycle::Skipped, Lifecycle::Queued | Lifecycle::Trashed)
            | (Lifecycle::Trashed, Lifecycle::Queued)
    )
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Section {
    pub id: u64,
    pub title: String,
    pub color_token: u32,
    pub rank: u64,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl Section {
    pub fn new(id: u64, title: String, now_ms: u64) -> Self {
        Self {
            id,
            title,
            color_token: 0,
            rank: 0,
            created_at_ms: now_ms,
            updated_at_ms: now_ms,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Item {
    pub id: u64,
    pub section_id: u64,
    pub body: String,
    pub content_language: ContentLanguage,
    pub lifecycle: Lifecycle,
    pub rank: u64,
    pub revision: u64,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl Item {
    pub fn new(
        id: u64,
        section_id: u64,
        body: String,
        language: Option<&str>,
        now_ms: u64,
    ) -> Result<Self, LanguageError> {
        Ok(Self {
            id,
            section_id,
            body,
            content_language: ContentLanguage::parse(language)?,
            lifecycle: Lifecycle::Queued,
            rank: 0,
            revision: 1,
            created_at_ms: now_ms,
            updated_at_ms: now_ms,
        })
    }

    pub fn transition(&mut self, to: Lifecycle, now_ms: u64) -> Result<(), InvalidTransition> {
        if !can_transition(self.lifecycle, to) {
            return Err(InvalidTransition {
                from: self.lifecycle,
                to,
            });
        }
        self.lifecycle = to;
        self.revision += 1;
        self.updated_at_ms = now_ms;
        Ok(())
    }

    pub fn set_body(&mut self, body: String, now_ms: u64) {
        self.body = body;
        self.revision += 1;
        self.updated_at_ms = now_ms;
    }
}

impl fmt::Display for Lifecycle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Queued => "queued",
            Self::Copied => "copied",
            Self::Active => "active",
            Self::Done => "done",
            Self::Skipped => "skipped",
            Self::Trashed => "trashed",
        })
    }
}

#[cfg(test)]
mod domain_tests {
    use super::*;

    #[test]
    fn lifecycle_closed_set_and_invalid_transitions_fail() {
        let states = [
            Lifecycle::Queued,
            Lifecycle::Copied,
            Lifecycle::Active,
            Lifecycle::Done,
            Lifecycle::Skipped,
            Lifecycle::Trashed,
        ];
        assert_eq!(states.len(), 6);
        let mut item = Item::new(1, 1, "x".into(), None, 10).expect("item");
        assert_eq!(item.lifecycle, Lifecycle::Queued);
        item.transition(Lifecycle::Copied, 11)
            .expect("queued->copied");
        assert!(item.transition(Lifecycle::Copied, 12).is_err());
        assert!(Item::new(2, 1, "y".into(), None, 10)
            .unwrap()
            .transition(Lifecycle::Trashed, 11)
            .is_ok());
        let mut done = Item::new(3, 1, "z".into(), None, 10).unwrap();
        done.transition(Lifecycle::Done, 11).unwrap();
        assert_eq!(
            done.transition(Lifecycle::Copied, 12).unwrap_err(),
            InvalidTransition {
                from: Lifecycle::Done,
                to: Lifecycle::Copied,
            }
        );
        let mut trash = Item::new(4, 1, "t".into(), None, 10).unwrap();
        trash.transition(Lifecycle::Trashed, 11).unwrap();
        assert!(trash.transition(Lifecycle::Active, 12).is_err());
        trash.transition(Lifecycle::Queued, 13).expect("restore");
    }

    #[test]
    fn content_language_defaults_to_und_and_accepts_bcp47() {
        let item = Item::new(1, 1, "body".into(), None, 1).unwrap();
        assert_eq!(item.content_language.as_str(), "und");
        let en = Item::new(2, 1, "body".into(), Some("en-US"), 1).unwrap();
        assert_eq!(en.content_language.as_str(), "en-US");
        assert_eq!(ContentLanguage::parse(Some("")), Ok(ContentLanguage::und()));
        assert_eq!(
            ContentLanguage::parse(Some(" en")),
            Err(LanguageError::InvalidTag)
        );
        assert_eq!(
            ContentLanguage::parse(Some("not a tag")),
            Err(LanguageError::InvalidTag)
        );
    }

    #[test]
    fn body_is_not_trimmed_or_normalized() {
        let raw = "  hello\r\n\t ";
        let mut item = Item::new(1, 1, raw.into(), None, 1).unwrap();
        assert_eq!(item.body, raw);
        item.set_body("  ".into(), 2);
        assert_eq!(item.body, "  ");
        assert_ne!(item.body, item.body.trim());
    }

    #[test]
    fn timestamps_are_caller_supplied_utc_millis() {
        let section = Section::new(1, "  inbox  ".into(), 1_700_000_000_000);
        assert_eq!(section.title, "  inbox  ");
        assert_eq!(section.created_at_ms, 1_700_000_000_000);
        let item = Item::new(9, 1, "x".into(), Some("und"), 1_700_000_000_001).unwrap();
        assert_eq!(item.created_at_ms, 1_700_000_000_001);
        assert_eq!(item.updated_at_ms, item.created_at_ms);
    }
}
