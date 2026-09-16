//! Rust-owned SQLite store (DAT-001, ADR-008). ADR-009 locator stays an interface.

mod backup;
mod composer;
mod export;
mod migrate;
mod queue;
mod receipts;
mod search;
mod shortcuts;
mod undo;

pub use backup::{BackupError, BackupSchedule};
pub use composer::{ComposerDraft, ComposerError};
pub use export::{
    check_entry_path, filter_settings_for_export, ExportError, ExportPreview, ImportError,
    ImportPreview, ImportStrategy, Overwrite, EXPORT_FORMAT, EXPORT_VERSION,
    SECRET_BODY_WARNING_KEY,
};
pub use migrate::{
    checksum_sql, locator_avoids_icloud, v1_migration, BackupBeforeMigration, MigrateError,
    Migration, NoopBackup, OpenError, PathLocator, Store, StoreLocator, StoreMode, SCHEMA_V1_SQL,
};
pub use queue::{
    queue_action_key, QueueAction, QueueError, QueueItemRow, DRAG_REQUIRED, QUEUE_COMPLETE_KEY,
    QUEUE_EDIT_KEY, QUEUE_MOVE_DOWN_KEY, QUEUE_MOVE_UP_KEY, QUEUE_SKIP_KEY, QUEUE_TRASH_KEY,
};
pub use receipts::{Command, CommandError, Receipt, RETRY_WINDOW_MS};
pub use search::{
    announce_search, claims_locale_search, SearchAnnouncement, SearchError, SearchHit, SearchKind,
    ADR_018_STATUS, QUE_007_COMPLETE,
};
pub use shortcuts::ShortcutStoreError;
pub use undo::UndoError;
