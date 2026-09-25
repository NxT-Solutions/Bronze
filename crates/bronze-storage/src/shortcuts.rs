//! Persist ShortcutActionId rows (story 7.2, SET-002, ADR-016).

use crate::migrate::Store;
use bronze_settings::ShortcutRegistry;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShortcutStoreError {
    Store,
}

impl Store {
    pub fn persist_shortcuts(
        &mut self,
        registry: &ShortcutRegistry,
    ) -> Result<(), ShortcutStoreError> {
        let tx = self
            .conn
            .transaction()
            .map_err(|_| ShortcutStoreError::Store)?;
        tx.execute("DELETE FROM shortcuts", [])
            .map_err(|_| ShortcutStoreError::Store)?;
        for (action, json, enabled, revision) in registry.persist_rows() {
            tx.execute(
                "INSERT INTO shortcuts (action, trigger_json, enabled, revision, updated_at_ms)
                 VALUES (?1, ?2, ?3, ?4, 1)",
                rusqlite::params![action, json, i64::from(enabled), revision],
            )
            .map_err(|_| ShortcutStoreError::Store)?;
        }
        tx.commit().map_err(|_| ShortcutStoreError::Store)?;
        Ok(())
    }

    pub fn load_shortcuts(&self) -> Result<ShortcutRegistry, ShortcutStoreError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT action, trigger_json, enabled, revision FROM shortcuts ORDER BY action",
            )
            .map_err(|_| ShortcutStoreError::Store)?;
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)? != 0,
                    row.get::<_, u32>(3)?,
                ))
            })
            .map_err(|_| ShortcutStoreError::Store)?;
        let collected: Result<Vec<_>, _> = rows.collect();
        let collected = collected.map_err(|_| ShortcutStoreError::Store)?;
        ShortcutRegistry::from_rows(&collected).map_err(|_| ShortcutStoreError::Store)
    }
}

#[cfg(test)]
mod shortcuts_tests {
    use super::*;
    use crate::migrate::{NoopBackup, PathLocator};
    use bronze_settings::ShortcutActionId;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static SEQ: AtomicU64 = AtomicU64::new(1);

    fn open_store() -> (Store, std::path::PathBuf) {
        let n = SEQ.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("bronze-shortcuts-{nanos}-{n}"));
        std::fs::create_dir_all(&dir).expect("dir");
        let locator = PathLocator {
            path: dir.join("bronze.sqlite"),
        };
        let mut backup = NoopBackup;
        (Store::open(&locator, &mut backup).expect("open"), dir)
    }

    #[test]
    fn shortcuts_table_holds_every_registry_action() {
        let (mut store, _dir) = open_store();
        let registry = ShortcutRegistry::seeded();
        store.persist_shortcuts(&registry).expect("persist");
        let loaded = store.load_shortcuts().expect("load");
        assert_eq!(loaded.persist_rows().len(), ShortcutActionId::ALL.len());
        assert_eq!(
            loaded.standard_chord().action,
            ShortcutActionId::CaptureSelection
        );
    }
}
