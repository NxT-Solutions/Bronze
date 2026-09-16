//! Trash retention and purge policy (story 4.4, QUE-006, DAT-004).

pub const TRASH_RETENTION_DAYS: i64 = 30;

pub fn can_purge_with_descendants(all_descendants_trashed: bool) -> bool {
    all_descendants_trashed
}

pub fn empty_trash_requires_confirmation() -> bool {
    true
}

pub fn claims_forensic_erasure() -> bool {
    false
}

#[cfg(test)]
mod purge_tests {
    use super::*;

    #[test]
    fn purge_never_cascade_deletes_live_descendants() {
        assert!(!can_purge_with_descendants(false));
        assert!(can_purge_with_descendants(true));
        assert_eq!(TRASH_RETENTION_DAYS, 30);
        assert!(empty_trash_requires_confirmation());
        assert!(!claims_forensic_erasure());
    }
}
