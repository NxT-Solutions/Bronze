use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

static CUSTOM_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

pub fn set_custom_path(path: Option<PathBuf>) {
    *CUSTOM_PATH
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = path;
}

pub fn custom_weights_path() -> Option<PathBuf> {
    CUSTOM_PATH
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

pub const GGUF_MAGIC: &[u8; 4] = b"GGUF";
pub const MAX_CUSTOM_BYTES: u64 = 2 * 1024 * 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CustomGguf {
    pub id: String,
    pub display_name: String,
    pub bytes: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CustomGgufError {
    NotGguf,
    Unreadable,
    TooLarge,
    CopyFailed,
}

impl CustomGgufError {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotGguf => "not_gguf",
            Self::Unreadable => "unreadable",
            Self::TooLarge => "too_large",
            Self::CopyFailed => "copy_failed",
        }
    }
}

pub fn is_gguf_magic(bytes: &[u8]) -> bool {
    bytes.len() >= 4 && bytes[..4] == GGUF_MAGIC[..]
}

pub fn gguf_magic_ok(path: &Path) -> bool {
    let mut magic = [0_u8; 4];
    let Ok(mut file) = File::open(path) else {
        return false;
    };
    file.read_exact(&mut magic).is_ok() && is_gguf_magic(&magic)
}

pub fn custom_id_ok(id: &str) -> bool {
    id.len() == 64 && id.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

pub fn custom_gguf_path(dest_dir: &Path, id: &str) -> Option<PathBuf> {
    if !custom_id_ok(id) {
        return None;
    }
    Some(dest_dir.join(format!("{id}.gguf")))
}

pub fn display_name_from(path: &Path) -> Option<String> {
    let name = path.file_name()?.to_str()?;
    if name.is_empty() || name.contains('/') || name.contains('\\') || name.contains('\0') {
        return None;
    }
    Some(name.to_string())
}

pub fn import_custom_gguf(src: &Path, dest_dir: &Path) -> Result<CustomGguf, CustomGgufError> {
    let meta = fs::metadata(src).map_err(|_| CustomGgufError::Unreadable)?;
    if !meta.is_file() {
        return Err(CustomGgufError::Unreadable);
    }
    if meta.len() > MAX_CUSTOM_BYTES {
        return Err(CustomGgufError::TooLarge);
    }
    if !gguf_magic_ok(src) {
        return Err(CustomGgufError::NotGguf);
    }
    let display_name = display_name_from(src).ok_or(CustomGgufError::Unreadable)?;
    let (id, bytes) = hash_file(src)?;
    fs::create_dir_all(dest_dir).map_err(|_| CustomGgufError::CopyFailed)?;
    let dest = dest_dir.join(format!("{id}.gguf"));
    if !dest.is_file() {
        copy_file(src, &dest)?;
    }
    Ok(CustomGguf {
        id,
        display_name,
        bytes,
    })
}

fn hash_file(path: &Path) -> Result<(String, u64), CustomGgufError> {
    let mut file = File::open(path).map_err(|_| CustomGgufError::Unreadable)?;
    let mut hasher = Sha256::new();
    let mut buf = [0_u8; 64 * 1024];
    let mut bytes = 0_u64;
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|_| CustomGgufError::Unreadable)?;
        if n == 0 {
            break;
        }
        bytes += n as u64;
        hasher.update(&buf[..n]);
    }
    Ok((hex_lower(&hasher.finalize()), bytes))
}

fn copy_file(src: &Path, dest: &Path) -> Result<(), CustomGgufError> {
    let mut input = File::open(src).map_err(|_| CustomGgufError::CopyFailed)?;
    let mut output = File::create(dest).map_err(|_| CustomGgufError::CopyFailed)?;
    let mut buf = [0_u8; 64 * 1024];
    loop {
        let n = input
            .read(&mut buf)
            .map_err(|_| CustomGgufError::CopyFailed)?;
        if n == 0 {
            break;
        }
        output
            .write_all(&buf[..n])
            .map_err(|_| CustomGgufError::CopyFailed)?;
    }
    output.flush().map_err(|_| CustomGgufError::CopyFailed)
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod custom_tests {
    use super::*;

    #[test]
    fn rejects_non_gguf_magic_and_does_not_copy() {
        let tmp = std::env::temp_dir().join(format!(
            "bronze-custom-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        let dest = tmp.join("out");
        fs::create_dir_all(&tmp).expect("tmp");
        let src = tmp.join("notes.txt");
        fs::write(&src, b"not a model").expect("write");
        assert!(!is_gguf_magic(b"not a model"));
        assert_eq!(
            import_custom_gguf(&src, &dest),
            Err(CustomGgufError::NotGguf)
        );
        assert!(!dest.exists() || dest.read_dir().map(|d| d.count()).unwrap_or(0) == 0);
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn copies_gguf_magic_file_and_hides_path_from_dto() {
        let tmp = std::env::temp_dir().join(format!(
            "bronze-custom-ok-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        let dest = tmp.join("title-engines");
        fs::create_dir_all(&tmp).expect("tmp");
        let src = tmp.join("tiny.gguf");
        let mut bytes = Vec::from(*GGUF_MAGIC);
        bytes.extend_from_slice(&[1, 2, 3, 4]);
        fs::write(&src, &bytes).expect("write");
        let imported = import_custom_gguf(&src, &dest).expect("import");
        assert_eq!(imported.display_name, "tiny.gguf");
        assert!(!imported.display_name.contains('/'));
        assert_eq!(imported.bytes, bytes.len() as u64);
        assert!(custom_id_ok(&imported.id));
        let stored = custom_gguf_path(&dest, &imported.id).expect("path");
        assert!(stored.is_file());
        assert!(gguf_magic_ok(&stored));
        assert_eq!(custom_gguf_path(&dest, "../escape"), None);
        let _ = fs::remove_dir_all(&tmp);
    }
}
