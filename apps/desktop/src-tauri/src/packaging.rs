//! Local debug packaging contract (story 9.1, SEC-005).
//! ADR-002 / DG-01 stay Proposed: record arm64 unless DG-01 selects Intel.

pub const PACKAGE_ARCH: &str = "arm64";
pub const DG01_INTEL_SUPPORT: bool = false;
pub const GET_TASK_ALLOW_FORBIDDEN: bool = true;
pub const PACKAGE_SCRIPT: &str = "tooling/package-debug.sh";

pub fn recorded_arch() -> &'static str {
    if DG01_INTEL_SUPPORT {
        "universal2"
    } else {
        PACKAGE_ARCH
    }
}

pub fn sbom_stub(lines: &[&str]) -> Vec<String> {
    let mut names: Vec<String> = lines
        .iter()
        .filter_map(|line| {
            line.strip_prefix("name = \"")
                .and_then(|rest| rest.strip_suffix('"'))
                .map(str::to_string)
        })
        .collect();
    names.sort();
    names.dedup();
    names
}

pub fn forbids_get_task_allow(text: &str) -> bool {
    !text.contains("get-task-allow")
}

#[cfg(test)]
mod packaging_tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn records_arm64_until_dg01_and_forbids_get_task_allow() {
        assert_eq!(recorded_arch(), "arm64");
        assert!(!DG01_INTEL_SUPPORT);
        assert!(GET_TASK_ALLOW_FORBIDDEN);
        assert_eq!(PACKAGE_SCRIPT, "tooling/package-debug.sh");
        let lock = include_str!("../../../../Cargo.lock");
        let sbom = sbom_stub(
            &lock
                .lines()
                .filter(|line| line.starts_with("name = \""))
                .collect::<Vec<_>>(),
        );
        assert!(!sbom.is_empty());
        assert!(sbom
            .iter()
            .any(|n| n == "bronze-desktop" || n == "bronze-domain"));
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let conf = fs::read_to_string(manifest.join("tauri.conf.json")).unwrap();
        assert!(forbids_get_task_allow(&conf));
        let entitlements =
            fs::read_to_string(manifest.join("entitlements/macos.release.plist")).unwrap();
        assert!(forbids_get_task_allow(&entitlements));
        assert!(entitlements.contains("app-sandbox"));
        assert!(entitlements.contains("<false/>"));
        assert!(!entitlements.contains("allow-jit"));
        assert!(!entitlements.contains("allow-unsigned-executable-memory"));
        let info = fs::read_to_string(manifest.join("Info.plist")).unwrap();
        assert!(info.contains("NSUserNotificationsUsageDescription"));
        assert!(info.contains("local banner"));
        assert!(manifest.join("icons/icon.icns").is_file());
        let build = fs::read_to_string(manifest.join("build.rs")).unwrap();
        assert!(build.contains("wrap_notice_helper"));
        assert!(build.contains("BronzeNotice.app"));
        assert!(build.contains("AppIcon.icns"));
        assert!(build.contains("NSUserNotificationsUsageDescription"));
        assert!(build.contains("NSUserNotificationAlertStyle"));
        assert!(build.contains("NSPrincipalClass"));
        assert!(build.contains("/usr/bin/codesign"));
        assert!(build.contains("app.bronze.desktop.notice"));
        assert!(build.contains("Bronze Notice"));
        assert!(build.contains("bronze-notice-signing.keychain"));
        assert!(!build.contains("externalBin"));
        assert!(conf.contains("\"resources\""));
        assert!(conf.contains("models/*"));
        assert!(build.contains("stage_bundled_title_models"));
        assert!(build.contains("same_len"));
        assert!(build.contains("qwen2.5-0.5b-instruct-q4_k_m.gguf"));
        assert!(build.contains("SmolLM2-135M-Instruct-Q4_K_M.gguf"));
        assert!(build.contains("SmolLM2-360M-Instruct-Q4_K_M.gguf"));
    }
}
