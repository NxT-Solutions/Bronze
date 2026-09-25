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

pub fn release_notes_from_markdown(markdown: &str) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for raw_line in markdown.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            if !matches!(lines.last(), Some(last) if last.is_empty()) {
                lines.push(String::new());
            }
            continue;
        }
        let mut line = trimmed
            .trim_start_matches('#')
            .trim()
            .replace("**", "")
            .replace("__", "")
            .replace('`', "");
        if line.eq_ignore_ascii_case("What's Changed") {
            continue;
        }
        if let Some(stripped) = line.strip_prefix("* ").or_else(|| line.strip_prefix("- ")) {
            line = format!("• {}", stripped.trim());
        }
        lines.push(line);
    }
    while matches!(lines.last(), Some(last) if last.is_empty()) {
        lines.pop();
    }
    lines
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
                "## What's Changed\n* Fix updater popup\n\n**Full Changelog**: https://example.com",
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
            vec![
                "• Fix updater popup".to_string(),
                String::new(),
                "Full Changelog: https://example.com".to_string()
            ]
        );
        assert!(allowed_release_url(&info.release_url));
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
