//! Tombstones, undo inverses, dependency-safe purge (story 4.4, QUE-006, DAT-004).

use crate::migrate::Store;
use bronze_domain::{
    can_purge_with_descendants, claims_forensic_erasure, empty_trash_requires_confirmation,
    TRASH_RETENTION_DAYS,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UndoError {
    EmptyTrashUnconfirmed,
    LiveDescendant,
    Store,
}

impl Store {
    pub fn trash_item(&mut self, id: &str, now_ms: i64) -> Result<(), UndoError> {
        let prev: String = self
            .conn
            .query_row("SELECT status FROM items WHERE id = ?1", [id], |row| {
                row.get(0)
            })
            .map_err(|_| UndoError::Store)?;
        self.conn
            .execute(
                "UPDATE items SET status='trashed', deleted_at_ms=?1, revision=revision+1 WHERE id=?2",
                rusqlite::params![now_ms, id],
            )
            .map_err(|_| UndoError::Store)?;
        let inverse = format!("RESTORE_ITEM:{id}:{prev}");
        self.conn
            .execute(
                "INSERT INTO undo_log (id, command_type, inverse_json, created_at_ms, expires_at_ms)
                 VALUES (?1, 'trash_item', ?2, ?3, ?4)",
                rusqlite::params![
                    format!("undo-{id}-{now_ms}"),
                    inverse,
                    now_ms,
                    now_ms + TRASH_RETENTION_DAYS * 86_400_000
                ],
            )
            .map_err(|_| UndoError::Store)?;
        Ok(())
    }

    pub fn undo_last(&mut self) -> Result<(), UndoError> {
        let (id, inverse): (String, String) = self
            .conn
            .query_row(
                "SELECT id, inverse_json FROM undo_log ORDER BY created_at_ms DESC LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|_| UndoError::Store)?;
        let parts: Vec<&str> = inverse.split(':').collect();
        if parts.first() == Some(&"RESTORE_ITEM") && parts.len() == 3 {
            self.conn
                .execute(
                    "UPDATE items SET status=?1, deleted_at_ms=NULL, revision=revision+1 WHERE id=?2",
                    rusqlite::params![parts[2], parts[1]],
                )
                .map_err(|_| UndoError::Store)?;
        }
        self.conn
            .execute("DELETE FROM undo_log WHERE id=?1", [id])
            .map_err(|_| UndoError::Store)?;
        Ok(())
    }

    pub fn purge_section(&mut self, section_id: &str) -> Result<u64, UndoError> {
        let live: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM items WHERE section_id=?1 AND status != 'trashed'",
                [section_id],
                |row| row.get(0),
            )
            .map_err(|_| UndoError::Store)?;
        if !can_purge_with_descendants(live == 0) {
            return Err(UndoError::LiveDescendant);
        }
        let n = self
            .conn
            .execute(
                "DELETE FROM items WHERE section_id=?1 AND status='trashed'",
                [section_id],
            )
            .map_err(|_| UndoError::Store)?;
        Ok(n as u64)
    }

    pub fn empty_trash(&mut self, confirmed: bool) -> Result<u64, UndoError> {
        if empty_trash_requires_confirmation() && !confirmed {
            return Err(UndoError::EmptyTrashUnconfirmed);
        }
        debug_assert!(!claims_forensic_erasure());
        let n = self
            .conn
            .execute("DELETE FROM items WHERE status='trashed'", [])
            .map_err(|_| UndoError::Store)?;
        Ok(n as u64)
    }
}

#[cfg(test)]
mod undo_tests {
    use super::*;
    use crate::migrate::{NoopBackup, PathLocator};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static SEQ: AtomicU64 = AtomicU64::new(1);

    fn open_store() -> Store {
        let n = SEQ.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("bronze-undo-{nanos}-{n}"));
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
                 INSERT INTO items VALUES ('live','s1','note','keep','und','queued','a',NULL,1,1,1,NULL,NULL,NULL);
                 INSERT INTO items VALUES ('gone','s1','note','drop','und','queued','b',NULL,1,1,1,NULL,NULL,NULL);",
            )
            .expect("seed");
        store
    }

    #[test]
    fn undo_trash_restore_and_purge_skips_live_descendants() {
        let mut store = open_store();
        store.trash_item("gone", 10).expect("trash");
        let status: String = store
            .conn
            .query_row("SELECT status FROM items WHERE id='gone'", [], |row| {
                row.get(0)
            })
            .expect("status");
        assert_eq!(status, "trashed");
        store.undo_last().expect("undo");
        let status: String = store
            .conn
            .query_row("SELECT status FROM items WHERE id='gone'", [], |row| {
                row.get(0)
            })
            .expect("restored");
        assert_eq!(status, "queued");
        store.trash_item("gone", 20).expect("trash again");
        assert_eq!(
            store.purge_section("s1").unwrap_err(),
            UndoError::LiveDescendant
        );
        let live: i64 = store
            .conn
            .query_row("SELECT COUNT(*) FROM items WHERE id='live'", [], |row| {
                row.get(0)
            })
            .expect("live");
        assert_eq!(live, 1);
        store.trash_item("live", 30).expect("trash live");
        assert_eq!(store.purge_section("s1").expect("purge"), 2);
        assert!(!claims_forensic_erasure());
        assert_eq!(TRASH_RETENTION_DAYS, 30);
    }

    #[test]
    fn undo_empty_trash_requires_confirmation() {
        let mut store = open_store();
        store.trash_item("gone", 1).expect("trash");
        assert_eq!(
            store.empty_trash(false).unwrap_err(),
            UndoError::EmptyTrashUnconfirmed
        );
        assert_eq!(store.empty_trash(true).expect("empty"), 1);
    }
}
