//! Persist composer drafts (story 6.1). Failure must not consume the draft.

use crate::migrate::{Store, StoreMode};
use bronze_domain::{composer_should_add, ComposerChord, ContentLanguage};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComposerDraft {
    pub body: String,
    pub content_language: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComposerError {
    ImeComposing,
    NotASubmit,
    Language,
    ReadOnly,
    Store,
}

impl Store {
    pub fn add_from_composer(
        &mut self,
        draft: &ComposerDraft,
        chord: ComposerChord,
        is_composing: bool,
        section_id: &str,
        item_id: &str,
        now_ms: i64,
    ) -> Result<String, ComposerError> {
        if !composer_should_add(chord, is_composing) {
            return Err(if is_composing {
                ComposerError::ImeComposing
            } else {
                ComposerError::NotASubmit
            });
        }
        if self.mode() != StoreMode::Writable {
            return Err(ComposerError::ReadOnly);
        }
        let language = ContentLanguage::parse(draft.content_language.as_deref())
            .map_err(|_| ComposerError::Language)?;
        self.conn
            .execute(
                "INSERT INTO items (id, section_id, kind, body, content_language, status, rank, source_id, revision, created_at_ms, updated_at_ms, completed_at_ms, deleted_at_ms)
                 VALUES (?1, ?2, 'prompt', ?3, ?4, 'queued', ?1, NULL, 1, ?5, ?5, NULL, NULL)",
                rusqlite::params![item_id, section_id, draft.body, language.as_str(), now_ms],
            )
            .map_err(|_| ComposerError::Store)?;
        Ok(item_id.into())
    }

    pub fn add_from_capture(
        &mut self,
        draft: &ComposerDraft,
        source_app_name: Option<&str>,
        section_id: &str,
        item_id: &str,
        now_ms: i64,
    ) -> Result<String, ComposerError> {
        if self.mode() != StoreMode::Writable {
            return Err(ComposerError::ReadOnly);
        }
        let language = ContentLanguage::parse(draft.content_language.as_deref())
            .map_err(|_| ComposerError::Language)?;
        let source_id = sanitize_app_name(source_app_name).and_then(|name| {
            let source_id = format!("src-{item_id}");
            self.conn
                .execute(
                    "INSERT INTO sources (id, bundle_id, app_name, safe_title, url, captured_at_ms, policy_version)
                     VALUES (?1, NULL, ?2, NULL, NULL, ?3, 1)",
                    rusqlite::params![source_id, name, now_ms],
                )
                .ok()?;
            Some(source_id)
        });
        self.conn
            .execute(
                "INSERT INTO items (id, section_id, kind, body, content_language, status, rank, source_id, revision, created_at_ms, updated_at_ms, completed_at_ms, deleted_at_ms)
                 VALUES (?1, ?2, 'snippet', ?3, ?4, 'queued', ?1, ?5, 1, ?6, ?6, NULL, NULL)",
                rusqlite::params![
                    item_id,
                    section_id,
                    draft.body,
                    language.as_str(),
                    source_id,
                    now_ms
                ],
            )
            .map_err(|_| ComposerError::Store)?;
        Ok(item_id.into())
    }
}

fn sanitize_app_name(name: Option<&str>) -> Option<String> {
    let name = name?;
    let cleaned: String = name.chars().filter(|c| !c.is_control()).take(64).collect();
    let trimmed = cleaned.trim();
    if trimmed.is_empty()
        || trimmed.eq_ignore_ascii_case("bronze-desktop")
        || trimmed.eq_ignore_ascii_case("Bronze")
    {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod composer_persist_tests {
    use super::*;
    use crate::migrate::{NoopBackup, PathLocator, Store};
    use bronze_domain::ComposerChord;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static SEQ: AtomicU64 = AtomicU64::new(1);

    fn open_store() -> Store {
        let n = SEQ.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("bronze-composer-{nanos}-{n}"));
        std::fs::create_dir_all(&dir).expect("dir");
        let locator = PathLocator {
            path: dir.join("bronze.sqlite"),
        };
        let mut backup = NoopBackup;
        let store = Store::open(&locator, &mut backup).expect("open");
        store
            .conn
            .execute_batch(
                "INSERT INTO workspaces VALUES ('w1','ws',1,1);
                 INSERT INTO sections VALUES ('s1','w1','inbox','a','active',NULL,1,1,1,NULL);",
            )
            .expect("seed");
        store
    }

    #[test]
    fn composer_cmd_enter_persists_und_and_failure_retains_draft() {
        let mut store = open_store();
        let draft = ComposerDraft {
            body: "  park me  ".into(),
            content_language: None,
        };
        let id = store
            .add_from_composer(&draft, ComposerChord::CmdEnter, false, "s1", "i-new", 10)
            .expect("add");
        assert_eq!(id, "i-new");
        let (body, lang, kind): (String, String, String) = store
            .conn
            .query_row(
                "SELECT body, content_language, kind FROM items WHERE id='i-new'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("row");
        assert_eq!(body, "  park me  ");
        assert_eq!(lang, "und");
        assert_eq!(kind, "prompt");
        assert_eq!(draft.body, "  park me  ");

        let mut failed = open_store();
        let kept = ComposerDraft {
            body: "keep draft".into(),
            content_language: None,
        };
        let err = failed
            .add_from_composer(&kept, ComposerChord::CmdEnter, false, "missing", "i-x", 11)
            .unwrap_err();
        assert_eq!(err, ComposerError::Store);
        assert_eq!(kept.body, "keep draft");
        let count: i64 = failed
            .conn
            .query_row("SELECT COUNT(*) FROM items", [], |row| row.get(0))
            .expect("count");
        assert_eq!(count, 0);
    }

    #[test]
    fn composer_enter_during_ime_does_not_persist() {
        let mut store = open_store();
        let draft = ComposerDraft {
            body: "ime".into(),
            content_language: Some("ja".into()),
        };
        assert_eq!(
            store
                .add_from_composer(&draft, ComposerChord::Enter, true, "s1", "i-ime", 1)
                .unwrap_err(),
            ComposerError::ImeComposing
        );
        assert_eq!(
            store
                .add_from_composer(&draft, ComposerChord::Enter, false, "s1", "i-ime", 1)
                .unwrap_err(),
            ComposerError::NotASubmit
        );
        let count: i64 = store
            .conn
            .query_row("SELECT COUNT(*) FROM items", [], |row| row.get(0))
            .expect("count");
        assert_eq!(count, 0);
        assert_eq!(draft.body, "ime");
    }

    #[test]
    fn capture_persists_app_name_without_url() {
        let mut store = open_store();
        let draft = ComposerDraft {
            body: "selected".into(),
            content_language: None,
        };
        store
            .add_from_capture(&draft, Some("TextEdit"), "s1", "i-cap", 12)
            .expect("capture");
        let (kind, app, url): (String, String, Option<String>) = store
            .conn
            .query_row(
                "SELECT items.kind, sources.app_name, sources.url
                 FROM items JOIN sources ON sources.id = items.source_id
                 WHERE items.id='i-cap'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("row");
        assert_eq!(kind, "snippet");
        assert_eq!(app, "TextEdit");
        assert_eq!(url, None);
        store
            .add_from_capture(&draft, Some("Bronze"), "s1", "i-self", 13)
            .expect("self");
        let source: Option<String> = store
            .conn
            .query_row("SELECT source_id FROM items WHERE id='i-self'", [], |row| {
                row.get(0)
            })
            .expect("self source");
        assert_eq!(source, None);
    }
}
