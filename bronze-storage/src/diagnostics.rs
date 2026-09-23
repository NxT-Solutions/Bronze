use crate::migrate::Store;
use rusqlite::OptionalExtension;

pub const DIAGNOSTIC_WINDOW_MS: i64 = 3_600_000;
pub const APP_SCHEMA_VERSION: i64 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagStoreError {
    Store,
    Invalid,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiagnosticEventRow {
    pub id: String,
    pub occurred_at_ms: i64,
    pub request_id: String,
    pub stage: String,
    pub result_code: String,
    pub duration_ms: Option<i64>,
    pub trigger_kind: String,
    pub provider_kind: Option<String>,
    pub permission_state: Option<String>,
    pub source_bundle_id: Option<String>,
    pub app_schema_version: i64,
    pub queue_depth: Option<i64>,
    pub overflow_count: Option<i64>,
    pub tap_health: Option<String>,
    pub store_result_code: Option<String>,
    pub build_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecentSourceRow {
    pub bundle_id: Option<String>,
    pub app_name: Option<String>,
    pub captured_at_ms: i64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct QueueStatusCounts {
    pub queued: u32,
    pub copied: u32,
    pub active: u32,
    pub done: u32,
    pub skipped: u32,
    pub trashed: u32,
}

impl Store {
    pub fn insert_diagnostic_event(&self, row: &DiagnosticEventRow) -> Result<(), DiagStoreError> {
        let stage = closed_stage(&row.stage).ok_or(DiagStoreError::Invalid)?;
        let result = closed_result(&row.result_code).ok_or(DiagStoreError::Invalid)?;
        let trigger = closed_trigger(&row.trigger_kind).ok_or(DiagStoreError::Invalid)?;
        let bundle = sanitize_diag_label(row.source_bundle_id.as_deref(), 128);
        self.conn
            .execute(
                "INSERT INTO diagnostic_events (
                    id, occurred_at_ms, request_id, stage, result_code, duration_ms,
                    trigger_kind, provider_kind, permission_state, source_bundle_id,
                    app_schema_version, queue_depth, overflow_count, tap_health,
                    store_result_code, build_id
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
                rusqlite::params![
                    row.id,
                    row.occurred_at_ms,
                    row.request_id,
                    stage,
                    result,
                    row.duration_ms,
                    trigger,
                    sanitize_diag_label(row.provider_kind.as_deref(), 32),
                    sanitize_diag_label(row.permission_state.as_deref(), 32),
                    bundle,
                    row.app_schema_version,
                    row.queue_depth,
                    row.overflow_count,
                    sanitize_diag_label(row.tap_health.as_deref(), 32),
                    sanitize_diag_label(row.store_result_code.as_deref(), 32),
                    row.build_id,
                ],
            )
            .map_err(|_| DiagStoreError::Store)?;
        Ok(())
    }

    pub fn list_diagnostics_since(
        &self,
        since_ms: i64,
    ) -> Result<Vec<DiagnosticEventRow>, DiagStoreError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, occurred_at_ms, request_id, stage, result_code, duration_ms,
                        trigger_kind, provider_kind, permission_state, source_bundle_id,
                        app_schema_version, queue_depth, overflow_count, tap_health,
                        store_result_code, build_id
                 FROM diagnostic_events
                 WHERE occurred_at_ms >= ?1
                 ORDER BY occurred_at_ms DESC, id DESC",
            )
            .map_err(|_| DiagStoreError::Store)?;
        let rows = stmt
            .query_map([since_ms], |row| {
                Ok(DiagnosticEventRow {
                    id: row.get(0)?,
                    occurred_at_ms: row.get(1)?,
                    request_id: row.get(2)?,
                    stage: row.get(3)?,
                    result_code: row.get(4)?,
                    duration_ms: row.get(5)?,
                    trigger_kind: row.get(6)?,
                    provider_kind: row.get(7)?,
                    permission_state: row.get(8)?,
                    source_bundle_id: row.get(9)?,
                    app_schema_version: row.get(10)?,
                    queue_depth: row.get(11)?,
                    overflow_count: row.get(12)?,
                    tap_health: row.get(13)?,
                    store_result_code: row.get(14)?,
                    build_id: row.get(15)?,
                })
            })
            .map_err(|_| DiagStoreError::Store)?;
        rows.collect::<Result<_, _>>()
            .map_err(|_| DiagStoreError::Store)
    }

    pub fn list_sources_since(
        &self,
        since_ms: i64,
    ) -> Result<Vec<RecentSourceRow>, DiagStoreError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT bundle_id, app_name, MAX(captured_at_ms)
                 FROM sources
                 WHERE captured_at_ms >= ?1
                   AND (bundle_id IS NOT NULL OR app_name IS NOT NULL)
                 GROUP BY bundle_id, app_name
                 ORDER BY MAX(captured_at_ms) DESC",
            )
            .map_err(|_| DiagStoreError::Store)?;
        let rows = stmt
            .query_map([since_ms], |row| {
                Ok(RecentSourceRow {
                    bundle_id: row.get(0)?,
                    app_name: row.get(1)?,
                    captured_at_ms: row.get(2)?,
                })
            })
            .map_err(|_| DiagStoreError::Store)?;
        rows.collect::<Result<_, _>>()
            .map_err(|_| DiagStoreError::Store)
    }

    pub fn queue_status_counts(&self) -> Result<QueueStatusCounts, DiagStoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT status, COUNT(*) FROM items GROUP BY status")
            .map_err(|_| DiagStoreError::Store)?;
        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(|_| DiagStoreError::Store)?;
        let mut counts = QueueStatusCounts::default();
        for row in rows {
            let (status, n) = row.map_err(|_| DiagStoreError::Store)?;
            let slot = match status.as_str() {
                "queued" => &mut counts.queued,
                "copied" => &mut counts.copied,
                "active" => &mut counts.active,
                "done" => &mut counts.done,
                "skipped" => &mut counts.skipped,
                "trashed" => &mut counts.trashed,
                _ => continue,
            };
            *slot = u32::try_from(n).unwrap_or(u32::MAX);
        }
        Ok(counts)
    }

    pub fn schema_version(&self) -> Result<i64, DiagStoreError> {
        self.conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| DiagStoreError::Store)
            .map(|value| value.unwrap_or(0))
    }
}

fn closed_stage(value: &str) -> Option<&'static str> {
    match value {
        "trigger" => Some("trigger"),
        "target" => Some("target"),
        "ax" => Some("ax"),
        "selection" => Some("selection"),
        "clipboard" => Some("clipboard"),
        "store" => Some("store"),
        "request" => Some("request"),
        _ => None,
    }
}

fn closed_result(value: &str) -> Option<&'static str> {
    match value {
        "ok" => Some("ok"),
        "trigger_queue_overflow" => Some("trigger_queue_overflow"),
        "context_unavailable" => Some("context_unavailable"),
        "protected_content" => Some("protected_content"),
        "protection_unknown" => Some("protection_unknown"),
        "app_excluded" => Some("app_excluded"),
        "no_selection" => Some("no_selection"),
        "clipboard_changed" => Some("clipboard_changed"),
        "clipboard_unsupported_type" => Some("clipboard_unsupported_type"),
        "persist_failed" => Some("persist_failed"),
        "cancelled" => Some("cancelled"),
        "diagnostic_write_failed" => Some("diagnostic_write_failed"),
        _ => None,
    }
}

fn closed_trigger(value: &str) -> Option<&'static str> {
    match value {
        "event_tap" => Some("event_tap"),
        "chord" => Some("chord"),
        "menu" => Some("menu"),
        "manual_clipboard" => Some("manual_clipboard"),
        _ => None,
    }
}

fn looks_forbidden(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("secret")
        || lower.contains("password")
        || lower.contains("token")
        || lower.contains("hunter2")
        || lower.contains("http")
        || value.contains('/')
}

fn sanitize_diag_label(value: Option<&str>, max_chars: usize) -> Option<String> {
    let value = value?;
    if looks_forbidden(value) {
        return None;
    }
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
mod diagnostics_store_tests {
    use super::*;
    use crate::composer::ComposerDraft;
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
        let dir = std::env::temp_dir().join(format!("bronze-diag-{nanos}-{n}"));
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

    fn sample_row(id: &str, at: i64, bundle: Option<&str>) -> DiagnosticEventRow {
        DiagnosticEventRow {
            id: id.into(),
            occurred_at_ms: at,
            request_id: format!("r-{id}"),
            stage: "store".into(),
            result_code: "ok".into(),
            duration_ms: Some(4),
            trigger_kind: "menu".into(),
            provider_kind: Some("ax".into()),
            permission_state: None,
            source_bundle_id: bundle.map(str::to_string),
            app_schema_version: APP_SCHEMA_VERSION,
            queue_depth: None,
            overflow_count: None,
            tap_health: None,
            store_result_code: Some("ok".into()),
            build_id: "0.1.0".into(),
        }
    }

    #[test]
    fn insert_lists_last_hour_without_body_text() {
        let mut store = open_store();
        store
            .add_from_capture(
                &ComposerDraft {
                    body: "hunter2-s3cret-payload".into(),
                    content_language: None,
                },
                Some("TextEdit"),
                Some("com.apple.TextEdit"),
                "s1",
                "i-cap",
                9_000,
            )
            .expect("capture");
        store
            .insert_diagnostic_event(&sample_row("d-old", 100, Some("com.apple.TextEdit")))
            .expect("old");
        store
            .insert_diagnostic_event(&sample_row("d-new", 8_000, Some("com.apple.TextEdit")))
            .expect("new");
        let listed = store.list_diagnostics_since(1_000).expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, "d-new");
        assert_eq!(
            listed[0].source_bundle_id.as_deref(),
            Some("com.apple.TextEdit")
        );
        let debug = format!("{listed:?}");
        assert!(!debug.contains("hunter2"));
        assert!(!debug.contains("s3cret"));
        let sources = store.list_sources_since(1).expect("sources");
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].bundle_id.as_deref(), Some("com.apple.TextEdit"));
        assert_eq!(sources[0].app_name.as_deref(), Some("TextEdit"));
        let src = include_str!("diagnostics.rs");
        let prod = src.split("#[cfg(test)]").next().expect("prod");
        assert!(!prod.contains("safe_title"));
        assert!(!prod.contains("url"));
        assert!(!prod.contains("items.body"));
        assert_eq!(store.schema_version().expect("schema"), 2);
        let counts = store.queue_status_counts().expect("counts");
        assert_eq!(counts.queued, 1);
    }

    #[test]
    fn diagnostic_insert_failure_keeps_saved_item() {
        let mut store = open_store();
        store
            .add_from_composer(
                &ComposerDraft {
                    body: "keep me".into(),
                    content_language: None,
                },
                ComposerChord::CmdEnter,
                false,
                "s1",
                "i-keep",
                3,
            )
            .expect("item");
        store
            .insert_diagnostic_event(&sample_row("d-dup", 20, None))
            .expect("first");
        assert_eq!(
            store.insert_diagnostic_event(&sample_row("d-dup", 21, None)),
            Err(DiagStoreError::Store)
        );
        assert_eq!(
            store.insert_diagnostic_event(&DiagnosticEventRow {
                stage: "not-a-stage".into(),
                ..sample_row("d-bad", 22, None)
            }),
            Err(DiagStoreError::Invalid)
        );
        let body: String = store
            .conn
            .query_row("SELECT body FROM items WHERE id='i-keep'", [], |row| {
                row.get(0)
            })
            .expect("kept");
        assert_eq!(body, "keep me");
        store
            .insert_diagnostic_event(&sample_row("d-omit", 23, Some("/Users/alex/doc")))
            .expect("stored");
        let stored: Option<String> = store
            .conn
            .query_row(
                "SELECT source_bundle_id FROM diagnostic_events WHERE id='d-omit'",
                [],
                |row| row.get(0),
            )
            .expect("row");
        assert_eq!(stored, None);
    }
}
