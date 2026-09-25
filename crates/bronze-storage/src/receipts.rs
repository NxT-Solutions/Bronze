//! Idempotent command receipts and optimistic revisions (story 4.3, DAT-001, QUE-002).

use crate::migrate::{Store, StoreMode};
use rusqlite::OptionalExtension;
use std::fmt;

pub const RETRY_WINDOW_MS: i64 = 7 * 24 * 60 * 60 * 1000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommandError {
    IdempotencyExpired,
    RevisionConflict,
    ReadOnly,
    Store,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Receipt {
    pub id: String,
    pub command_type: String,
    pub result_code: String,
    pub created_at_ms: i64,
    pub expires_at_ms: i64,
    pub entity_id: String,
    pub result_revision: i64,
}

impl fmt::Display for Receipt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "receipt {} type={} code={}",
            self.id, self.command_type, self.result_code
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Command {
    pub id: String,
    pub command_type: String,
    pub entity_id: String,
    pub expected_revision: i64,
    pub issued_at_ms: i64,
}

impl Store {
    pub fn execute_command(
        &mut self,
        command: &Command,
        now_ms: i64,
    ) -> Result<Receipt, CommandError> {
        if self.mode() != StoreMode::Writable {
            return Err(CommandError::ReadOnly);
        }
        if now_ms.saturating_sub(command.issued_at_ms) > RETRY_WINDOW_MS {
            return Err(CommandError::IdempotencyExpired);
        }
        if let Some(existing) =
            load_receipt(&self.conn, &command.id).map_err(|_| CommandError::Store)?
        {
            if now_ms > existing.expires_at_ms {
                return Err(CommandError::IdempotencyExpired);
            }
            return Ok(existing);
        }
        let current =
            item_revision(&self.conn, &command.entity_id).map_err(|_| CommandError::Store)?;
        if current != command.expected_revision {
            return Err(CommandError::RevisionConflict);
        }
        let next = current + 1;
        let expires = now_ms + RETRY_WINDOW_MS;
        self.conn
            .execute(
                "UPDATE items SET revision = ?1, updated_at_ms = ?2 WHERE id = ?3",
                rusqlite::params![next, now_ms, command.entity_id],
            )
            .map_err(|_| CommandError::Store)?;
        self.conn
            .execute(
                "INSERT INTO command_receipts (id, command_type, result_code, created_at_ms, expires_at_ms)
                 VALUES (?1, ?2, 'ok', ?3, ?4)",
                rusqlite::params![command.id, command.command_type, now_ms, expires],
            )
            .map_err(|_| CommandError::Store)?;
        self.conn
            .execute(
                "INSERT INTO command_receipt_entities (command_id, ordinal, entity_id, result_revision)
                 VALUES (?1, 0, ?2, ?3)",
                rusqlite::params![command.id, command.entity_id, next],
            )
            .map_err(|_| CommandError::Store)?;
        load_receipt(&self.conn, &command.id)
            .map_err(|_| CommandError::Store)?
            .ok_or(CommandError::Store)
    }
}

fn load_receipt(conn: &rusqlite::Connection, id: &str) -> rusqlite::Result<Option<Receipt>> {
    conn.query_row(
        "SELECT r.id, r.command_type, r.result_code, r.created_at_ms, r.expires_at_ms,
                e.entity_id, e.result_revision
         FROM command_receipts r
         JOIN command_receipt_entities e ON e.command_id = r.id
         WHERE r.id = ?1",
        [id],
        |row| {
            Ok(Receipt {
                id: row.get(0)?,
                command_type: row.get(1)?,
                result_code: row.get(2)?,
                created_at_ms: row.get(3)?,
                expires_at_ms: row.get(4)?,
                entity_id: row.get(5)?,
                result_revision: row.get(6)?,
            })
        },
    )
    .optional()
}

fn item_revision(conn: &rusqlite::Connection, id: &str) -> rusqlite::Result<i64> {
    conn.query_row("SELECT revision FROM items WHERE id = ?1", [id], |row| {
        row.get(0)
    })
}

#[cfg(test)]
mod receipts_tests {
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
        let dir = std::env::temp_dir().join(format!("bronze-receipts-{nanos}-{n}"));
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
                 INSERT INTO items VALUES ('i1','s1','note','  body  ','und','queued','a',NULL,1,1,1,NULL,NULL,NULL);",
            )
            .expect("seed");
        store
    }

    fn cmd(id: &str, expected: i64, issued: i64) -> Command {
        Command {
            id: id.into(),
            command_type: "touch".into(),
            entity_id: "i1".into(),
            expected_revision: expected,
            issued_at_ms: issued,
        }
    }

    #[test]
    fn receipts_duplicate_id_reconstructs_prior_result() {
        let mut store = open_store();
        let first = store
            .execute_command(&cmd("c1", 1, 100), 100)
            .expect("first");
        assert_eq!(first.result_code, "ok");
        assert_eq!(first.result_revision, 2);
        let again = store.execute_command(&cmd("c1", 1, 100), 200).expect("dup");
        assert_eq!(again, first);
        let revision: i64 = store
            .conn
            .query_row("SELECT revision FROM items WHERE id='i1'", [], |row| {
                row.get(0)
            })
            .expect("rev");
        assert_eq!(revision, 2);
        let rendered = format!("{first:?}{first}");
        assert!(!rendered.contains("body"));
        let cols: Vec<String> = store
            .conn
            .prepare("PRAGMA table_info(command_receipts)")
            .expect("info")
            .query_map([], |row| row.get::<_, String>(1))
            .expect("map")
            .map(|c| c.expect("col"))
            .collect();
        assert!(!cols.iter().any(|c| c.contains("body")));
    }

    #[test]
    fn receipts_expired_id_does_not_execute() {
        let mut store = open_store();
        let err = store
            .execute_command(&cmd("old", 1, 0), RETRY_WINDOW_MS + 1)
            .unwrap_err();
        assert_eq!(err, CommandError::IdempotencyExpired);
        let revision: i64 = store
            .conn
            .query_row("SELECT revision FROM items WHERE id='i1'", [], |row| {
                row.get(0)
            })
            .expect("rev");
        assert_eq!(revision, 1);
        let count: i64 = store
            .conn
            .query_row("SELECT COUNT(*) FROM command_receipts", [], |row| {
                row.get(0)
            })
            .expect("count");
        assert_eq!(count, 0);
    }

    #[test]
    fn receipts_stale_revision_is_conflict() {
        let mut store = open_store();
        store.execute_command(&cmd("c1", 1, 10), 10).expect("ok");
        let err = store.execute_command(&cmd("c2", 1, 11), 11).unwrap_err();
        assert_eq!(err, CommandError::RevisionConflict);
    }
}
