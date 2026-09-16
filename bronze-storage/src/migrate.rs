//! Versioned checksummed migrations (story 4.2, DAT-001, G-03, ADR-008).
//! ADR-009 stays Proposed: callers supply a locator; no App Group / iCloud default.

use rusqlite::{Connection, OpenFlags, OptionalExtension};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

pub const SCHEMA_V1_SQL: &str = include_str!("v1.sql");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoreMode {
    Writable,
    ReadOnly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MigrateError {
    BackupRequired,
    ApplyFailed,
    IntegrityFailed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OpenError {
    Io,
    Sqlite,
    ReadOnlyRecovery,
}

pub trait StoreLocator {
    fn database_path(&self) -> PathBuf;
}

/// Online backup runs before applying pending SQL. Story 4.5 owns real rotation.
pub trait BackupBeforeMigration {
    fn backup(&mut self, db_path: &Path) -> Result<(), MigrateError>;
}

pub struct PathLocator {
    pub path: PathBuf,
}

impl StoreLocator for PathLocator {
    fn database_path(&self) -> PathBuf {
        self.path.clone()
    }
}

pub struct NoopBackup;

impl BackupBeforeMigration for NoopBackup {
    fn backup(&mut self, _db_path: &Path) -> Result<(), MigrateError> {
        Ok(())
    }
}

pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub sql: &'static str,
}

pub fn v1_migration() -> Migration {
    Migration {
        version: 1,
        name: "schema_v1",
        sql: SCHEMA_V1_SQL,
    }
}

pub fn checksum_sql(sql: &str) -> String {
    let digest = Sha256::digest(sql.as_bytes());
    hex_lower(&digest)
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0xf) as usize] as char);
    }
    out
}

pub fn locator_avoids_icloud(path: &Path) -> bool {
    let raw = path.to_string_lossy();
    !raw.contains("Mobile Documents") && !raw.to_ascii_lowercase().contains("icloud")
}

pub struct Store {
    pub(crate) conn: Connection,
    path: PathBuf,
    mode: StoreMode,
}

impl std::fmt::Debug for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Store")
            .field("path", &self.path)
            .field("mode", &self.mode)
            .finish_non_exhaustive()
    }
}

impl Store {
    pub fn open(
        locator: &dyn StoreLocator,
        backup: &mut dyn BackupBeforeMigration,
    ) -> Result<Self, OpenError> {
        Self::open_with(locator, backup, &[v1_migration()])
    }

    pub fn open_with(
        locator: &dyn StoreLocator,
        backup: &mut dyn BackupBeforeMigration,
        migrations: &[Migration],
    ) -> Result<Self, OpenError> {
        let path = locator.database_path();
        if !locator_avoids_icloud(&path) {
            return Err(OpenError::Io);
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|_| OpenError::Io)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(parent, fs::Permissions::from_mode(0o700));
            }
        }
        let conn = Connection::open_with_flags(
            &path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )
        .map_err(|_| OpenError::Sqlite)?;
        configure_pragmas(&conn).map_err(|_| OpenError::Sqlite)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
        }
        let mut store = Self {
            conn,
            path,
            mode: StoreMode::Writable,
        };
        match store.apply_pending(backup, migrations) {
            Ok(()) => Ok(store),
            Err(_) => {
                store.mode = StoreMode::ReadOnly;
                Err(OpenError::ReadOnlyRecovery)
            }
        }
    }

    pub fn mode(&self) -> StoreMode {
        self.mode
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn journal_mode(&self) -> rusqlite::Result<String> {
        self.conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
    }

    pub fn table_names(&self) -> rusqlite::Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        rows.collect()
    }

    pub fn migration_checksum(&self, version: i64) -> rusqlite::Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT checksum FROM schema_migrations WHERE version = ?1",
                [version],
                |row| row.get(0),
            )
            .optional()
    }

    fn apply_pending(
        &mut self,
        backup: &mut dyn BackupBeforeMigration,
        migrations: &[Migration],
    ) -> Result<(), MigrateError> {
        let current = current_version(&self.conn).map_err(|_| MigrateError::ApplyFailed)?;
        let pending: Vec<&Migration> = migrations.iter().filter(|m| m.version > current).collect();
        if pending.is_empty() {
            return Ok(());
        }
        backup.backup(&self.path)?;
        for migration in pending {
            apply_one(&self.conn, migration)?;
        }
        self.conn
            .execute_batch("PRAGMA foreign_key_check; PRAGMA integrity_check;")
            .map_err(|_| MigrateError::IntegrityFailed)?;
        Ok(())
    }
}

fn configure_pragmas(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA foreign_keys=ON;
         PRAGMA busy_timeout=5000;
         PRAGMA synchronous=NORMAL;",
    )
}

fn current_version(conn: &Connection) -> rusqlite::Result<i64> {
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='schema_migrations'",
        [],
        |row| row.get(0),
    )?;
    if exists == 0 {
        return Ok(0);
    }
    conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )
}

fn apply_one(conn: &Connection, migration: &Migration) -> Result<(), MigrateError> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|_| MigrateError::ApplyFailed)?;
    tx.execute_batch(migration.sql)
        .map_err(|_| MigrateError::ApplyFailed)?;
    let checksum = checksum_sql(migration.sql);
    tx.execute(
        "INSERT INTO schema_migrations (version, name, checksum, applied_at_ms) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![
            migration.version,
            migration.name,
            checksum,
            0_i64,
        ],
    )
    .map_err(|_| MigrateError::ApplyFailed)?;
    tx.commit().map_err(|_| MigrateError::ApplyFailed)?;
    Ok(())
}

#[cfg(test)]
mod migrate_tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static SEQ: AtomicU64 = AtomicU64::new(1);

    fn temp_db() -> PathBuf {
        let n = SEQ.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("bronze-migrate-{nanos}-{n}"));
        fs::create_dir_all(&dir).expect("dir");
        dir.join("bronze.sqlite")
    }

    struct RecordingBackup {
        calls: u32,
        fail: bool,
    }

    impl BackupBeforeMigration for RecordingBackup {
        fn backup(&mut self, _db_path: &Path) -> Result<(), MigrateError> {
            self.calls += 1;
            if self.fail {
                Err(MigrateError::BackupRequired)
            } else {
                Ok(())
            }
        }
    }

    fn required_tables() -> &'static [&'static str] {
        &[
            "command_receipt_entities",
            "command_receipts",
            "diagnostic_events",
            "item_revisions",
            "items",
            "output_profiles",
            "schema_migrations",
            "sections",
            "settings",
            "shortcuts",
            "sources",
            "undo_log",
            "workspaces",
        ]
    }

    #[test]
    fn migrate_v1_creates_docs08_tables_including_receipts_and_diagnostics() {
        let path = temp_db();
        let locator = PathLocator { path: path.clone() };
        let mut backup = RecordingBackup {
            calls: 0,
            fail: false,
        };
        let store = Store::open(&locator, &mut backup).expect("open");
        assert_eq!(store.mode(), StoreMode::Writable);
        assert_eq!(
            store.journal_mode().expect("wal").to_ascii_lowercase(),
            "wal"
        );
        let names = store.table_names().expect("tables");
        for table in required_tables() {
            assert!(names.iter().any(|n| n == table), "missing {table}");
        }
        let checksum = store
            .migration_checksum(1)
            .expect("row")
            .expect("v1 recorded");
        assert_eq!(checksum, checksum_sql(SCHEMA_V1_SQL));
        assert_eq!(backup.calls, 1);
        assert!(locator_avoids_icloud(&path));
    }

    #[test]
    fn migrate_backup_hook_runs_before_schema_and_can_block() {
        let path = temp_db();
        let locator = PathLocator { path };
        let mut backup = RecordingBackup {
            calls: 0,
            fail: true,
        };
        let err = Store::open(&locator, &mut backup).unwrap_err();
        assert!(matches!(err, OpenError::ReadOnlyRecovery));
        assert_eq!(backup.calls, 1);
    }

    #[test]
    fn migrate_failed_sql_keeps_original_and_read_only() {
        let path = temp_db();
        let locator = PathLocator { path: path.clone() };
        let mut backup = NoopBackup;
        let store = Store::open(&locator, &mut backup).expect("v1");
        drop(store);

        let bad = Migration {
            version: 2,
            name: "broken",
            sql: "CREATE TABLE broken (id TEXT PRIMARY KEY); INSERT INTO missing VALUES (1);",
        };
        let migrations = [v1_migration(), bad];
        let mut backup = NoopBackup;
        let err = Store::open_with(&locator, &mut backup, &migrations).unwrap_err();
        assert!(matches!(err, OpenError::ReadOnlyRecovery));

        let conn = Connection::open(&path).expect("reopen original");
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='broken'",
                [],
                |row| row.get(0),
            )
            .expect("count");
        assert_eq!(count, 0);
        let version: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .expect("version");
        assert_eq!(version, 1);
    }

    #[test]
    fn migrate_webview_has_no_sql() {
        let desktop = Path::new(env!("CARGO_MANIFEST_DIR")).join("../apps/desktop");
        let cargo = fs::read_to_string(desktop.join("src-tauri/Cargo.toml")).expect("cargo");
        assert!(!cargo.contains("tauri-plugin-sql"));
        let src_root = desktop.join("src");
        if src_root.exists() {
            for entry in fs::read_dir(&src_root).expect("src") {
                let entry = entry.expect("entry");
                if entry.path().is_file() {
                    let raw = fs::read_to_string(entry.path()).unwrap_or_default();
                    assert!(
                        !raw.contains("tauri-plugin-sql") && !raw.contains("plugin-sql"),
                        "{} must not talk SQL (ADR-008)",
                        entry.path().display()
                    );
                }
            }
        }
    }
}
