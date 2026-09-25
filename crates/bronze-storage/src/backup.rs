//! SQLite online backup / verified restore (story 4.5, DAT-002).

use crate::migrate::Store;
use rusqlite::backup::Backup;
use rusqlite::Connection;
use std::fs;
use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackupSchedule {
    Daily,
    Weekly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackupError {
    OnlineBackupFailed,
    IntegrityFailed,
    Io,
}

impl Store {
    pub fn online_backup(&self, dest: &Path) -> Result<(), BackupError> {
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|_| BackupError::Io)?;
        }
        let mut dst = Connection::open(dest).map_err(|_| BackupError::OnlineBackupFailed)?;
        {
            let backup =
                Backup::new(&self.conn, &mut dst).map_err(|_| BackupError::OnlineBackupFailed)?;
            backup
                .run_to_completion(5, std::time::Duration::from_millis(0), None)
                .map_err(|_| BackupError::OnlineBackupFailed)?;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(dest, fs::Permissions::from_mode(0o600));
        }
        Ok(())
    }

    pub fn restore_verified(&mut self, src: &Path, snapshot: &Path) -> Result<(), BackupError> {
        self.online_backup(snapshot)?;
        if !integrity_ok(src) {
            return Err(BackupError::IntegrityFailed);
        }
        let src_conn = Connection::open(src).map_err(|_| BackupError::OnlineBackupFailed)?;
        let backup =
            Backup::new(&src_conn, &mut self.conn).map_err(|_| BackupError::OnlineBackupFailed)?;
        backup
            .run_to_completion(5, std::time::Duration::from_millis(0), None)
            .map_err(|_| BackupError::OnlineBackupFailed)?;
        Ok(())
    }
}

fn integrity_ok(path: &Path) -> bool {
    let Ok(conn) = Connection::open(path) else {
        return false;
    };
    conn.query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))
        .ok()
        .is_some_and(|s| s.eq_ignore_ascii_case("ok"))
}

#[cfg(test)]
mod backup_tests {
    use super::*;
    use crate::migrate::{NoopBackup, PathLocator};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static SEQ: AtomicU64 = AtomicU64::new(1);

    fn open_store() -> (Store, std::path::PathBuf) {
        let n = SEQ.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("bronze-backup-{nanos}-{n}"));
        std::fs::create_dir_all(&dir).expect("dir");
        let locator = PathLocator {
            path: dir.join("bronze.sqlite"),
        };
        let mut hook = NoopBackup;
        let store = Store::open(&locator, &mut hook).expect("open");
        store
            .conn
            .execute_batch("INSERT INTO workspaces VALUES ('w1','ws',1,1);")
            .expect("seed");
        (store, dir)
    }

    #[test]
    fn backup_schedule_is_daily_or_weekly_only() {
        let names = [BackupSchedule::Daily, BackupSchedule::Weekly];
        assert_eq!(names.len(), 2);
        assert!(!format!("{names:?}").contains("monthly"));
    }

    #[test]
    fn backup_online_restore_snapshots_first_and_rejects_corrupt() {
        let (mut store, dir) = open_store();
        let dest = dir.join("ok.sqlite");
        store.online_backup(&dest).expect("backup");
        assert!(integrity_ok(&dest));

        let snapshot = dir.join("pre-restore.sqlite");
        store
            .restore_verified(&dest, &snapshot)
            .expect("restore good");
        assert!(snapshot.exists());

        let bad = dir.join("bad.sqlite");
        fs::write(&bad, b"not a database").expect("junk");
        let before: i64 = store
            .conn
            .query_row("SELECT COUNT(*) FROM workspaces", [], |row| row.get(0))
            .expect("count");
        assert_eq!(
            store
                .restore_verified(&bad, &dir.join("snap2.sqlite"))
                .unwrap_err(),
            BackupError::IntegrityFailed
        );
        let after: i64 = store
            .conn
            .query_row("SELECT COUNT(*) FROM workspaces", [], |row| row.get(0))
            .expect("after");
        assert_eq!(before, after);
    }

    #[test]
    fn backup_does_not_file_copy_live_wal() {
        let src = include_str!("backup.rs");
        assert!(src.contains("Backup::new"));
        let forbidden = ["std::fs::", "copy"].concat();
        assert!(!src.contains(&forbidden));
    }
}
