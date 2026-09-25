use std::collections::HashMap;
use std::sync::Mutex;

use crate::RemoteError;

pub const ACCOUNT: &str = "bronze";
pub const SLOT_OPENAI: &str = "bronze.title.hosted.openai";
pub const SLOT_ANTHROPIC: &str = "bronze.title.hosted.anthropic";
pub const SLOT_OPENROUTER: &str = "bronze.title.hosted.openrouter";

pub trait SecretStore: Send + Sync {
    fn put(&self, slot: &str, secret: &str) -> Result<(), RemoteError>;
    fn get(&self, slot: &str) -> Result<Option<String>, RemoteError>;
    fn delete(&self, slot: &str) -> Result<(), RemoteError>;
}

#[derive(Default)]
pub struct MemorySecrets {
    map: Mutex<HashMap<String, String>>,
}

impl MemorySecrets {
    pub fn new() -> Self {
        Self::default()
    }
}

impl SecretStore for MemorySecrets {
    fn put(&self, slot: &str, secret: &str) -> Result<(), RemoteError> {
        if !slot_ok(slot) || secret.is_empty() {
            return Err(RemoteError::MissingKey);
        }
        self.map
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(slot.to_string(), secret.to_string());
        Ok(())
    }

    fn get(&self, slot: &str) -> Result<Option<String>, RemoteError> {
        if !slot_ok(slot) {
            return Err(RemoteError::MissingKey);
        }
        Ok(self
            .map
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(slot)
            .cloned())
    }

    fn delete(&self, slot: &str) -> Result<(), RemoteError> {
        if !slot_ok(slot) {
            return Err(RemoteError::MissingKey);
        }
        self.map
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(slot);
        Ok(())
    }
}

pub struct KeychainSecrets;

impl SecretStore for KeychainSecrets {
    fn put(&self, slot: &str, secret: &str) -> Result<(), RemoteError> {
        if !slot_ok(slot) || secret.is_empty() {
            return Err(RemoteError::MissingKey);
        }
        #[cfg(target_os = "macos")]
        {
            let status = std::process::Command::new("/usr/bin/security")
                .args(["add-generic-password", "-a", ACCOUNT, "-s", slot, "-w"])
                .arg(secret)
                .arg("-U")
                .status()
                .map_err(|_| RemoteError::Unreachable)?;
            if status.success() {
                Ok(())
            } else {
                Err(RemoteError::Unreachable)
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = slot;
            let _ = secret;
            Err(RemoteError::Unreachable)
        }
    }

    fn get(&self, slot: &str) -> Result<Option<String>, RemoteError> {
        if !slot_ok(slot) {
            return Err(RemoteError::MissingKey);
        }
        #[cfg(target_os = "macos")]
        {
            let out = std::process::Command::new("/usr/bin/security")
                .args(["find-generic-password", "-a", ACCOUNT, "-s", slot, "-w"])
                .output()
                .map_err(|_| RemoteError::Unreachable)?;
            if !out.status.success() {
                return Ok(None);
            }
            let key = String::from_utf8(out.stdout).map_err(|_| RemoteError::Unreachable)?;
            let key = key.trim_end_matches('\n').to_string();
            if key.is_empty() {
                Ok(None)
            } else {
                Ok(Some(key))
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = slot;
            Ok(None)
        }
    }

    fn delete(&self, slot: &str) -> Result<(), RemoteError> {
        if !slot_ok(slot) {
            return Err(RemoteError::MissingKey);
        }
        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("/usr/bin/security")
                .args(["delete-generic-password", "-a", ACCOUNT, "-s", slot])
                .status();
            Ok(())
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = slot;
            Ok(())
        }
    }
}

pub fn slot_for_provider(provider: &str) -> Option<&'static str> {
    match provider {
        "hosted-openai" | "openai" => Some(SLOT_OPENAI),
        "hosted-anthropic" | "anthropic" => Some(SLOT_ANTHROPIC),
        "hosted-openrouter" | "openrouter" => Some(SLOT_OPENROUTER),
        _ => None,
    }
}

fn slot_ok(slot: &str) -> bool {
    matches!(slot, SLOT_OPENAI | SLOT_ANTHROPIC | SLOT_OPENROUTER)
}

#[cfg(test)]
mod keystore_tests {
    use super::*;

    #[test]
    fn memory_store_round_trips_and_rejects_unknown_slot() {
        let store = MemorySecrets::new();
        assert_eq!(store.get(SLOT_OPENAI).expect("get"), None);
        store.put(SLOT_OPENAI, "sk-test").expect("put");
        assert_eq!(
            store.get(SLOT_OPENAI).expect("get").as_deref(),
            Some("sk-test")
        );
        store.delete(SLOT_OPENAI).expect("delete");
        assert_eq!(store.get(SLOT_OPENAI).expect("get"), None);
        assert_eq!(store.put("other", "x"), Err(RemoteError::MissingKey));
        assert_eq!(slot_for_provider("hosted-openai"), Some(SLOT_OPENAI));
        assert_eq!(slot_for_provider("needle"), None);
    }
}
