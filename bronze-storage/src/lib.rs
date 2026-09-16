//! Rust-owned SQLite store (DAT-001, ADR-008). ADR-009 locator stays an interface.

mod migrate;

pub use migrate::{
    checksum_sql, locator_avoids_icloud, v1_migration, BackupBeforeMigration, MigrateError,
    Migration, NoopBackup, OpenError, PathLocator, Store, StoreLocator, StoreMode, SCHEMA_V1_SQL,
};
