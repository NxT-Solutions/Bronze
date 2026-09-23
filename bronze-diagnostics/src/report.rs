use crate::bundle::{BundleError, AUTOMATIC_UPLOAD, HUMAN_GATES};
use std::fmt::Write as _;

pub const LAST_HOUR_MS: u64 = 3_600_000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupportNamedState {
    pub name: String,
    pub state: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupportQueueCounts {
    pub queued: u32,
    pub copied: u32,
    pub active: u32,
    pub done: u32,
    pub skipped: u32,
    pub trashed: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupportSourceApp {
    pub bundle_id: String,
    pub app_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupportEventLine {
    pub occurred_at_ms: u64,
    pub request_id: String,
    pub stage: String,
    pub result: String,
    pub duration_ms: Option<u32>,
    pub trigger_kind: String,
    pub source_bundle_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupportReport {
    pub generated_at_ms: u64,
    pub app_version: String,
    pub toolchain: String,
    pub rust_min: String,
    pub tauri_version: String,
    pub rusqlite_version: String,
    pub settings_schema: String,
    pub store_schema: String,
    pub os: String,
    pub os_version: String,
    pub arch: String,
    pub model: String,
    pub data_path: String,
    pub login_item_setting: String,
    pub login_item_status: String,
    pub title_engine_tier: String,
    pub title_engine_phase: String,
    pub permissions: Vec<SupportNamedState>,
    pub queue: SupportQueueCounts,
    pub excluded_bundle_ids: Vec<String>,
    pub recent_sources: Vec<SupportSourceApp>,
    pub events: Vec<SupportEventLine>,
    pub last_hour_ms: u64,
}

pub fn contains_forbidden_payload(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("secret")
        || lower.contains("password")
        || lower.contains("token")
        || lower.contains("hunter2")
        || lower.contains("http")
        || has_unredacted_home(text)
}

pub fn has_unredacted_home(text: &str) -> bool {
    let mut rest = text;
    while let Some(idx) = rest.find("/Users/") {
        let after = &rest[idx + 7..];
        if let Some(stripped) = after.strip_prefix("[redacted]") {
            rest = stripped;
            continue;
        }
        let name: String = after
            .chars()
            .take_while(|c| *c != '/' && !c.is_whitespace())
            .collect();
        if !name.is_empty() {
            return true;
        }
        rest = after;
    }
    false
}

pub fn redact_home_path(path: &str) -> String {
    let mut out = path.to_string();
    if let Ok(home) = std::env::var("HOME") {
        if home.starts_with("/Users/") && home.len() > "/Users/".len() && out.contains(&home) {
            out = out.replace(&home, "/Users/[redacted]");
        }
    }
    let mut rebuilt = String::with_capacity(out.len());
    let mut rest = out.as_str();
    while let Some(idx) = rest.find("/Users/") {
        rebuilt.push_str(&rest[..idx]);
        rebuilt.push_str("/Users/");
        let after = &rest[idx + 7..];
        if let Some(stripped) = after.strip_prefix("[redacted]") {
            rebuilt.push_str("[redacted]");
            rest = stripped;
            continue;
        }
        if let Some(slash) = after.find('/') {
            rebuilt.push_str("[redacted]");
            rest = &after[slash..];
            continue;
        }
        let name_len = after
            .find(|c: char| c.is_whitespace())
            .unwrap_or(after.len());
        if name_len > 0 {
            rebuilt.push_str("[redacted]");
            rest = &after[name_len..];
        } else {
            rest = after;
        }
    }
    rebuilt.push_str(rest);
    rebuilt
}

pub fn render_support_report(report: &SupportReport) -> Result<String, BundleError> {
    let body = format_support_report(report);
    if contains_forbidden_payload(&body) {
        return Err(BundleError::SecretDetected);
    }
    Ok(body)
}

fn format_support_report(report: &SupportReport) -> String {
    let mut out = String::new();
    push(&mut out, "Bronze support report");
    push(
        &mut out,
        &format!("generated_at_ms={}", report.generated_at_ms),
    );
    push(&mut out, &format!("automatic_upload={}", AUTOMATIC_UPLOAD));
    push(&mut out, &format!("human_gates={}", HUMAN_GATES.join(", ")));
    push(&mut out, "");
    push(&mut out, "App");
    push(&mut out, &format!("  version={}", report.app_version));
    push(&mut out, &format!("  toolchain={}", report.toolchain));
    push(&mut out, &format!("  rust_min={}", report.rust_min));
    push(&mut out, &format!("  tauri={}", report.tauri_version));
    push(&mut out, &format!("  rusqlite={}", report.rusqlite_version));
    push(
        &mut out,
        &format!("  settings_schema={}", report.settings_schema),
    );
    push(&mut out, &format!("  schema={}", report.store_schema));
    push(&mut out, "");
    push(&mut out, "Machine");
    push(&mut out, &format!("  os={}", report.os));
    push(&mut out, &format!("  os_version={}", report.os_version));
    push(&mut out, &format!("  arch={}", report.arch));
    push(&mut out, &format!("  model={}", report.model));
    push(&mut out, "");
    push(&mut out, "Data");
    push(&mut out, &format!("  path={}", report.data_path));
    push(&mut out, "");
    push(&mut out, "Permissions");
    for row in &report.permissions {
        if contains_forbidden_payload(&row.name) || contains_forbidden_payload(&row.state) {
            continue;
        }
        push(&mut out, &format!("  {}={}", row.name, row.state));
    }
    push(
        &mut out,
        &format!("  login_item_setting={}", report.login_item_setting),
    );
    push(
        &mut out,
        &format!("  login_item_status={}", report.login_item_status),
    );
    push(&mut out, "");
    push(&mut out, "Title engine");
    push(&mut out, &format!("  tier={}", report.title_engine_tier));
    push(&mut out, &format!("  phase={}", report.title_engine_phase));
    push(&mut out, "");
    push(&mut out, "Queue");
    push(
        &mut out,
        &format!(
            "  queued={} copied={} active={} done={} skipped={} trashed={}",
            report.queue.queued,
            report.queue.copied,
            report.queue.active,
            report.queue.done,
            report.queue.skipped,
            report.queue.trashed
        ),
    );
    push(&mut out, "");
    push(&mut out, "Excluded apps");
    write_list(
        &mut out,
        report
            .excluded_bundle_ids
            .iter()
            .filter(|id| !contains_forbidden_payload(id))
            .map(String::as_str),
    );
    push(&mut out, "");
    push(&mut out, "Recent source apps");
    push(&mut out, &format!("  window={} ms", report.last_hour_ms));
    if report.recent_sources.is_empty() {
        push(&mut out, "  (none)");
    } else {
        for source in &report.recent_sources {
            let line = format!(
                "  bundle={} app={}",
                empty_as_unavailable(&source.bundle_id),
                empty_as_unavailable(&source.app_name)
            );
            if contains_forbidden_payload(&line) {
                continue;
            }
            push(&mut out, &line);
        }
    }
    push(&mut out, "");
    push(&mut out, "Last-hour diagnostic events");
    push(&mut out, &format!("  window={} ms", report.last_hour_ms));
    write_events(&mut out, report.events.iter());
    push(&mut out, "");
    push(&mut out, "Last-hour copy attempts");
    push(&mut out, &format!("  window={} ms", report.last_hour_ms));
    write_events(
        &mut out,
        report
            .events
            .iter()
            .filter(|event| event.stage == "clipboard"),
    );
    out
}

fn write_events<'a, I>(out: &mut String, events: I)
where
    I: IntoIterator<Item = &'a SupportEventLine>,
{
    let mut any = false;
    for event in events {
        let dur = event
            .duration_ms
            .map(|ms| ms.to_string())
            .unwrap_or_else(|| "unavailable".into());
        let source = event
            .source_bundle_id
            .as_deref()
            .map(empty_as_unavailable)
            .unwrap_or("unavailable");
        let line = format!(
            "  t={} req={} stage={} result={} dur_ms={} trigger={} source={}",
            event.occurred_at_ms,
            event.request_id,
            event.stage,
            event.result,
            dur,
            event.trigger_kind,
            source
        );
        if contains_forbidden_payload(&line) {
            continue;
        }
        push(out, &line);
        any = true;
    }
    if !any {
        push(out, "  (none)");
    }
}

fn write_list<'a, I>(out: &mut String, items: I)
where
    I: IntoIterator<Item = &'a str>,
{
    let mut any = false;
    for item in items {
        push(out, &format!("  {item}"));
        any = true;
    }
    if !any {
        push(out, "  (none)");
    }
}

fn empty_as_unavailable(value: &str) -> &str {
    if value.is_empty() {
        "unavailable"
    } else {
        value
    }
}

fn push(out: &mut String, line: &str) {
    let _ = writeln!(out, "{line}");
}

#[cfg(test)]
fn sample_report() -> SupportReport {
    SupportReport {
        generated_at_ms: 1_700_000_000_000,
        app_version: "0.1.0".into(),
        toolchain: "1.98.1".into(),
        rust_min: "1.77.2".into(),
        tauri_version: "2.11.3".into(),
        rusqlite_version: "0.32".into(),
        settings_schema: "1".into(),
        store_schema: "2".into(),
        os: "macos".into(),
        os_version: "26.5.0".into(),
        arch: "aarch64".into(),
        model: "Mac16,6".into(),
        data_path: "/Users/[redacted]/Library/Application Support/Bronze".into(),
        login_item_setting: "off".into(),
        login_item_status: "unavailable".into(),
        title_engine_tier: "extractive".into(),
        title_engine_phase: "idle".into(),
        permissions: vec![
            SupportNamedState {
                name: "input_monitoring".into(),
                state: "unavailable".into(),
            },
            SupportNamedState {
                name: "accessibility".into(),
                state: "unavailable".into(),
            },
            SupportNamedState {
                name: "capture_pipeline_self_test".into(),
                state: "unknown".into(),
            },
        ],
        queue: SupportQueueCounts {
            queued: 1,
            copied: 0,
            active: 0,
            done: 0,
            skipped: 0,
            trashed: 0,
        },
        excluded_bundle_ids: vec!["com.example.excluded".into()],
        recent_sources: vec![SupportSourceApp {
            bundle_id: "com.apple.TextEdit".into(),
            app_name: "TextEdit".into(),
        }],
        events: vec![SupportEventLine {
            occurred_at_ms: 1_700_000_000_100,
            request_id: "r1".into(),
            stage: "store".into(),
            result: "ok".into(),
            duration_ms: Some(4),
            trigger_kind: "menu".into(),
            source_bundle_id: Some("com.apple.TextEdit".into()),
        }],
        last_hour_ms: LAST_HOUR_MS,
    }
}

#[cfg(test)]
mod report_tests {
    use super::*;

    #[test]
    fn fixture_report_has_debug_headings_and_matching_preview() {
        assert!(!AUTOMATIC_UPLOAD);
        assert_eq!(LAST_HOUR_MS, 3_600_000);
        let report = sample_report();
        let preview = render_support_report(&report).expect("preview");
        let export = render_support_report(&report).expect("export");
        assert_eq!(preview, export);
        assert!(!preview.contains("events=0"));
        for needle in [
            "version=",
            "os=",
            "arch=",
            "schema=",
            "/Users/[redacted]",
            "input_monitoring=",
            "human_gates=3.9, 3.10, 5.5, 9.3",
            "Last-hour diagnostic events",
            "Last-hour copy attempts",
            "3.9",
        ] {
            assert!(preview.contains(needle), "missing {needle}");
        }
        assert!(!preview.contains("hunter2"));
        assert!(!preview.contains("http"));
        assert!(!has_unredacted_home(&preview));
    }

    #[test]
    fn secret_scan_rejects_seeded_needles_and_home_paths() {
        const SECRET: &str = "hunter2-s3cret-payload";
        let mut report = sample_report();
        report.app_version = SECRET.into();
        assert_eq!(
            render_support_report(&report),
            Err(BundleError::SecretDetected)
        );
        report = sample_report();
        report.data_path = "/Users/alex/Library/Application Support/Bronze".into();
        assert_eq!(
            render_support_report(&report),
            Err(BundleError::SecretDetected)
        );
        assert_eq!(
            redact_home_path("/Users/alex/Library/Application Support/Bronze"),
            "/Users/[redacted]/Library/Application Support/Bronze"
        );
        assert!(contains_forbidden_payload("http"));
        assert!(contains_forbidden_payload(SECRET));
        assert!(!contains_forbidden_payload("com.apple.TextEdit"));
    }
}
