//! Identity of the process Permission health is checking.
//!
//! macOS Accessibility stores an ad-hoc cdhash. A System Settings row for the
//! same bundle id can belong to a different copy than the one that is running.

use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunningBundle {
    pub name: String,
    pub version: String,
    pub bundle_path: String,
    pub cdhash: String,
    pub signature_kind: String,
}

pub fn current_running_bundle() -> RunningBundle {
    let exe = std::env::current_exe().unwrap_or_default();
    identity_from_exe(&exe, env!("CARGO_PKG_VERSION"))
}

pub fn identity_from_exe(exe: &Path, fallback_version: &str) -> RunningBundle {
    if let Some(app) = app_bundle_root(exe) {
        let plist = read_plist_text(&app.join("Contents").join("Info.plist"));
        let name = plist
            .as_deref()
            .and_then(|xml| plist_string(xml, "CFBundleDisplayName"))
            .or_else(|| {
                plist
                    .as_deref()
                    .and_then(|xml| plist_string(xml, "CFBundleName"))
            })
            .unwrap_or_else(|| file_stem_string(&app));
        let version = plist
            .as_deref()
            .and_then(|xml| plist_string(xml, "CFBundleShortVersionString"))
            .unwrap_or_else(|| fallback_version.to_string());
        let (cdhash, signature_kind) = code_identity(&app);
        return RunningBundle {
            name,
            version,
            bundle_path: app.to_string_lossy().into_owned(),
            cdhash,
            signature_kind,
        };
    }
    let (cdhash, signature_kind) = code_identity(exe);
    RunningBundle {
        name: file_stem_string(exe),
        version: fallback_version.to_string(),
        bundle_path: exe.to_string_lossy().into_owned(),
        cdhash,
        signature_kind,
    }
}

fn code_identity(path: &Path) -> (String, String) {
    if !path.exists() {
        return (String::new(), String::new());
    }
    let Ok(output) = Command::new("/usr/bin/codesign")
        .args(["-d", "--verbose=4"])
        .arg(path)
        .output()
    else {
        return (String::new(), String::new());
    };
    // codesign writes identity lines to stderr.
    let text = String::from_utf8_lossy(&output.stderr);
    let cdhash = text
        .lines()
        .find_map(|line| line.strip_prefix("CDHash="))
        .unwrap_or("")
        .trim()
        .to_string();
    let signature_kind = if text.contains("Signature=adhoc") {
        "adhoc"
    } else if text.contains("Authority=") {
        "signed"
    } else {
        "unknown"
    };
    (cdhash, signature_kind.to_string())
}

fn app_bundle_root(exe: &Path) -> Option<PathBuf> {
    let macos = exe.parent()?;
    if macos.file_name()? != "MacOS" {
        return None;
    }
    let contents = macos.parent()?;
    if contents.file_name()? != "Contents" {
        return None;
    }
    let app = contents.parent()?;
    if app.extension()? != "app" {
        return None;
    }
    Some(app.to_path_buf())
}

fn file_stem_string(path: &Path) -> String {
    path.file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .filter(|stem| !stem.is_empty())
        .unwrap_or_else(|| "Bronze".to_string())
}

fn read_plist_text(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    if bytes.starts_with(b"bplist") {
        return None;
    }
    String::from_utf8(bytes).ok()
}

fn plist_string(xml: &str, key: &str) -> Option<String> {
    let needle = format!("<key>{key}</key>");
    let rest = xml.get(xml.find(&needle)? + needle.len()..)?;
    let start = rest.find("<string>")? + "<string>".len();
    let end = rest.get(start..)?.find("</string>")?;
    let text = decode_plist_text(rest[start..start + end].trim());
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn decode_plist_text(raw: &str) -> String {
    raw.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

#[cfg(test)]
mod tests {
    use super::identity_from_exe;
    use std::fs;

    #[test]
    fn bundled_exe_names_the_app_copy_and_its_version() {
        let root =
            std::env::temp_dir().join(format!("bronze-bundle-identity-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let macos = root.join("Bronze.app/Contents/MacOS");
        fs::create_dir_all(&macos).expect("bundle dirs");
        let plist = r#"<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0"><dict>
<key>CFBundleDisplayName</key>
<string>Bronze</string>
<key>CFBundleShortVersionString</key>
<string>0.1.1</string>
</dict></plist>"#;
        fs::write(root.join("Bronze.app/Contents/Info.plist"), plist).expect("plist");
        let exe = macos.join("bronze-desktop");
        fs::write(&exe, b"").expect("exe");
        let identity = identity_from_exe(&exe, "9.9.9");
        assert_eq!(identity.name, "Bronze");
        assert_eq!(identity.version, "0.1.1");
        assert!(identity.bundle_path.ends_with("Bronze.app"));
        assert!(!identity.bundle_path.contains("Contents/MacOS"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn raw_binary_reports_its_own_path() {
        let path = std::env::temp_dir().join(format!("bronze-desktop-{}", std::process::id()));
        fs::write(&path, b"").expect("exe");
        let identity = identity_from_exe(&path, "0.1.1");
        assert!(identity.name.starts_with("bronze-desktop"));
        assert_eq!(identity.version, "0.1.1");
        assert_eq!(identity.bundle_path, path.to_string_lossy());
        assert!(identity.signature_kind == "adhoc" || identity.signature_kind == "unknown");
        let _ = fs::remove_file(&path);
    }
}
