//! Deterministic JSON archive + Markdown (story 4.6, DAT-003, SET-001, I18N-003).

use crate::migrate::Store;
use bronze_domain::ContentLanguage;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path};

pub const SECRET_BODY_WARNING_KEY: &str = "export.preview.secretBodies";
pub const EXPORT_FORMAT: &str = "bronze-export";
pub const EXPORT_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Overwrite {
    Fail,
    Replace,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImportStrategy {
    Merge,
    Replace,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExportError {
    PathExists,
    Io,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImportError {
    PathTraversal,
    ChecksumMismatch,
    Schema,
    Conflict,
    InvalidContentLanguage,
    Io,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExportPreview {
    pub item_count: usize,
    pub section_count: usize,
    pub warning_keys: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportPreview {
    pub item_count: usize,
    pub section_count: usize,
    pub warning_keys: Vec<String>,
}

pub fn filter_settings_for_export(raw: &BTreeMap<String, String>) -> BTreeMap<String, Value> {
    raw.iter()
        .filter(|(key, value)| {
            settings_key_exportable(key) && !value_looks_like_machine_path(value)
        })
        .map(|(k, v)| (k.clone(), parse_or_string(v)))
        .collect()
}

fn settings_key_exportable(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    ![
        "diagnostic",
        "path",
        "token",
        "credential",
        "secret",
        "installidentity",
    ]
    .iter()
    .any(|frag| lower.contains(frag))
}

fn value_looks_like_machine_path(value: &str) -> bool {
    let trimmed = value.trim().trim_matches('"');
    trimmed.starts_with('/') || trimmed.contains(":\\")
}

fn parse_or_string(raw: &str) -> Value {
    serde_json::from_str(raw).unwrap_or_else(|_| Value::String(raw.to_string()))
}

pub fn check_entry_path(rel: &str) -> Result<(), ImportError> {
    if rel.is_empty() || rel.contains('\0') {
        return Err(ImportError::PathTraversal);
    }
    let path = Path::new(rel);
    if path.is_absolute() {
        return Err(ImportError::PathTraversal);
    }
    for component in path.components() {
        match component {
            Component::Normal(_) | Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(ImportError::PathTraversal);
            }
        }
    }
    Ok(())
}

fn rfc3339_utc(ms: i64) -> String {
    let secs = ms.div_euclid(1000);
    let days = secs.div_euclid(86_400);
    let tod = secs.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = tod / 3600;
    let min = (tod % 3600) / 60;
    let sec = tod % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{min:02}:{sec:02}Z")
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m as u32, d as u32)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
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

fn json_pretty(value: &Value) -> String {
    let mut out = serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".into());
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn object(pairs: &[(&str, Value)]) -> Value {
    let mut map = Map::new();
    for (k, v) in pairs {
        map.insert((*k).to_string(), v.clone());
    }
    Value::Object(map)
}

fn note_filename(title: &str, id: &str) -> String {
    let slug: String = title
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let id_safe: String = id
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect();
    format!("{slug}-{id_safe}.md")
}

struct SectionRow {
    id: String,
    workspace_id: String,
    title: String,
    rank: String,
    state: String,
    color_token: Option<String>,
    revision: i64,
    created_at_ms: i64,
    updated_at_ms: i64,
}

struct ItemRow {
    id: String,
    section_id: String,
    kind: String,
    body: String,
    content_language: String,
    status: String,
    rank: String,
    source_id: Option<String>,
    revision: i64,
    created_at_ms: i64,
    updated_at_ms: i64,
}

impl Store {
    pub fn export_preview(&self) -> Result<ExportPreview, ExportError> {
        let section_count = count_rows(&self.conn, "sections").map_err(|_| ExportError::Io)?;
        let item_count = count_rows(&self.conn, "items").map_err(|_| ExportError::Io)?;
        Ok(ExportPreview {
            item_count,
            section_count,
            warning_keys: vec![SECRET_BODY_WARNING_KEY.to_string()],
        })
    }

    pub fn export_archive(
        &self,
        dest: &Path,
        exported_at_ms: i64,
        overwrite: Overwrite,
    ) -> Result<ExportPreview, ExportError> {
        if dest.exists() {
            if overwrite != Overwrite::Replace {
                return Err(ExportError::PathExists);
            }
            fs::remove_dir_all(dest).map_err(|_| ExportError::Io)?;
        }
        let preview = self.export_preview()?;
        let files = self.build_archive_files(exported_at_ms)?;
        let parent = dest.parent().ok_or(ExportError::Io)?;
        let staging = parent.join(format!(
            ".{}.staging",
            dest.file_name().ok_or(ExportError::Io)?.to_string_lossy()
        ));
        if staging.exists() {
            fs::remove_dir_all(&staging).map_err(|_| ExportError::Io)?;
        }
        fs::create_dir_all(staging.join("notes")).map_err(|_| ExportError::Io)?;
        for (rel, bytes) in &files {
            check_entry_path(rel).map_err(|_| ExportError::Io)?;
            let path = staging.join(rel);
            if let Some(dir) = path.parent() {
                fs::create_dir_all(dir).map_err(|_| ExportError::Io)?;
            }
            fs::write(&path, bytes).map_err(|_| ExportError::Io)?;
        }
        fs::rename(&staging, dest).map_err(|_| ExportError::Io)?;
        Ok(preview)
    }

    pub fn import_preview(&self, root: &Path) -> Result<ImportPreview, ImportError> {
        let files = read_archive_dir(root)?;
        validate_archive(&files)?;
        let data = parse_data(&files)?;
        let items = data
            .get("items")
            .and_then(Value::as_array)
            .ok_or(ImportError::Schema)?;
        let sections = data
            .get("sections")
            .and_then(Value::as_array)
            .ok_or(ImportError::Schema)?;
        Ok(ImportPreview {
            item_count: items.len(),
            section_count: sections.len(),
            warning_keys: vec![SECRET_BODY_WARNING_KEY.to_string()],
        })
    }

    pub fn import_commit(
        &mut self,
        root: &Path,
        strategy: ImportStrategy,
    ) -> Result<ImportPreview, ImportError> {
        let preview = self.import_preview(root)?;
        let files = read_archive_dir(root)?;
        let data = parse_data(&files)?;
        if strategy == ImportStrategy::Replace {
            self.conn
                .execute_batch(
                    "DELETE FROM items;
                     DELETE FROM sources;
                     DELETE FROM sections;
                     DELETE FROM output_profiles;
                     DELETE FROM shortcuts;
                     DELETE FROM settings;",
                )
                .map_err(|_| ImportError::Io)?;
        }
        insert_workspace(&self.conn, &data)?;
        insert_sections(&self.conn, &data, strategy)?;
        insert_items(&self.conn, &data, strategy)?;
        insert_settings(&self.conn, &data)?;
        Ok(preview)
    }

    fn build_archive_files(
        &self,
        exported_at_ms: i64,
    ) -> Result<BTreeMap<String, Vec<u8>>, ExportError> {
        let data = self.data_json(exported_at_ms)?;
        let notes = self.markdown_notes()?;
        let mut files = BTreeMap::new();
        files.insert("data.json".into(), json_pretty(&data).into_bytes());
        for (name, body) in notes {
            files.insert(format!("notes/{name}"), body.into_bytes());
        }
        let checksum_rows: Vec<Value> = files
            .iter()
            .map(|(path, bytes)| {
                object(&[
                    ("file", Value::String(path.clone())),
                    ("sha256", Value::String(sha256_hex(bytes))),
                ])
            })
            .collect();
        let manifest = object(&[
            ("checksums", Value::Array(checksum_rows)),
            ("exportedAt", Value::String(rfc3339_utc(exported_at_ms))),
            ("format", Value::String(EXPORT_FORMAT.into())),
            ("schemaVersion", Value::from(EXPORT_VERSION)),
            ("version", Value::from(EXPORT_VERSION)),
        ]);
        files.insert("manifest.json".into(), json_pretty(&manifest).into_bytes());
        let mut listing: Vec<String> = files
            .iter()
            .map(|(path, bytes)| format!("{}  {path}", sha256_hex(bytes)))
            .collect();
        listing.sort();
        listing.push(String::new());
        files.insert("checksums.sha256".into(), listing.join("\n").into_bytes());
        Ok(files)
    }

    fn data_json(&self, exported_at_ms: i64) -> Result<Value, ExportError> {
        let workspace = load_workspace(&self.conn).map_err(|_| ExportError::Io)?;
        let sections = load_sections(&self.conn).map_err(|_| ExportError::Io)?;
        let items = load_items(&self.conn).map_err(|_| ExportError::Io)?;
        let settings = load_settings(&self.conn).map_err(|_| ExportError::Io)?;
        Ok(object(&[
            ("exportedAt", Value::String(rfc3339_utc(exported_at_ms))),
            ("format", Value::String(EXPORT_FORMAT.into())),
            (
                "items",
                Value::Array(items.iter().map(item_value).collect()),
            ),
            ("outputProfiles", Value::Array(vec![])),
            (
                "sections",
                Value::Array(sections.iter().map(section_value).collect()),
            ),
            (
                "settings",
                Value::Object(filter_settings_for_export(&settings).into_iter().collect()),
            ),
            ("shortcuts", Value::Array(vec![])),
            ("sources", Value::Array(vec![])),
            ("version", Value::from(EXPORT_VERSION)),
            ("workspace", workspace),
        ]))
    }

    fn markdown_notes(&self) -> Result<BTreeMap<String, String>, ExportError> {
        let sections = load_sections(&self.conn).map_err(|_| ExportError::Io)?;
        let items = load_items(&self.conn).map_err(|_| ExportError::Io)?;
        let mut notes = BTreeMap::new();
        for section in &sections {
            let mut body = format!(
                "---\nschema: bronze-note/1\nsectionId: {}\n---\n",
                section.id
            );
            for item in items.iter().filter(|i| i.section_id == section.id) {
                body.push_str(&format!(
                    "\n## {}\ncontentLanguage: {}\n\n{}\n",
                    item.id, item.content_language, item.body
                ));
            }
            notes.insert(note_filename(&section.title, &section.id), body);
        }
        Ok(notes)
    }
}

fn count_rows(conn: &rusqlite::Connection, table: &str) -> rusqlite::Result<usize> {
    conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
        row.get::<_, i64>(0).map(|n| n as usize)
    })
}

fn load_workspace(conn: &rusqlite::Connection) -> rusqlite::Result<Value> {
    let mut stmt = conn.prepare("SELECT id, name FROM workspaces ORDER BY id LIMIT 1")?;
    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next()? {
        Ok(object(&[
            ("id", Value::String(row.get(0)?)),
            ("name", Value::String(row.get(1)?)),
        ]))
    } else {
        Ok(object(&[
            ("id", Value::String("default".into())),
            ("name", Value::String("Bronze".into())),
        ]))
    }
}

fn load_sections(conn: &rusqlite::Connection) -> rusqlite::Result<Vec<SectionRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, workspace_id, title, rank, state, color_token, revision, created_at_ms, updated_at_ms
         FROM sections ORDER BY id",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(SectionRow {
            id: row.get(0)?,
            workspace_id: row.get(1)?,
            title: row.get(2)?,
            rank: row.get(3)?,
            state: row.get(4)?,
            color_token: row.get(5)?,
            revision: row.get(6)?,
            created_at_ms: row.get(7)?,
            updated_at_ms: row.get(8)?,
        })
    })?;
    rows.collect()
}

fn load_items(conn: &rusqlite::Connection) -> rusqlite::Result<Vec<ItemRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, section_id, kind, body, content_language, status, rank, source_id, revision, created_at_ms, updated_at_ms
         FROM items ORDER BY id",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(ItemRow {
            id: row.get(0)?,
            section_id: row.get(1)?,
            kind: row.get(2)?,
            body: row.get(3)?,
            content_language: row.get(4)?,
            status: row.get(5)?,
            rank: row.get(6)?,
            source_id: row.get(7)?,
            revision: row.get(8)?,
            created_at_ms: row.get(9)?,
            updated_at_ms: row.get(10)?,
        })
    })?;
    rows.collect()
}

fn load_settings(conn: &rusqlite::Connection) -> rusqlite::Result<BTreeMap<String, String>> {
    let mut stmt = conn.prepare("SELECT key, json_value FROM settings ORDER BY key")?;
    let mut map = BTreeMap::new();
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, Option<String>>(1)?.unwrap_or_default(),
        ))
    })?;
    for row in rows {
        let (k, v) = row?;
        map.insert(k, v);
    }
    Ok(map)
}

fn section_value(row: &SectionRow) -> Value {
    object(&[
        (
            "colorToken",
            row.color_token
                .as_ref()
                .map(|s| Value::String(s.clone()))
                .unwrap_or(Value::Null),
        ),
        ("createdAtMs", Value::from(row.created_at_ms)),
        ("id", Value::String(row.id.clone())),
        ("rank", Value::String(row.rank.clone())),
        ("revision", Value::from(row.revision)),
        ("state", Value::String(row.state.clone())),
        ("title", Value::String(row.title.clone())),
        ("updatedAtMs", Value::from(row.updated_at_ms)),
        ("workspaceId", Value::String(row.workspace_id.clone())),
    ])
}

fn item_value(row: &ItemRow) -> Value {
    object(&[
        ("body", Value::String(row.body.clone())),
        (
            "contentLanguage",
            Value::String(row.content_language.clone()),
        ),
        ("createdAtMs", Value::from(row.created_at_ms)),
        ("id", Value::String(row.id.clone())),
        ("kind", Value::String(row.kind.clone())),
        ("rank", Value::String(row.rank.clone())),
        ("revision", Value::from(row.revision)),
        ("sectionId", Value::String(row.section_id.clone())),
        (
            "sourceId",
            row.source_id
                .as_ref()
                .map(|s| Value::String(s.clone()))
                .unwrap_or(Value::Null),
        ),
        ("status", Value::String(row.status.clone())),
        ("updatedAtMs", Value::from(row.updated_at_ms)),
    ])
}

fn read_archive_dir(root: &Path) -> Result<BTreeMap<String, Vec<u8>>, ImportError> {
    let mut files = BTreeMap::new();
    read_tree(root, root, &mut files)?;
    Ok(files)
}

fn read_tree(
    root: &Path,
    dir: &Path,
    files: &mut BTreeMap<String, Vec<u8>>,
) -> Result<(), ImportError> {
    let entries = fs::read_dir(dir).map_err(|_| ImportError::Io)?;
    for entry in entries {
        let entry = entry.map_err(|_| ImportError::Io)?;
        let path = entry.path();
        let meta = fs::symlink_metadata(&path).map_err(|_| ImportError::Io)?;
        if meta.file_type().is_symlink() {
            return Err(ImportError::PathTraversal);
        }
        let rel = path
            .strip_prefix(root)
            .map_err(|_| ImportError::PathTraversal)?;
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        check_entry_path(&rel_str)?;
        if meta.is_dir() {
            read_tree(root, &path, files)?;
        } else {
            files.insert(rel_str, fs::read(&path).map_err(|_| ImportError::Io)?);
        }
    }
    Ok(())
}

fn validate_archive(files: &BTreeMap<String, Vec<u8>>) -> Result<(), ImportError> {
    for path in files.keys() {
        check_entry_path(path)?;
    }
    let manifest_bytes = files.get("manifest.json").ok_or(ImportError::Schema)?;
    let manifest: Value =
        serde_json::from_slice(manifest_bytes).map_err(|_| ImportError::Schema)?;
    let checksums = manifest
        .get("checksums")
        .and_then(Value::as_array)
        .ok_or(ImportError::Schema)?;
    for row in checksums {
        let file = row
            .get("file")
            .and_then(Value::as_str)
            .ok_or(ImportError::Schema)?;
        check_entry_path(file)?;
        let expected = row
            .get("sha256")
            .and_then(Value::as_str)
            .ok_or(ImportError::Schema)?;
        let bytes = files.get(file).ok_or(ImportError::ChecksumMismatch)?;
        if sha256_hex(bytes) != expected {
            return Err(ImportError::ChecksumMismatch);
        }
    }
    Ok(())
}

fn parse_data(files: &BTreeMap<String, Vec<u8>>) -> Result<Value, ImportError> {
    let bytes = files.get("data.json").ok_or(ImportError::Schema)?;
    let data: Value = serde_json::from_slice(bytes).map_err(|_| ImportError::Schema)?;
    if data.get("format").and_then(Value::as_str) != Some(EXPORT_FORMAT) {
        return Err(ImportError::Schema);
    }
    let items = data
        .get("items")
        .and_then(Value::as_array)
        .ok_or(ImportError::Schema)?;
    for item in items {
        let lang = item
            .get("contentLanguage")
            .and_then(Value::as_str)
            .ok_or(ImportError::Schema)?;
        ContentLanguage::parse(Some(lang)).map_err(|_| ImportError::InvalidContentLanguage)?;
    }
    Ok(data)
}

fn insert_workspace(conn: &rusqlite::Connection, data: &Value) -> Result<(), ImportError> {
    let Some(ws) = data.get("workspace") else {
        return Ok(());
    };
    let id = ws.get("id").and_then(Value::as_str).unwrap_or("default");
    let name = ws.get("name").and_then(Value::as_str).unwrap_or("Bronze");
    conn.execute(
        "INSERT OR IGNORE INTO workspaces (id, name, created_at_ms, updated_at_ms) VALUES (?1, ?2, 0, 0)",
        rusqlite::params![id, name],
    )
    .map_err(|_| ImportError::Io)?;
    Ok(())
}

fn insert_sections(
    conn: &rusqlite::Connection,
    data: &Value,
    strategy: ImportStrategy,
) -> Result<(), ImportError> {
    let Some(sections) = data.get("sections").and_then(Value::as_array) else {
        return Ok(());
    };
    for section in sections {
        let id = section
            .get("id")
            .and_then(Value::as_str)
            .ok_or(ImportError::Schema)?;
        if strategy == ImportStrategy::Merge && exists(conn, "sections", id)? {
            return Err(ImportError::Conflict);
        }
        conn.execute(
            "INSERT INTO sections (id, workspace_id, title, rank, state, color_token, revision, created_at_ms, updated_at_ms, deleted_at_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, NULL)",
            rusqlite::params![
                id,
                section
                    .get("workspaceId")
                    .and_then(Value::as_str)
                    .unwrap_or("default"),
                section.get("title").and_then(Value::as_str).unwrap_or(""),
                section.get("rank").and_then(Value::as_str).unwrap_or("a"),
                section
                    .get("state")
                    .and_then(Value::as_str)
                    .unwrap_or("active"),
                section.get("colorToken").and_then(Value::as_str),
                section.get("revision").and_then(Value::as_i64).unwrap_or(1),
                section.get("createdAtMs").and_then(Value::as_i64).unwrap_or(0),
                section.get("updatedAtMs").and_then(Value::as_i64).unwrap_or(0),
            ],
        )
        .map_err(|_| ImportError::Io)?;
    }
    Ok(())
}

fn insert_items(
    conn: &rusqlite::Connection,
    data: &Value,
    strategy: ImportStrategy,
) -> Result<(), ImportError> {
    let Some(items) = data.get("items").and_then(Value::as_array) else {
        return Ok(());
    };
    for item in items {
        let id = item
            .get("id")
            .and_then(Value::as_str)
            .ok_or(ImportError::Schema)?;
        if strategy == ImportStrategy::Merge && exists(conn, "items", id)? {
            return Err(ImportError::Conflict);
        }
        let lang = item
            .get("contentLanguage")
            .and_then(Value::as_str)
            .unwrap_or("und");
        ContentLanguage::parse(Some(lang)).map_err(|_| ImportError::InvalidContentLanguage)?;
        conn.execute(
            "INSERT INTO items (id, section_id, kind, body, content_language, status, rank, source_id, revision, created_at_ms, updated_at_ms, completed_at_ms, deleted_at_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, NULL, NULL)",
            rusqlite::params![
                id,
                item.get("sectionId").and_then(Value::as_str).unwrap_or(""),
                item.get("kind").and_then(Value::as_str).unwrap_or("note"),
                item.get("body").and_then(Value::as_str).unwrap_or(""),
                lang,
                item.get("status").and_then(Value::as_str).unwrap_or("queued"),
                item.get("rank").and_then(Value::as_str).unwrap_or("a"),
                item.get("sourceId").and_then(Value::as_str),
                item.get("revision").and_then(Value::as_i64).unwrap_or(1),
                item.get("createdAtMs").and_then(Value::as_i64).unwrap_or(0),
                item.get("updatedAtMs").and_then(Value::as_i64).unwrap_or(0),
            ],
        )
        .map_err(|_| ImportError::Io)?;
    }
    Ok(())
}

fn insert_settings(conn: &rusqlite::Connection, data: &Value) -> Result<(), ImportError> {
    let Some(settings) = data.get("settings").and_then(Value::as_object) else {
        return Ok(());
    };
    let raw: BTreeMap<String, String> = settings
        .iter()
        .map(|(k, v)| {
            (
                k.clone(),
                match v {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                },
            )
        })
        .collect();
    for (key, value) in filter_settings_for_export(&raw) {
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, schema_version, json_value, updated_at_ms) VALUES (?1, 1, ?2, 0)",
            rusqlite::params![key, value.to_string()],
        )
        .map_err(|_| ImportError::Io)?;
    }
    Ok(())
}

fn exists(conn: &rusqlite::Connection, table: &str, id: &str) -> Result<bool, ImportError> {
    let sql = format!("SELECT 1 FROM {table} WHERE id = ?1 LIMIT 1");
    let found: Option<i64> = conn
        .query_row(&sql, [id], |row| row.get(0))
        .optional()
        .map_err(|_| ImportError::Io)?;
    Ok(found.is_some())
}

use rusqlite::OptionalExtension;

#[cfg(test)]
mod export_tests {
    use super::*;
    use crate::migrate::{NoopBackup, PathLocator};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static SEQ: AtomicU64 = AtomicU64::new(1);

    fn open_store() -> (Store, PathBuf) {
        let n = SEQ.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("bronze-export-{nanos}-{n}"));
        fs::create_dir_all(&dir).expect("dir");
        let locator = PathLocator {
            path: dir.join("bronze.sqlite"),
        };
        let mut backup = NoopBackup;
        let store = Store::open(&locator, &mut backup).expect("open");
        store
            .conn
            .execute_batch(
                "INSERT INTO workspaces VALUES ('w1','ws',1,1);
                 INSERT INTO sections VALUES ('s-b','w1','Beta','b','active',NULL,1,1,1,NULL);
                 INSERT INTO sections VALUES ('s-a','w1','Alpha','a','active',NULL,1,1,1,NULL);
                 INSERT INTO items VALUES ('i-b','s-a','note','secret token','fr','queued','b',NULL,1,1,1,NULL,NULL);
                 INSERT INTO items VALUES ('i-a','s-a','note','hello','en-US','queued','a',NULL,1,1,1,NULL,NULL);
                 INSERT INTO settings VALUES ('theme','1','\"dark\"',1);
                 INSERT INTO settings VALUES ('diagnosticFlush','1','true',1);
                 INSERT INTO settings VALUES ('logPath','1','\"/Users/me/bronze.log\"',1);
                 INSERT INTO settings VALUES ('permissionToken','1','\"abc\"',1);",
            )
            .expect("seed");
        (store, dir)
    }

    #[test]
    fn export_manifest_is_locale_neutral_and_sorted() {
        let (store, dir) = open_store();
        let dest = dir.join("out");
        assert_eq!(rfc3339_utc(0), "1970-01-01T00:00:00Z");
        let preview = store
            .export_archive(&dest, 1_704_067_200_000, Overwrite::Fail)
            .expect("export");
        assert_eq!(preview.warning_keys, [SECRET_BODY_WARNING_KEY]);
        let manifest = fs::read_to_string(dest.join("manifest.json")).expect("manifest");
        assert!(manifest.contains("\"exportedAt\": \"2024-01-01T00:00:00Z\""));
        assert!(!manifest.contains("January"));
        let data = fs::read_to_string(dest.join("data.json")).expect("data");
        let items = data.find("\"items\"").expect("items");
        let first = data[items..].find("\"i-a\"").expect("i-a");
        let second = data[items..].find("\"i-b\"").expect("i-b");
        assert!(first < second);
        assert!(data.contains("\"contentLanguage\": \"en-US\""));
        let note = fs::read_to_string(dest.join("notes/alpha-s-a.md")).expect("note");
        assert!(note.contains("contentLanguage: en-US"));
        assert!(note.contains("contentLanguage: fr"));
        let dest2 = dir.join("out2");
        store
            .export_archive(&dest2, 1_704_067_200_000, Overwrite::Fail)
            .expect("export2");
        assert_eq!(
            fs::read(dest.join("data.json")).unwrap(),
            fs::read(dest2.join("data.json")).unwrap()
        );
    }

    #[test]
    fn export_refuses_silent_overwrite() {
        let (store, dir) = open_store();
        let dest = dir.join("out");
        store
            .export_archive(&dest, 0, Overwrite::Fail)
            .expect("first");
        assert_eq!(
            store.export_archive(&dest, 0, Overwrite::Fail).unwrap_err(),
            ExportError::PathExists
        );
    }

    #[test]
    fn settings_export_excludes_diagnostics_paths_tokens() {
        let mut raw = BTreeMap::new();
        raw.insert("theme".into(), "\"dark\"".into());
        raw.insert("diagnosticFlush".into(), "true".into());
        raw.insert("logPath".into(), "\"/tmp/x\"".into());
        raw.insert("permissionToken".into(), "\"abc\"".into());
        raw.insert("backupSchedule".into(), "\"daily\"".into());
        let filtered = filter_settings_for_export(&raw);
        assert!(filtered.contains_key("theme"));
        assert!(filtered.contains_key("backupSchedule"));
        assert!(!filtered.contains_key("diagnosticFlush"));
        assert!(!filtered.contains_key("logPath"));
        assert!(!filtered.contains_key("permissionToken"));
        let (store, dir) = open_store();
        let dest = dir.join("out");
        store
            .export_archive(&dest, 0, Overwrite::Fail)
            .expect("export");
        let data = fs::read_to_string(dest.join("data.json")).expect("data");
        assert!(data.contains("\"theme\""));
        assert!(!data.contains("diagnostic"));
        assert!(!data.contains("permissionToken"));
        assert!(!data.contains("/Users/me/bronze.log"));
        assert!(!data.contains("diagnostic_events"));
    }

    #[test]
    fn path_traversal_archives_fail() {
        assert_eq!(
            check_entry_path("../etc/passwd"),
            Err(ImportError::PathTraversal)
        );
        assert_eq!(
            check_entry_path("/etc/passwd"),
            Err(ImportError::PathTraversal)
        );
        assert_eq!(
            check_entry_path("notes/../../secret"),
            Err(ImportError::PathTraversal)
        );
        assert_eq!(check_entry_path("notes/alpha-s-a.md"), Ok(()));
        let (_store, dir) = open_store();
        let hostile = dir.join("hostile");
        fs::create_dir_all(hostile.join("notes")).expect("dir");
        fs::write(
            hostile.join("manifest.json"),
            r#"{"checksums":[{"file":"../escape","sha256":"00"}],"format":"bronze-export","version":1}"#,
        )
        .expect("manifest");
        fs::write(
            hostile.join("data.json"),
            r#"{"format":"bronze-export","items":[],"sections":[]}"#,
        )
        .expect("data");
        let store = {
            let locator = PathLocator {
                path: dir.join("import.sqlite"),
            };
            let mut backup = NoopBackup;
            Store::open(&locator, &mut backup).expect("open")
        };
        assert_eq!(
            store.import_preview(&hostile).unwrap_err(),
            ImportError::PathTraversal
        );
    }

    #[test]
    fn import_preserves_content_language_and_refuses_silent_overwrite() {
        let (store, dir) = open_store();
        let dest = dir.join("out");
        store
            .export_archive(&dest, 0, Overwrite::Fail)
            .expect("export");
        let mut fresh = {
            let locator = PathLocator {
                path: dir.join("fresh.sqlite"),
            };
            let mut backup = NoopBackup;
            Store::open(&locator, &mut backup).expect("open")
        };
        fresh
            .import_commit(&dest, ImportStrategy::Merge)
            .expect("import");
        let lang: String = fresh
            .conn
            .query_row(
                "SELECT content_language FROM items WHERE id='i-a'",
                [],
                |row| row.get(0),
            )
            .expect("lang");
        assert_eq!(lang, "en-US");
        assert_eq!(
            fresh
                .import_commit(&dest, ImportStrategy::Merge)
                .unwrap_err(),
            ImportError::Conflict
        );
    }
}
