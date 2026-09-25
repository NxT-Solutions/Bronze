use serde::{Deserialize, Serialize};
use std::time::Duration;

const RELEASES_LATEST_URL: &str =
    "https://api.github.com/repos/NxT-Solutions/Bronze/releases/latest";
const RELEASES_PAGE_URL: &str = "https://github.com/NxT-Solutions/Bronze/releases/latest";
const USER_AGENT: &str = "bronze-desktop";
const BREW_UPGRADE: &str = "brew upgrade --cask bronze";
const FORCE_DIALOG_ENV: &str = "BRONZE_FORCE_UPDATE_DIALOG";
const FORCE_VERSION_ENV: &str = "BRONZE_FORCE_UPDATE_VERSION";
const FORCE_URL_ENV: &str = "BRONZE_FORCE_UPDATE_URL";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InstallSource {
    Debug,
    Homebrew,
    Direct,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum UpdateAction {
    None,
    OpenRelease,
    BrewUpgrade,
    Debug,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionInfoDto {
    pub version: String,
    pub install_source: InstallSource,
    pub can_check: bool,
    pub release_url: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheckDto {
    pub current_version: String,
    pub latest_version: Option<String>,
    pub available: bool,
    pub notes: Vec<String>,
    pub release_url: String,
    pub install_source: InstallSource,
    pub action: UpdateAction,
    pub action_command: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: String,
    #[serde(default)]
    body: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct SemVer {
    major: u64,
    minor: u64,
    patch: u64,
}

pub fn current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

pub fn classify_install_path(path: &str, debug_build: bool) -> InstallSource {
    let lower = path.replace('\\', "/").to_ascii_lowercase();
    if debug_build || lower.contains("/target/") {
        return InstallSource::Debug;
    }
    if lower.contains("/caskroom/") || lower.contains("/cellar/") {
        return InstallSource::Homebrew;
    }
    if lower.contains("/applications/bronze.app/") || lower.ends_with("/applications/bronze.app") {
        return InstallSource::Direct;
    }
    InstallSource::Unknown
}

pub fn detect_install_source() -> InstallSource {
    let raw = std::env::current_exe().ok();
    let display = raw
        .as_ref()
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_default();
    let canonical = raw
        .as_ref()
        .and_then(|path| std::fs::canonicalize(path).ok())
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_else(|| display.clone());
    classify_install_path(&canonical, cfg!(debug_assertions))
}

pub fn allowed_release_url(url: &str) -> bool {
    let url = url.trim();
    let Some(rest) = url.strip_prefix("https://github.com/NxT-Solutions/Bronze/releases") else {
        return false;
    };
    if rest.contains("..") || rest.contains('@') || rest.contains('\\') || rest.contains('\n') {
        return false;
    }
    rest.is_empty() || rest.starts_with('/') || rest.starts_with('?')
}

const MAX_UPDATE_NOTES: usize = 5;
const NOTE_CHECK_FOR_UPDATES: &str = "Check for updates is easier to see.";
const NOTE_ENGINE_SWITCH: &str =
    "Switching the on-this-Mac title engine finishes instead of staying on Loading.";
const NOTE_PERMISSIONS: &str = "Bronze asks once for the macOS permissions it uses.";

pub fn release_notes_from_markdown(markdown: &str) -> Vec<String> {
    let lines: Vec<&str> = markdown.lines().collect();
    if let Some(section) = whats_new_section(&lines) {
        return take_notes(section.iter().map(String::as_str), true);
    }
    take_notes(lines.into_iter(), false)
}

fn whats_new_section(lines: &[&str]) -> Option<Vec<String>> {
    let mut start = None;
    for (index, line) in lines.iter().enumerate() {
        if heading_text(line).is_some_and(|text| is_whats_new_heading(&text)) {
            start = Some(index + 1);
            break;
        }
    }
    let start = start?;
    let mut section = Vec::new();
    for line in &lines[start..] {
        if heading_text(line).is_some() {
            break;
        }
        section.push((*line).to_string());
    }
    Some(section)
}

fn take_notes<'a>(lines: impl Iterator<Item = &'a str>, from_whats_new: bool) -> Vec<String> {
    let mut notes = Vec::new();
    for raw in lines {
        if notes.len() >= MAX_UPDATE_NOTES {
            break;
        }
        let Some(note) = note_from_line(raw, from_whats_new) else {
            continue;
        };
        if notes.iter().any(|existing| existing == &note) {
            continue;
        }
        notes.push(note);
    }
    notes
}

fn note_from_line(raw: &str, from_whats_new: bool) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || heading_text(trimmed).is_some() {
        return None;
    }
    let cleaned = clean_note_line(trimmed);
    if cleaned.is_empty() || is_version_line(&cleaned) || is_changelog_trailer(&cleaned) {
        return None;
    }
    if let Some((kind, desc)) = conventional_commit(&cleaned) {
        return map_user_change(kind, desc);
    }
    if cleaned.contains('@') {
        return None;
    }
    if !from_whats_new && (looks_like_commit_subject(&cleaned) || !cleaned.contains(' ')) {
        return None;
    }
    Some(cleaned)
}

fn heading_text(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if !trimmed.starts_with('#') {
        return None;
    }
    let text = trimmed.trim_start_matches('#').trim();
    if text.is_empty() {
        None
    } else {
        Some(text.to_string())
    }
}

fn is_whats_new_heading(text: &str) -> bool {
    let normalized = normalize_apostrophe(text.trim().trim_end_matches(':'));
    normalized == "what's new" || normalized == "whats new"
}

fn is_version_line(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    lower.starts_with("version ") && lower.contains("is available")
}

fn is_changelog_trailer(line: &str) -> bool {
    let normalized = normalize_apostrophe(line);
    normalized.starts_with("full changelog")
        || normalized.starts_with("what's changed")
        || normalized == "changes"
}

fn looks_like_commit_subject(line: &str) -> bool {
    if line.ends_with('.') {
        return false;
    }
    let first = line
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim_end_matches(':')
        .to_ascii_lowercase();
    matches!(
        first.as_str(),
        "fix"
            | "feat"
            | "chore"
            | "docs"
            | "style"
            | "refactor"
            | "perf"
            | "test"
            | "build"
            | "ci"
            | "revert"
            | "add"
            | "update"
            | "set"
            | "bump"
            | "wip"
    )
}

fn clean_note_line(raw: &str) -> String {
    let line = strip_bullet(raw.trim());
    let line = strip_markdown_links(line);
    let line = line.replace("**", "").replace("__", "").replace('`', "");
    let line = strip_author_and_urls(&line);
    line.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn strip_bullet(line: &str) -> &str {
    for marker in ["- ", "* ", "• "] {
        if let Some(rest) = line.strip_prefix(marker) {
            return rest.trim();
        }
    }
    line
}

fn strip_markdown_links(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut out = String::new();
    let mut index = 0;
    while index < chars.len() {
        if let Some((text, next)) = markdown_link_at(&chars, index) {
            out.push_str(&text);
            index = next;
            continue;
        }
        out.push(chars[index]);
        index += 1;
    }
    out
}

fn markdown_link_at(chars: &[char], start: usize) -> Option<(String, usize)> {
    if chars.get(start) != Some(&'[') {
        return None;
    }
    let mut index = start + 1;
    let text_start = index;
    while index < chars.len() && chars[index] != ']' {
        index += 1;
    }
    if index >= chars.len() || chars.get(index + 1) != Some(&'(') {
        return None;
    }
    let text: String = chars[text_start..index].iter().collect();
    index += 2;
    while index < chars.len() && chars[index] != ')' {
        index += 1;
    }
    if index >= chars.len() {
        return None;
    }
    Some((text, index + 1))
}

fn strip_author_and_urls(input: &str) -> String {
    let without_urls = strip_urls(input);
    let words: Vec<&str> = without_urls.split_whitespace().collect();
    let mut kept = Vec::new();
    let mut index = 0;
    while index < words.len() {
        if words[index].eq_ignore_ascii_case("by")
            && words
                .get(index + 1)
                .is_some_and(|next| next.starts_with('@'))
        {
            index += 2;
            if words
                .get(index)
                .is_some_and(|next| next.eq_ignore_ascii_case("in"))
            {
                index += 1;
            }
            continue;
        }
        kept.push(words[index]);
        index += 1;
    }
    kept.join(" ")
}

fn strip_urls(input: &str) -> String {
    input
        .split_whitespace()
        .filter(|word| !word_has_http_url(word))
        .collect::<Vec<_>>()
        .join(" ")
}

fn word_has_http_url(word: &str) -> bool {
    let lower = word.to_ascii_lowercase();
    lower.contains("https://") || lower.contains("http://")
}

fn conventional_commit(line: &str) -> Option<(&str, &str)> {
    let colon = line.find(':')?;
    let head = line[..colon].trim();
    let desc = line[colon + 1..].trim();
    if desc.is_empty() || head.contains(' ') {
        return None;
    }
    let type_name = head.split('(').next()?.trim().trim_end_matches('!');
    if type_name.is_empty() || !type_name.chars().all(|c| c.is_ascii_alphabetic()) {
        return None;
    }
    if !is_conventional_type(type_name) {
        return None;
    }
    if let Some(open) = head.find('(') {
        let close = head.find(')')?;
        if close < open {
            return None;
        }
        let after = &head[close + 1..];
        if after != "!" && !after.is_empty() {
            return None;
        }
    } else if head.contains('!') && !head.ends_with('!') {
        return None;
    }
    Some((type_name, desc))
}

fn is_conventional_type(type_name: &str) -> bool {
    matches!(
        type_name.to_ascii_lowercase().as_str(),
        "feat"
            | "fix"
            | "chore"
            | "docs"
            | "style"
            | "refactor"
            | "perf"
            | "test"
            | "build"
            | "ci"
            | "revert"
    )
}

fn map_user_change(kind: &str, desc: &str) -> Option<String> {
    if !matches!(kind.to_ascii_lowercase().as_str(), "feat" | "fix") {
        return None;
    }
    let description = desc.trim().trim_end_matches('.').to_ascii_lowercase();
    if description.contains("set version") || description.contains("bump version") {
        return None;
    }
    if description.contains("check for updates") && description.contains("push button") {
        return Some(NOTE_CHECK_FOR_UPDATES.to_string());
    }
    if (description.contains("on-this-mac") || description.contains("on this mac"))
        && description.contains("engine")
        && (description.contains("switch") || description.contains("settle"))
    {
        return Some(NOTE_ENGINE_SWITCH.to_string());
    }
    if description.contains("permission")
        && (description.contains("once") || description.contains("prompt"))
    {
        return Some(NOTE_PERMISSIONS.to_string());
    }
    None
}

fn normalize_apostrophe(text: &str) -> String {
    text.replace('\u{2019}', "'").to_ascii_lowercase()
}

fn parse_semver(raw: &str) -> Option<SemVer> {
    let core = raw.trim().trim_start_matches('v').split('-').next()?;
    let mut parts = core.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    Some(SemVer {
        major,
        minor,
        patch,
    })
}

fn action_for(source: InstallSource, available: bool) -> (UpdateAction, Option<String>) {
    if !available {
        return (UpdateAction::None, None);
    }
    match source {
        InstallSource::Debug => (UpdateAction::Debug, None),
        InstallSource::Homebrew => (UpdateAction::BrewUpgrade, Some(BREW_UPGRADE.to_string())),
        InstallSource::Direct | InstallSource::Unknown => (UpdateAction::OpenRelease, None),
    }
}

fn resolve_check(
    release: GithubRelease,
    current: &str,
    source: InstallSource,
) -> Result<UpdateCheckDto, String> {
    let latest = parse_semver(&release.tag_name).ok_or_else(|| "parse".to_string())?;
    let current_ver = parse_semver(current).ok_or_else(|| "parse".to_string())?;
    let available = latest > current_ver;
    let release_url = if allowed_release_url(&release.html_url) {
        release.html_url
    } else {
        RELEASES_PAGE_URL.to_string()
    };
    let (action, action_command) = action_for(source, available);
    Ok(UpdateCheckDto {
        current_version: current.to_string(),
        latest_version: Some(release.tag_name.trim().trim_start_matches('v').to_string()),
        available,
        notes: if available {
            release_notes_from_markdown(&release.body)
        } else {
            Vec::new()
        },
        release_url,
        install_source: source,
        action,
        action_command,
    })
}

fn http_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(15))
        .build()
}

fn fetch_latest_release() -> Result<GithubRelease, String> {
    let response = http_agent()
        .get(RELEASES_LATEST_URL)
        .set("User-Agent", USER_AGENT)
        .set("Accept", "application/vnd.github+json")
        .call()
        .map_err(|_| "network".to_string())?;
    if response.status() < 200 || response.status() >= 300 {
        return Err("network".into());
    }
    response.into_json().map_err(|_| "parse".to_string())
}

fn forced_source() -> Option<InstallSource> {
    let value = std::env::var(FORCE_DIALOG_ENV).ok()?;
    let value = value.trim().to_ascii_lowercase();
    match value.as_str() {
        "" | "1" | "true" | "yes" | "direct" | "installable" => Some(InstallSource::Direct),
        "homebrew" | "brew" | "managed" | "package" => Some(InstallSource::Homebrew),
        "debug" => Some(InstallSource::Debug),
        "unknown" => Some(InstallSource::Unknown),
        _ => Some(InstallSource::Direct),
    }
}

fn forced_check(current: &str, detected: InstallSource) -> Result<Option<UpdateCheckDto>, String> {
    let Some(source) = forced_source() else {
        return Ok(None);
    };
    let latest = std::env::var(FORCE_VERSION_ENV)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| next_patch(current));
    let release_url = std::env::var(FORCE_URL_ENV)
        .ok()
        .filter(|value| allowed_release_url(value))
        .unwrap_or_else(|| RELEASES_PAGE_URL.to_string());
    resolve_check(
        GithubRelease {
            tag_name: latest,
            html_url: release_url,
            body: "Forced update preview.\n- Shows parsed notes for Settings.".into(),
        },
        current,
        if matches!(source, InstallSource::Direct) {
            detected
        } else {
            source
        },
    )
    .map(Some)
}

fn next_patch(current: &str) -> String {
    parse_semver(current)
        .map(|ver| format!("{}.{}.{}", ver.major, ver.minor, ver.patch + 1))
        .unwrap_or_else(|| format!("{current}.forced"))
}

pub fn version_info() -> VersionInfoDto {
    VersionInfoDto {
        version: current_version(),
        install_source: detect_install_source(),
        can_check: true,
        release_url: RELEASES_PAGE_URL.to_string(),
    }
}

pub fn check_latest() -> Result<UpdateCheckDto, String> {
    let current = current_version();
    let source = detect_install_source();
    if let Some(forced) = forced_check(&current, source)? {
        return Ok(forced);
    }
    let release = fetch_latest_release()?;
    resolve_check(release, &current, source)
}

pub fn open_allowed_release_page(url: &str) -> Result<(), String> {
    if !allowed_release_url(url) {
        return Err("blocked_url".into());
    }
    std::process::Command::new("/usr/bin/open")
        .arg(url)
        .status()
        .map_err(|err| err.to_string())
        .and_then(|status| {
            if status.success() {
                Ok(())
            } else {
                Err("open_failed".into())
            }
        })
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn app_version_info() -> VersionInfoDto {
    version_info()
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn check_for_update() -> Result<UpdateCheckDto, String> {
    check_latest()
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn open_release_page(url: String) -> Result<(), String> {
    open_allowed_release_page(&url)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(tag: &str, body: &str) -> GithubRelease {
        GithubRelease {
            tag_name: tag.into(),
            html_url: format!("https://github.com/NxT-Solutions/Bronze/releases/tag/{tag}"),
            body: body.into(),
        }
    }

    #[test]
    fn tauri_conf_version_matches_package() {
        let conf = include_str!("../tauri.conf.json");
        let needle = format!("\"version\": \"{}\"", env!("CARGO_PKG_VERSION"));
        assert!(conf.contains(&needle), "{needle}");
    }

    #[test]
    fn webview_connect_src_stays_ipc_only() {
        let conf = include_str!("../tauri.conf.json");
        assert!(conf.contains("\"connect-src\": \"ipc: http://ipc.localhost\""));
        assert!(!conf.to_ascii_lowercase().contains("api.github.com"));
    }

    #[test]
    fn setup_does_not_check_for_update() {
        let src = include_str!("lib.rs");
        let setup = src.split(".setup(|app|").nth(1).expect("setup");
        let setup = setup.split(".on_menu_event").next().expect("end");
        assert!(!setup.contains("check_for_update"));
        assert!(!setup.contains("releases/latest"));
        assert!(!setup.contains("app_version_info"));
    }

    #[test]
    fn handler_exposes_update_commands() {
        let src = include_str!("lib.rs");
        assert!(src.contains("update::app_version_info"));
        assert!(src.contains("update::check_for_update"));
        assert!(src.contains("update::open_release_page"));
        let used = include_str!("../permissions/used-permissions.toml");
        let block = used
            .split("identifier = \"allow-settings-live\"")
            .nth(1)
            .expect("settings live");
        let block = block.split("[[permission]]").next().expect("block");
        assert!(block.contains("app_version_info"));
        assert!(block.contains("check_for_update"));
        assert!(block.contains("open_release_page"));
    }

    #[test]
    fn classify_debug_target_and_assertions() {
        assert_eq!(
            classify_install_path("/Users/me/bronze-app/target/debug/bronze-desktop", false),
            InstallSource::Debug
        );
        assert_eq!(
            classify_install_path("/Applications/Bronze.app/Contents/MacOS/Bronze", true),
            InstallSource::Debug
        );
    }

    #[test]
    fn classify_homebrew_and_direct() {
        assert_eq!(
            classify_install_path(
                "/opt/homebrew/Caskroom/bronze/0.1.0/Bronze.app/Contents/MacOS/Bronze",
                false
            ),
            InstallSource::Homebrew
        );
        assert_eq!(
            classify_install_path(
                "/opt/homebrew/Cellar/bronze/0.1.0/Bronze.app/Contents/MacOS/Bronze",
                false
            ),
            InstallSource::Homebrew
        );
        assert_eq!(
            classify_install_path("/Applications/Bronze.app/Contents/MacOS/Bronze", false),
            InstallSource::Direct
        );
        assert_eq!(
            classify_install_path(
                "/Users/me/Downloads/Bronze.app/Contents/MacOS/Bronze",
                false
            ),
            InstallSource::Unknown
        );
    }

    #[test]
    fn newer_release_returns_notes_and_direct_action() {
        let info = resolve_check(
            release(
                "v1.2.0",
                "### What's new\n- The update window lists what changed.\n\n**Full Changelog**: https://example.com",
            ),
            "1.1.0",
            InstallSource::Direct,
        )
        .expect("resolve");
        assert!(info.available);
        assert_eq!(info.latest_version.as_deref(), Some("1.2.0"));
        assert_eq!(info.action, UpdateAction::OpenRelease);
        assert_eq!(
            info.notes,
            vec!["The update window lists what changed.".to_string()]
        );
        assert!(allowed_release_url(&info.release_url));
    }

    #[test]
    fn conventional_commit_becomes_a_plain_bullet() {
        let notes = release_notes_from_markdown(
            "* fix(desktop): style Check for updates as a push button by @NoahNxT in https://github.com/NxT-Solutions/Bronze/pull/44",
        );
        assert_eq!(notes, vec![NOTE_CHECK_FOR_UPDATES.to_string()]);
    }

    #[test]
    fn published_012_notes_drop_author_url_and_jargon() {
        let body = [
            "## What's Changed",
            "* fix(desktop): style Check for updates as a push button by @NoahNxT in https://github.com/NxT-Solutions/Bronze/pull/44",
            "* fix(release): bleed the Dock mark to the icon canvas by @NoahNxT in https://github.com/NxT-Solutions/Bronze/pull/45",
            "* fix(title): settle on-this-Mac engine switches by @NoahNxT in https://github.com/NxT-Solutions/Bronze/pull/46",
            "* feat(desktop): prompt used macOS permissions once by @NoahNxT in https://github.com/NxT-Solutions/Bronze/pull/49",
            "* chore(release): set version 0.1.2 by @NoahNxT in https://github.com/NxT-Solutions/Bronze/pull/47",
            "* fix(storage): retune sqlite wal checkpoint pragma by @NoahNxT in https://github.com/NxT-Solutions/Bronze/pull/99",
            "",
            "**Full Changelog**: https://github.com/NxT-Solutions/Bronze/compare/v0.1.1...v0.1.2",
        ]
        .join("\n");
        let notes = release_notes_from_markdown(&body);
        assert_eq!(
            notes,
            vec![
                NOTE_CHECK_FOR_UPDATES.to_string(),
                NOTE_ENGINE_SWITCH.to_string(),
                NOTE_PERMISSIONS.to_string(),
            ]
        );
        for note in &notes {
            assert!(!note.contains("http"), "{note}");
            assert!(!note.contains('@'), "{note}");
            assert!(!note.contains("fix("), "{note}");
            assert!(!note.to_ascii_lowercase().contains("dock"), "{note}");
            assert!(!note.to_ascii_lowercase().contains("tile"), "{note}");
            assert!(
                note.split_whitespace().count() >= 2,
                "wrap on words: {note}"
            );
        }
    }

    #[test]
    fn whats_new_section_wins_over_commit_subjects() {
        let notes = release_notes_from_markdown(
            "### What's new\n\n- Search is faster.\n- Check for updates is easier to see.\n\n## What's Changed\n* fix(release): bleed the Dock mark to the icon canvas by @NoahNxT in https://github.com/NxT-Solutions/Bronze/pull/45\n",
        );
        assert_eq!(
            notes,
            vec![
                "Search is faster.".to_string(),
                NOTE_CHECK_FOR_UPDATES.to_string(),
            ]
        );
    }

    #[test]
    fn version_line_stays_out_of_the_notes() {
        let notes = release_notes_from_markdown(
            "Version 0.1.2 is available.\n### What's new\n- Check for updates is easier to see.\n",
        );
        assert_eq!(notes, vec![NOTE_CHECK_FOR_UPDATES.to_string()]);
        assert!(notes.iter().all(|note| !note.contains("Version")));
    }

    #[test]
    fn plain_notes_cap_and_do_not_need_a_url_to_wrap() {
        let body = (1..=6)
            .map(|index| format!("- Sentence number {index} stays short."))
            .collect::<Vec<_>>()
            .join("\n");
        let markdown = format!("### What's new\n{body}\n");
        let notes = release_notes_from_markdown(&markdown);
        assert_eq!(notes.len(), MAX_UPDATE_NOTES);
        assert!(notes.iter().all(|note| !note.contains("://")));
        assert!(notes.iter().all(|note| note.contains(' ')));
    }

    #[test]
    fn current_release_is_not_available() {
        let info = resolve_check(release("v0.1.0", "- notes"), "0.1.0", InstallSource::Direct)
            .expect("resolve");
        assert!(!info.available);
        assert_eq!(info.action, UpdateAction::None);
        assert!(info.notes.is_empty());
    }

    #[test]
    fn homebrew_update_returns_brew_command() {
        let info = resolve_check(
            release("v0.2.0", "- brew"),
            "0.1.0",
            InstallSource::Homebrew,
        )
        .expect("resolve");
        assert_eq!(info.action, UpdateAction::BrewUpgrade);
        assert_eq!(info.action_command.as_deref(), Some(BREW_UPGRADE));
    }

    #[test]
    fn debug_update_does_not_offer_install() {
        let info = resolve_check(release("v9.0.0", "- debug"), "0.1.0", InstallSource::Debug)
            .expect("resolve");
        assert!(info.available);
        assert_eq!(info.action, UpdateAction::Debug);
        assert!(info.action_command.is_none());
    }

    #[test]
    fn allowed_release_urls_are_bronze_github_only() {
        assert!(allowed_release_url(RELEASES_PAGE_URL));
        assert!(allowed_release_url(
            "https://github.com/NxT-Solutions/Bronze/releases/tag/v0.1.0"
        ));
        assert!(!allowed_release_url(
            "https://github.com/NxT-Solutions/Bronze/releases/../evil"
        ));
        assert!(!allowed_release_url(
            "https://github.com/evil/Bronze/releases/latest"
        ));
        assert!(!allowed_release_url("https://evil.example/releases"));
        assert!(!allowed_release_url("javascript:alert(1)"));
        assert_eq!(
            open_allowed_release_page("https://example.com").unwrap_err(),
            "blocked_url"
        );
    }

    #[test]
    fn fetch_uses_github_latest_and_no_proxy_env() {
        let src = include_str!("update.rs");
        let prod = src.split("#[cfg(test)]").next().expect("prod");
        assert!(prod.contains(RELEASES_LATEST_URL));
        assert!(prod.contains("User-Agent"));
        assert!(prod.contains("AgentBuilder::new()"));
        assert!(!prod.contains("proxy_from_env"));
        assert!(!prod.contains("HTTP_PROXY"));
        assert!(!prod.contains("ALL_PROXY"));
        assert!(!prod.contains("plugin-updater"));
        assert!(!prod.contains("Sparkle"));
    }

    #[test]
    fn version_info_is_local() {
        let info = version_info();
        assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
        assert!(info.can_check);
        assert_eq!(info.release_url, RELEASES_PAGE_URL);
    }
}
