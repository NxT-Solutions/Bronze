//! Queue actions without drag (story 6.2, QUE-002/003/006).

use crate::migrate::Store;
use bronze_domain::{can_transition, compact_title, Lifecycle};
use rusqlite::OptionalExtension;

pub const QUEUE_MOVE_UP_KEY: &str = "queue.item.moveUp";
pub const QUEUE_MOVE_DOWN_KEY: &str = "queue.item.moveDown";
pub const QUEUE_COMPLETE_KEY: &str = "queue.item.complete";
pub const QUEUE_SKIP_KEY: &str = "queue.item.skip";
pub const QUEUE_TRASH_KEY: &str = "queue.item.trash";
pub const QUEUE_EDIT_KEY: &str = "queue.item.edit";
pub const DRAG_REQUIRED: bool = false;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueueAction {
    Complete,
    Skip,
    Trash,
    MoveUp,
    MoveDown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueueError {
    NotFound,
    InvalidTransition,
    Store,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueueItemRow {
    pub id: String,
    pub section_id: String,
    pub body: String,
    pub title: Option<String>,
    pub content_language: String,
    pub status: String,
    pub rank: String,
    pub source_app_name: Option<String>,
    pub source_bundle_id: Option<String>,
}

pub fn queue_action_key(action: QueueAction) -> &'static str {
    match action {
        QueueAction::Complete => QUEUE_COMPLETE_KEY,
        QueueAction::Skip => QUEUE_SKIP_KEY,
        QueueAction::Trash => QUEUE_TRASH_KEY,
        QueueAction::MoveUp => QUEUE_MOVE_UP_KEY,
        QueueAction::MoveDown => QUEUE_MOVE_DOWN_KEY,
    }
}

impl Store {
    pub fn ensure_inbox(&self, now_ms: i64) -> Result<String, QueueError> {
        let existing: Option<String> = self
            .conn
            .query_row(
                "SELECT id FROM sections WHERE id='inbox' AND state='active'",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| QueueError::Store)?;
        if let Some(id) = existing {
            return Ok(id);
        }
        self.conn
            .execute(
                "INSERT OR IGNORE INTO workspaces (id, name, created_at_ms, updated_at_ms)
                 VALUES ('default', 'Bronze', ?1, ?1)",
                [now_ms],
            )
            .map_err(|_| QueueError::Store)?;
        self.conn
            .execute(
                "INSERT OR IGNORE INTO sections
                 (id, workspace_id, title, rank, state, color_token, revision, created_at_ms, updated_at_ms, deleted_at_ms)
                 VALUES ('inbox', 'default', 'Inbox', '0', 'active', NULL, 1, ?1, ?1, NULL)",
                [now_ms],
            )
            .map_err(|_| QueueError::Store)?;
        Ok("inbox".into())
    }

    pub fn list_items(&self, include_trashed: bool) -> Result<Vec<QueueItemRow>, QueueError> {
        let sql = if include_trashed {
            "SELECT items.id, items.section_id, items.body, items.title, items.content_language, items.status, items.rank, sources.app_name, sources.bundle_id FROM items LEFT JOIN sources ON sources.id = items.source_id ORDER BY items.rank, items.id"
        } else {
            "SELECT items.id, items.section_id, items.body, items.title, items.content_language, items.status, items.rank, sources.app_name, sources.bundle_id FROM items LEFT JOIN sources ON sources.id = items.source_id WHERE items.status != 'trashed' ORDER BY items.rank, items.id"
        };
        let mut stmt = self.conn.prepare(sql).map_err(|_| QueueError::Store)?;
        let rows = stmt
            .query_map([], queue_item_from_row)
            .map_err(|_| QueueError::Store)?;
        rows.collect::<Result<_, _>>()
            .map_err(|_| QueueError::Store)
    }

    pub fn get_item(&self, id: &str) -> Result<QueueItemRow, QueueError> {
        self.conn
            .query_row(
                "SELECT items.id, items.section_id, items.body, items.title, items.content_language, items.status, items.rank, sources.app_name, sources.bundle_id FROM items LEFT JOIN sources ON sources.id = items.source_id WHERE items.id=?1",
                [id],
                queue_item_from_row,
            )
            .map_err(|_| QueueError::NotFound)
    }

    pub fn mark_copied(&mut self, id: &str, now_ms: i64) -> Result<(), QueueError> {
        self.set_status(id, "copied", now_ms)
    }

    pub fn apply_queue_action(
        &mut self,
        id: &str,
        action: QueueAction,
        now_ms: i64,
    ) -> Result<(), QueueError> {
        match action {
            QueueAction::Complete => self.set_status(id, "done", now_ms),
            QueueAction::Skip => self.set_status(id, "skipped", now_ms),
            QueueAction::Trash => self.set_status(id, "trashed", now_ms),
            QueueAction::MoveUp => self.move_rank(id, -1),
            QueueAction::MoveDown => self.move_rank(id, 1),
        }
    }

    pub fn edit_item_body(&mut self, id: &str, body: &str, now_ms: i64) -> Result<(), QueueError> {
        let title = compact_title(body);
        let n = self
            .conn
            .execute(
                "UPDATE items SET body=?1, title=?2, revision=revision+1, updated_at_ms=?3 WHERE id=?4",
                rusqlite::params![body, title, now_ms, id],
            )
            .map_err(|_| QueueError::Store)?;
        if n == 0 {
            Err(QueueError::NotFound)
        } else {
            Ok(())
        }
    }

    fn set_status(&mut self, id: &str, to: &str, now_ms: i64) -> Result<(), QueueError> {
        let from: String = self
            .conn
            .query_row("SELECT status FROM items WHERE id=?1", [id], |row| {
                row.get(0)
            })
            .map_err(|_| QueueError::NotFound)?;
        let from_life = parse_lifecycle(&from).ok_or(QueueError::InvalidTransition)?;
        let to_life = parse_lifecycle(to).ok_or(QueueError::InvalidTransition)?;
        if !can_transition(from_life, to_life) {
            return Err(QueueError::InvalidTransition);
        }
        let deleted = if to == "trashed" { Some(now_ms) } else { None };
        self.conn
            .execute(
                "UPDATE items SET status=?1, deleted_at_ms=?2, revision=revision+1, updated_at_ms=?3 WHERE id=?4",
                rusqlite::params![to, deleted, now_ms, id],
            )
            .map_err(|_| QueueError::Store)?;
        Ok(())
    }

    fn move_rank(&mut self, id: &str, delta: i64) -> Result<(), QueueError> {
        let (section, rank): (String, String) = self
            .conn
            .query_row(
                "SELECT section_id, rank FROM items WHERE id=?1",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|_| QueueError::NotFound)?;
        let ranks: Vec<(String, String)> = self
            .conn
            .prepare(
                "SELECT id, rank FROM items WHERE section_id=?1 AND status != 'trashed' ORDER BY rank",
            )
            .map_err(|_| QueueError::Store)?
            .query_map([&section], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|_| QueueError::Store)?
            .collect::<Result<_, _>>()
            .map_err(|_| QueueError::Store)?;
        let idx = ranks
            .iter()
            .position(|(i, _)| i == id)
            .ok_or(QueueError::NotFound)?;
        let dest = if delta < 0 {
            idx.checked_sub(1)
        } else {
            Some(idx + 1).filter(|n| *n < ranks.len())
        };
        let Some(other) = dest else {
            return Ok(());
        };
        let other_rank = ranks[other].1.clone();
        self.conn
            .execute(
                "UPDATE items SET rank=?1 WHERE id=?2",
                rusqlite::params![format!("tmp-{id}"), id],
            )
            .map_err(|_| QueueError::Store)?;
        self.conn
            .execute(
                "UPDATE items SET rank=?1 WHERE id=?2",
                rusqlite::params![rank, ranks[other].0],
            )
            .map_err(|_| QueueError::Store)?;
        self.conn
            .execute(
                "UPDATE items SET rank=?1 WHERE id=?2",
                rusqlite::params![other_rank, id],
            )
            .map_err(|_| QueueError::Store)?;
        Ok(())
    }
}

fn queue_item_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<QueueItemRow> {
    Ok(QueueItemRow {
        id: row.get(0)?,
        section_id: row.get(1)?,
        body: row.get(2)?,
        title: row.get(3)?,
        content_language: row.get(4)?,
        status: row.get(5)?,
        rank: row.get(6)?,
        source_app_name: row.get(7)?,
        source_bundle_id: row.get(8)?,
    })
}

fn parse_lifecycle(status: &str) -> Option<Lifecycle> {
    Some(match status {
        "queued" => Lifecycle::Queued,
        "copied" => Lifecycle::Copied,
        "active" => Lifecycle::Active,
        "done" => Lifecycle::Done,
        "skipped" => Lifecycle::Skipped,
        "trashed" => Lifecycle::Trashed,
        _ => return None,
    })
}

#[cfg(test)]
mod queue_tests {
    use super::*;
    use crate::migrate::{NoopBackup, PathLocator, Store};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static SEQ: AtomicU64 = AtomicU64::new(1);

    fn open_store() -> Store {
        let n = SEQ.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("bronze-queue-{nanos}-{n}"));
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
                 INSERT INTO sections VALUES ('s1','w1','inbox','a','active',NULL,1,1,1,NULL);
                 INSERT INTO items VALUES ('a','s1','note','one','und','queued','1',NULL,1,1,1,NULL,NULL,NULL);
                 INSERT INTO items VALUES ('b','s1','note','two','und','queued','2',NULL,1,1,1,NULL,NULL,NULL);",
            )
            .expect("seed");
        store
    }

    #[test]
    fn queue_actions_move_complete_skip_trash_and_edit_without_drag() {
        assert!(!DRAG_REQUIRED);
        assert_eq!(queue_action_key(QueueAction::MoveUp), QUEUE_MOVE_UP_KEY);
        let mut store = open_store();
        store
            .apply_queue_action("b", QueueAction::MoveUp, 10)
            .expect("up");
        let first: String = store
            .conn
            .query_row(
                "SELECT id FROM items WHERE status!='trashed' ORDER BY rank LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("first");
        assert_eq!(first, "b");
        store
            .apply_queue_action("a", QueueAction::Complete, 11)
            .expect("done");
        store
            .apply_queue_action("b", QueueAction::Skip, 12)
            .expect("skip");
        store.edit_item_body("a", "  edited  ", 13).expect("edit");
        let body: String = store
            .conn
            .query_row("SELECT body FROM items WHERE id='a'", [], |row| row.get(0))
            .expect("body");
        assert_eq!(body, "  edited  ");
        let title: Option<String> = store
            .conn
            .query_row("SELECT title FROM items WHERE id='a'", [], |row| row.get(0))
            .expect("title");
        assert_eq!(
            title.as_deref(),
            Some(bronze_domain::compact_title("  edited  ").as_str())
        );
        let src = include_str!("queue.rs");
        let prod = src.split("#[cfg(test)]").next().expect("prod");
        assert!(prod.contains("compact_title"));
        assert!(!prod.contains("set_item_title"));
        assert!(!prod.contains("native_item_title"));
        store
            .apply_queue_action("a", QueueAction::Trash, 14)
            .expect("trash");
        let status: String = store
            .conn
            .query_row("SELECT status FROM items WHERE id='a'", [], |row| {
                row.get(0)
            })
            .expect("status");
        assert_eq!(status, "trashed");
        assert_eq!(
            store
                .apply_queue_action("a", QueueAction::Complete, 15)
                .unwrap_err(),
            QueueError::InvalidTransition
        );
    }

    #[test]
    fn ensure_inbox_lists_and_marks_copied() {
        let mut store = open_store();
        assert_eq!(store.ensure_inbox(20).expect("inbox"), "inbox");
        assert_eq!(store.list_items(false).expect("list").len(), 2);
        store.mark_copied("b", 21).expect("copied");
        assert_eq!(store.get_item("b").expect("row").status, "copied");
        store
            .apply_queue_action("b", QueueAction::Trash, 22)
            .expect("trash");
        assert_eq!(store.list_items(false).expect("active").len(), 1);
        assert_eq!(store.list_items(true).expect("all").len(), 2);
    }
}
