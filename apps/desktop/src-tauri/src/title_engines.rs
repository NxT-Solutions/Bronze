use crate::live_session::LiveSession;
#[cfg(test)]
use bronze_title_remote::MemorySecrets;
use bronze_title_remote::{
    disclosure, parse_hosted_base, slot_for_provider, HostedProvider, KeychainSecrets, SecretStore,
};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

static SECRETS: OnceLock<Mutex<Arc<dyn SecretStore>>> = OnceLock::new();

fn secrets() -> Arc<dyn SecretStore> {
    SECRETS
        .get_or_init(|| Mutex::new(Arc::new(KeychainSecrets) as Arc<dyn SecretStore>))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

pub fn secret_store() -> Arc<dyn SecretStore> {
    secrets()
}

#[cfg(test)]
pub fn install_memory_secrets() -> Arc<MemorySecrets> {
    let store = Arc::new(MemorySecrets::new());
    *SECRETS
        .get_or_init(|| Mutex::new(store.clone() as Arc<dyn SecretStore>))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = store.clone();
    store
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomTitleDto {
    pub id: String,
    pub display_name: String,
    pub bytes: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostedDisclosureDto {
    pub host: String,
    pub payload_class: String,
}

fn pick_gguf_path() -> Result<PathBuf, String> {
    rfd::FileDialog::new()
        .add_filter("GGUF", &["gguf"])
        .pick_file()
        .ok_or_else(|| "cancelled".into())
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn import_title_gguf(
    session: tauri::State<std::sync::Mutex<LiveSession>>,
) -> Result<CustomTitleDto, String> {
    let path = pick_gguf_path()?;
    let dto = session
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .import_custom_gguf_from(&path)?;
    if dto.display_name.contains('/') || dto.id.contains('/') {
        return Err("invalid_custom_gguf".into());
    }
    Ok(dto)
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn list_ollama_title_models() -> Result<Vec<String>, String> {
    bronze_title_remote::list_ollama_models().map_err(|err| err.as_str().into())
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn set_hosted_title_key(provider: String, key: String) -> Result<(), String> {
    let slot = slot_for_provider(&provider).ok_or("unknown_provider")?;
    if key.trim().is_empty() {
        return Err("missing_key".into());
    }
    secrets()
        .put(slot, key.trim())
        .map_err(|err| err.as_str().into())
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn clear_hosted_title_key(provider: String) -> Result<(), String> {
    let slot = slot_for_provider(&provider).ok_or("unknown_provider")?;
    secrets().delete(slot).map_err(|err| err.as_str().into())
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn hosted_title_disclosure(
    provider: String,
    custom_base: String,
) -> Result<HostedDisclosureDto, String> {
    let parsed = HostedProvider::parse(&provider).ok_or("unknown_provider")?;
    if !custom_base.trim().is_empty() {
        parse_hosted_base(&custom_base).map_err(|err| err.as_str().to_string())?;
    }
    let (host, class) = disclosure(parsed, &custom_base).map_err(|err| err.as_str().to_string())?;
    Ok(HostedDisclosureDto {
        host,
        payload_class: class.into(),
    })
}

#[cfg(test)]
mod title_engines_tests {
    use super::*;
    use bronze_title_model::GGUF_MAGIC;
    use bronze_title_remote::{PAYLOAD_CLASS, SLOT_OPENAI};
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static SEQ: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn import_rejects_non_gguf_and_hides_path() {
        let n = SEQ.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("bronze-import-{n}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("dir");
        let mut session = LiveSession::open(dir.clone()).expect("open");
        let bad = dir.join("notes.txt");
        fs::write(&bad, b"hello").expect("write");
        assert_eq!(
            session.import_custom_gguf_from(&bad).unwrap_err(),
            "not_gguf"
        );
        let src = dir.join("tiny.gguf");
        let mut bytes = Vec::from(*GGUF_MAGIC);
        bytes.extend_from_slice(b"data");
        fs::write(&src, bytes).expect("gguf");
        let dto = session.import_custom_gguf_from(&src).expect("import");
        assert_eq!(dto.display_name, "tiny.gguf");
        assert!(!dto.display_name.contains('/'));
        let json = serde_json::to_string(&dto).expect("json");
        assert!(!json.contains(dir.to_string_lossy().as_ref()));
        assert_eq!(session.settings().general.title_model.as_str(), "custom");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn hosted_key_stays_out_of_settings_and_export() {
        let store = install_memory_secrets();
        store.put(SLOT_OPENAI, "sk-live-secret").expect("put");
        let n = SEQ.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("bronze-key-{n}"));
        let _ = fs::remove_dir_all(&dir);
        let session = LiveSession::open(dir.clone()).expect("open");
        let json = session.settings().to_json().expect("json");
        assert!(!json.contains("sk-live-secret"));
        let preview = bronze_settings::export_settings(&session.settings(), &Default::default())
            .expect("export");
        let blob = preview.payload.to_string();
        assert!(!blob.contains("sk-live-secret"));
        assert_eq!(
            hosted_title_disclosure("hosted-openai".into(), String::new())
                .expect("disc")
                .host,
            "api.openai.com"
        );
        assert_eq!(
            hosted_title_disclosure("hosted-openai".into(), "https://127.0.0.1".into())
                .unwrap_err(),
            "blocked_host"
        );
        assert_eq!(PAYLOAD_CLASS, "truncated_capture_2048");
        let _ = fs::remove_dir_all(&dir);
    }
}
