//! Local debug packaging contract (story 9.1, SEC-005).
//! ADR-002 Accepted: split arm64 and x86_64 packages (not a universal2 blob).

#[cfg(target_arch = "aarch64")]
pub const PACKAGE_ARCH: &str = "arm64";
#[cfg(target_arch = "x86_64")]
pub const PACKAGE_ARCH: &str = "x86_64";
#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
pub const PACKAGE_ARCH: &str = "unknown";

pub const DG01_INTEL_SUPPORT: bool = true;
pub const GET_TASK_ALLOW_FORBIDDEN: bool = true;
pub const PACKAGE_SCRIPT: &str = "tooling/package-debug.sh";
pub const RELEASE_ARCHES: &[&str] = &["arm64", "x86_64"];

pub fn recorded_arch() -> &'static str {
    PACKAGE_ARCH
}

pub fn supported_release_arches() -> &'static [&'static str] {
    RELEASE_ARCHES
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
fn icns_png_sizes(bytes: &[u8]) -> Vec<(u32, u32)> {
    if bytes.len() < 8 || &bytes[..4] != b"icns" {
        return Vec::new();
    }
    let mut sizes = Vec::new();
    let mut off = 8usize;
    while off + 8 <= bytes.len() {
        let len = u32::from_be_bytes(bytes[off + 4..off + 8].try_into().unwrap()) as usize;
        if len < 8 || off + len > bytes.len() {
            break;
        }
        let payload = &bytes[off + 8..off + len];
        if payload.len() >= 24
            && payload.starts_with(&[0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n'])
        {
            let width = u32::from_be_bytes(payload[16..20].try_into().unwrap());
            let height = u32::from_be_bytes(payload[20..24].try_into().unwrap());
            sizes.push((width, height));
        }
        off += len;
    }
    sizes
}

#[cfg(test)]
mod packaging_tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn records_split_arches_and_forbids_get_task_allow() {
        assert!(
            recorded_arch() == "arm64"
                || recorded_arch() == "x86_64"
                || recorded_arch() == "unknown"
        );
        assert!(DG01_INTEL_SUPPORT);
        assert_eq!(supported_release_arches(), ["arm64", "x86_64"]);
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
        assert!(info.contains("NSAccessibilityUsageDescription"));
        assert!(info.contains("NSInputMonitoringUsageDescription"));
        assert!(info.contains("Bronze reads the current selection so a capture can use it."));
        assert!(info
            .contains("Bronze watches the capture chord so a double-tap can add the selection."));
        assert!(!info.contains("NSScreenCaptureUsageDescription"));
        assert!(conf.contains("\"infoPlist\": \"Info.plist\""));
        assert!(manifest.join("icons/icon.icns").is_file());
        let icns = fs::read(manifest.join("icons/icon.icns")).unwrap();
        let sizes = icns_png_sizes(&icns);
        assert!(
            sizes.contains(&(1024, 1024)),
            "icon.icns must include the 1024px image, found {sizes:?}"
        );
        let checker = manifest.join("../../../tooling/dock-icon-fill.py");
        let fill = std::process::Command::new("python3")
            .arg(&checker)
            .arg(manifest.join("icons/icon.icns"))
            .arg(manifest.join("icons/icon.png"))
            .arg(manifest.join("icons/128x128.png"))
            .arg(manifest.join("icons/128x128@2x.png"))
            .status()
            .expect("dock icon fill check");
        assert!(
            fill.success(),
            "Dock icon artwork must cover the canvas edge"
        );
        let icns_at = conf.find("\"icons/icon.icns\"").unwrap();
        let template_at = conf.find("\"icons/32x32.png\"").unwrap();
        assert!(
            icns_at < template_at,
            "bundle.icon must list icon.icns before the menu-bar template"
        );
        let build = fs::read_to_string(manifest.join("build.rs")).unwrap();
        assert!(build.contains("wrap_notice_helper"));
        assert!(build.contains("BronzeNotice.app"));
        assert!(build.contains("AppIcon.icns"));
        assert!(build.contains("AppIcon.icns must match icon.icns"));
        assert!(build.contains("dock-icon-fill.py"));
        let release =
            fs::read_to_string(manifest.join("../../../.github/workflows/release.yml")).unwrap();
        assert!(release.contains("differs from icon.icns"));
        assert!(release.contains("BronzeNotice AppIcon.icns was not produced"));
        assert!(build.contains("NSUserNotificationsUsageDescription"));
        assert!(build.contains("NSUserNotificationAlertStyle"));
        assert!(build.contains("NSPrincipalClass"));
        assert!(build.contains("/usr/bin/codesign"));
        assert!(build.contains("app.bronze.desktop.notice"));
        assert!(build.contains("Bronze Notice"));
        assert!(build.contains("--keychain"));
        assert!(build.contains("bronze-notice-signing.keychain"));
        assert!(build.contains("--arch"));
        assert!(build.contains("swift_target_triple") || build.contains("apple-macosx14.0"));
        assert!(!build.contains("externalBin"));
        assert!(conf.contains("\"resources\""));
        assert!(conf.contains("bronze-title-model/vendor/*.gguf"));
        assert!(conf.contains("\"models/\""));
        assert!(conf.contains("\"../../../packages/i18n/locales\": \"locales/\""));
        assert!(!build.contains("stage_bundled_title_models"));
        assert!(!build.contains("same_len"));
        assert!(!build.contains("manifest.join(\"models\")"));
    }

    #[test]
    fn sbom_stub_sorts_dedups_and_ignores_non_names() {
        let names = sbom_stub(&[
            "name = \"zeta\"",
            "version = \"1.0.0\"",
            "name = \"alpha\"",
            "name = \"alpha\"",
            "not a crate line",
            "name = \"beta\"",
        ]);
        assert_eq!(
            names,
            vec!["alpha".to_string(), "beta".to_string(), "zeta".to_string()]
        );
    }

    #[test]
    fn get_task_allow_in_text_fails_the_packaging_check() {
        assert!(!forbids_get_task_allow("com.apple.security.get-task-allow"));
        assert!(forbids_get_task_allow(
            "<key>com.apple.security.app-sandbox</key>"
        ));
        assert!(icns_png_sizes(b"not-an-icns").is_empty());
        assert!(icns_png_sizes(b"icns\x00\x00\x00\xff").is_empty());
    }
}
