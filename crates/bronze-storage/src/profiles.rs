use crate::migrate::Store;
use bronze_settings::SettingsProfileExport;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProfileStoreError {
    Store,
    Invalid,
}

impl Store {
    pub fn list_output_profiles(&self) -> Result<Vec<SettingsProfileExport>, ProfileStoreError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, builtin_key, name, format, format_options_json, source_policy,
                        post_copy_action, advance_policy, revision
                 FROM output_profiles ORDER BY id",
            )
            .map_err(|_| ProfileStoreError::Store)?;
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, u32>(8)?,
                ))
            })
            .map_err(|_| ProfileStoreError::Store)?;
        let mut profiles = Vec::new();
        for row in rows {
            let (
                id,
                builtin_key,
                name,
                format,
                options_json,
                source_policy,
                post_copy,
                advance,
                revision,
            ) = row.map_err(|_| ProfileStoreError::Store)?;
            let format_options =
                serde_json::from_str(&options_json).unwrap_or_else(|_| serde_json::json!({}));
            profiles.push(SettingsProfileExport {
                id,
                builtin_key,
                name,
                format,
                format_options,
                source_policy,
                post_copy_action: post_copy,
                advance_policy: advance,
                revision,
            });
        }
        Ok(profiles)
    }

    pub fn replace_output_profiles(
        &mut self,
        profiles: &[SettingsProfileExport],
    ) -> Result<(), ProfileStoreError> {
        for profile in profiles {
            if !profile_id_ok(&profile.id) {
                return Err(ProfileStoreError::Invalid);
            }
        }
        let tx = self
            .conn
            .transaction()
            .map_err(|_| ProfileStoreError::Store)?;
        tx.execute("DELETE FROM output_profiles", [])
            .map_err(|_| ProfileStoreError::Store)?;
        for profile in profiles {
            let options = serde_json::to_string(&profile.format_options)
                .map_err(|_| ProfileStoreError::Invalid)?;
            tx.execute(
                "INSERT INTO output_profiles (
                    id, builtin_key, name, format, format_options_json, source_policy,
                    post_copy_action, advance_policy, revision
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![
                    profile.id,
                    profile.builtin_key,
                    profile.name,
                    profile.format,
                    options,
                    profile.source_policy,
                    profile.post_copy_action,
                    profile.advance_policy,
                    profile.revision
                ],
            )
            .map_err(|_| ProfileStoreError::Store)?;
        }
        tx.commit().map_err(|_| ProfileStoreError::Store)?;
        Ok(())
    }
}

fn profile_id_ok(id: &str) -> bool {
    let trimmed = id.trim();
    !trimmed.is_empty()
        && !trimmed.contains('/')
        && !trimmed.contains('\\')
        && !trimmed.contains("..")
}

#[cfg(test)]
mod profiles_tests {
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
        let dir = std::env::temp_dir().join(format!("bronze-profiles-{nanos}-{n}"));
        std::fs::create_dir_all(&dir).expect("dir");
        let locator = PathLocator {
            path: dir.join("bronze.sqlite"),
        };
        let mut backup = NoopBackup;
        Store::open(&locator, &mut backup).expect("open")
    }

    #[test]
    fn output_profiles_round_trip_and_reject_path_ids() {
        let mut store = open_store();
        assert!(store.list_output_profiles().expect("empty").is_empty());
        let row = SettingsProfileExport {
            id: "custom-prompt".into(),
            builtin_key: None,
            name: "Vault prompt".into(),
            format: "promptBlock".into(),
            format_options: serde_json::json!({ "contextHeading": "Client notes" }),
            source_policy: "none".into(),
            post_copy_action: "copied".into(),
            advance_policy: "keep".into(),
            revision: 1,
        };
        store.replace_output_profiles(&[row.clone()]).expect("save");
        let loaded = store.list_output_profiles().expect("load");
        assert_eq!(loaded, vec![row]);
        assert_eq!(
            store
                .replace_output_profiles(&[SettingsProfileExport {
                    id: "../etc".into(),
                    builtin_key: None,
                    name: "bad".into(),
                    format: "plain".into(),
                    format_options: serde_json::json!({}),
                    source_policy: "none".into(),
                    post_copy_action: "copied".into(),
                    advance_policy: "keep".into(),
                    revision: 1,
                }])
                .unwrap_err(),
            ProfileStoreError::Invalid
        );
    }
}
