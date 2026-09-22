//! Persist composer drafts (story 6.1). Failure must not consume the draft.

use crate::migrate::{Store, StoreMode};
use bronze_domain::{compact_title, composer_should_add, ComposerChord, ContentLanguage};

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
        let title = compact_title(&draft.body);
        self.conn
            .execute(
                "INSERT INTO items (id, section_id, kind, body, title, content_language, status, rank, source_id, revision, created_at_ms, updated_at_ms, completed_at_ms, deleted_at_ms)
                 VALUES (?1, ?2, 'prompt', ?3, ?4, ?5, 'queued', ?1, NULL, 1, ?6, ?6, NULL, NULL)",
                rusqlite::params![
                    item_id,
                    section_id,
                    draft.body,
                    title,
                    language.as_str(),
                    now_ms
                ],
            )
            .map_err(|_| ComposerError::Store)?;
        Ok(item_id.into())
    }

    pub fn add_from_capture(
        &mut self,
        draft: &ComposerDraft,
        source_app_name: Option<&str>,
        bundle_id: Option<&str>,
        section_id: &str,
        item_id: &str,
        now_ms: i64,
    ) -> Result<String, ComposerError> {
        if self.mode() != StoreMode::Writable {
            return Err(ComposerError::ReadOnly);
        }
        let language = ContentLanguage::parse(draft.content_language.as_deref())
            .map_err(|_| ComposerError::Language)?;
        let name = sanitize_app_name(source_app_name);
        let bundle = sanitize_bundle_id(bundle_id);
        let source_id = if name.is_none() && bundle.is_none() {
            None
        } else {
            let source_id = format!("src-{item_id}");
            self.conn
                .execute(
                    "INSERT INTO sources (id, bundle_id, app_name, safe_title, url, captured_at_ms, policy_version)
                     VALUES (?1, ?2, ?3, NULL, NULL, ?4, 1)",
                    rusqlite::params![source_id, bundle, name, now_ms],
                )
                .ok()
                .map(|_| source_id)
        };
        let title = compact_title(&draft.body);
        self.conn
            .execute(
                "INSERT INTO items (id, section_id, kind, body, title, content_language, status, rank, source_id, revision, created_at_ms, updated_at_ms, completed_at_ms, deleted_at_ms)
                 VALUES (?1, ?2, 'snippet', ?3, ?4, ?5, 'queued', ?1, ?6, 1, ?7, ?7, NULL, NULL)",
                rusqlite::params![
                    item_id,
                    section_id,
                    draft.body,
                    title,
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
    sanitize_source_label(name, 64)
}

fn sanitize_bundle_id(bundle_id: Option<&str>) -> Option<String> {
    sanitize_source_label(bundle_id, 128)
}

fn sanitize_source_label(value: Option<&str>, max_chars: usize) -> Option<String> {
    let value = value?;
    let cleaned: String = value
        .chars()
        .filter(|c| !c.is_control())
        .take(max_chars)
        .collect();
    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
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
            .add_from_capture(&draft, Some("TextEdit"), None, "s1", "i-cap", 12)
            .expect("capture");
        let (kind, app, url, bundle): (String, String, Option<String>, Option<String>) = store
            .conn
            .query_row(
                "SELECT items.kind, sources.app_name, sources.url, sources.bundle_id
                 FROM items JOIN sources ON sources.id = items.source_id
                 WHERE items.id='i-cap'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .expect("row");
        assert_eq!(kind, "snippet");
        assert_eq!(app, "TextEdit");
        assert_eq!(url, None);
        assert_eq!(bundle, None);
        let title: String = store
            .conn
            .query_row("SELECT title FROM items WHERE id='i-cap'", [], |row| {
                row.get(0)
            })
            .expect("title");
        assert_eq!(title, bronze_domain::compact_title("selected"));
        store
            .add_from_capture(&draft, Some("Bronze"), None, "s1", "i-self", 13)
            .expect("self");
        let (source, app): (Option<String>, String) = store
            .conn
            .query_row(
                "SELECT source_id, (SELECT app_name FROM sources WHERE id = items.source_id)
                 FROM items WHERE id='i-self'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("self source");
        assert_eq!(source.as_deref(), Some("src-i-self"));
        assert_eq!(app, "Bronze");
    }

    #[test]
    fn capture_persists_bundle_id_without_url_or_icon_blob() {
        let src = include_str!("composer.rs");
        let insert = src
            .split("INSERT INTO sources")
            .nth(1)
            .expect("insert")
            .split(';')
            .next()
            .expect("stmt");
        let lower = insert.to_ascii_lowercase();
        assert!(!lower.contains("png"));
        assert!(!lower.contains("blob"));
        assert!(!lower.contains("icon"));
        assert!(insert.contains("bundle_id"));

        let mut store = open_store();
        let draft = ComposerDraft {
            body: "selected".into(),
            content_language: None,
        };
        store
            .add_from_capture(
                &draft,
                Some("TextEdit"),
                Some("com.apple.TextEdit"),
                "s1",
                "i-bundle",
                14,
            )
            .expect("capture");
        let (app, bundle, url): (String, String, Option<String>) = store
            .conn
            .query_row(
                "SELECT app_name, bundle_id, url FROM sources WHERE id='src-i-bundle'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("source");
        assert_eq!(app, "TextEdit");
        assert_eq!(bundle, "com.apple.TextEdit");
        assert_eq!(url, None);
        let row = store.get_item("i-bundle").expect("row");
        assert_eq!(row.source_bundle_id.as_deref(), Some("com.apple.TextEdit"));
        assert_eq!(row.source_app_name.as_deref(), Some("TextEdit"));
        store
            .add_from_capture(
                &draft,
                Some("Notes"),
                Some("Bronze"),
                "s1",
                "i-bronze-bundle",
                15,
            )
            .expect("bronze bundle");
        let kept: Option<String> = store
            .conn
            .query_row(
                "SELECT bundle_id FROM sources WHERE id='src-i-bronze-bundle'",
                [],
                |row| row.get(0),
            )
            .expect("kept");
        assert_eq!(kept.as_deref(), Some("Bronze"));
        let cols: Vec<String> = store
            .conn
            .prepare("PRAGMA table_info(sources)")
            .expect("pragma")
            .query_map([], |row| row.get::<_, String>(1))
            .expect("cols")
            .collect::<Result<_, _>>()
            .expect("names");
        assert!(!cols
            .iter()
            .any(|name| name.to_ascii_lowercase().contains("png")));
        assert!(!cols
            .iter()
            .any(|name| name.to_ascii_lowercase().contains("icon")));
    }
}
