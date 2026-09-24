use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rustc-check-cfg=cfg(bronze_native_linked)");
    stage_bundled_title_models();
    link_bronze_native();
    tauri_build::build();
}

fn stage_bundled_title_models() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let vendor = manifest.join("../../../bronze-title-model/vendor");
    let dest = manifest.join("models");
    let files = [
        "SmolLM2-135M-Instruct-Q4_K_M.gguf",
        "SmolLM2-360M-Instruct-Q4_K_M.gguf",
        "qwen2.5-0.5b-instruct-q4_k_m.gguf",
    ];
    let _ = std::fs::create_dir_all(&dest);
    println!("cargo:rerun-if-changed={}", vendor.display());
    for name in files {
        let src = vendor.join(name);
        println!("cargo:rerun-if-changed={}", src.display());
        if src.is_file() {
            let _ = std::fs::copy(&src, dest.join(name));
        }
    }
}

fn link_bronze_native() {
    if env::var("CARGO_CFG_TARGET_OS").ok().as_deref() != Some("macos") {
        return;
    }

    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let package = manifest.join("../../../native/macos/BronzeNative");
    let profile = match env::var("PROFILE").as_deref() {
        Ok("release") => "release",
        _ => "debug",
    };

    let status = Command::new("swift")
        .args([
            "build",
            "--package-path",
            package.to_str().unwrap(),
            "-c",
            profile,
        ])
        .status()
        .expect("swift build");
    assert!(
        status.success(),
        "swift build --package-path native/macos/BronzeNative failed"
    );

    let bin = Command::new("swift")
        .args([
            "build",
            "--package-path",
            package.to_str().unwrap(),
            "-c",
            profile,
            "--show-bin-path",
        ])
        .output()
        .expect("swift --show-bin-path");
    assert!(bin.status.success(), "swift --show-bin-path failed");
    let bin_path = String::from_utf8(bin.stdout).unwrap();
    let bin_path = bin_path.trim();

    println!(
        "cargo:rerun-if-changed={}",
        package.join("Package.swift").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        package.join("Sources").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        manifest.join("icons/icon.icns").display()
    );
    println!("cargo:rustc-link-search=native={bin_path}");
    println!("cargo:rustc-link-lib=static=BronzeNative");

    for dir in swift_runtime_dirs() {
        println!("cargo:rustc-link-search=native={}", dir.display());
    }
    println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");
    println!("cargo:rustc-link-lib=dylib=swiftCore");
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-lib=framework=AppKit");
    println!("cargo:rustc-link-lib=framework=UserNotifications");
    println!("cargo:rustc-link-lib=framework=ServiceManagement");
    println!("cargo:rustc-cfg=bronze_native_linked");

    wrap_notice_helper(
        Path::new(bin_path),
        &profile_dir(),
        &manifest.join("icons/icon.icns"),
    );
}

fn profile_dir() -> PathBuf {
    PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"))
        .ancestors()
        .nth(3)
        .expect("cargo profile dir")
        .to_path_buf()
}

fn wrap_notice_helper(swift_bin: &Path, dest_dir: &Path, icon: &Path) {
    let exe = swift_bin.join("BronzeNotice");
    assert!(
        exe.is_file(),
        "swift build must produce BronzeNotice next to BronzeNative"
    );
    let app = dest_dir.join("BronzeNotice.app");
    let macos = app.join("Contents/MacOS");
    let resources = app.join("Contents/Resources");
    std::fs::create_dir_all(&macos).expect("BronzeNotice.app MacOS");
    std::fs::create_dir_all(&resources).expect("BronzeNotice.app Resources");
    std::fs::copy(&exe, macos.join("BronzeNotice")).expect("copy BronzeNotice");
    std::fs::write(app.join("Contents/Info.plist"), NOTICE_HELPER_PLIST)
        .expect("BronzeNotice Info.plist");
    if icon.is_file() {
        std::fs::copy(icon, resources.join("AppIcon.icns")).expect("BronzeNotice icon");
    }
    std::fs::write(app.join("Contents/PkgInfo"), "APPL????").expect("BronzeNotice PkgInfo");
    sign_notice_helper(&app, dest_dir);
}

/// Ad-hoc signatures pin the designated requirement to a cdhash, so
/// Notification Center treats each rebuild as a different app. A persistent
/// local cert keeps identifier + certificate leaf stable across tauri-dev.
fn sign_notice_helper(app: &Path, dest_dir: &Path) {
    let target_dir = dest_dir.parent().unwrap_or(dest_dir);
    let identity = ensure_notice_signing_identity(target_dir);
    let previous = user_keychains();
    if let Some(keychain) = identity.as_ref() {
        let mut next = vec![keychain.clone()];
        for path in &previous {
            if path != keychain {
                next.push(path.clone());
            }
        }
        set_user_keychains(&next);
        unlock_notice_keychain(Path::new(keychain));
    }
    let mut sign = Command::new("/usr/bin/codesign");
    sign.args(["--force", "--deep", "--sign"]);
    if identity.is_some() {
        sign.arg("Bronze Notice");
    } else {
        sign.arg("-");
    }
    sign.args(["--identifier", "app.bronze.desktop.notice"])
        .arg(app);
    let signed = sign.status().expect("codesign BronzeNotice.app");
    set_user_keychains(&previous);
    assert!(
        signed.success(),
        "codesign must bind Info.plist onto BronzeNotice.app"
    );
}

fn ensure_notice_signing_identity(target_dir: &Path) -> Option<String> {
    let keychain = target_dir.join("bronze-notice-signing.keychain-db");
    if !keychain.exists() {
        let created = Command::new("/usr/bin/security")
            .args(["create-keychain", "-p", NOTICE_SIGNING_PASS])
            .arg(&keychain)
            .status()
            .ok()?;
        if !created.success() {
            return None;
        }
        let _ = Command::new("/usr/bin/security")
            .args(["set-keychain-settings", "-t", "86400"])
            .arg(&keychain)
            .status();
    }
    unlock_notice_keychain(&keychain);
    if !notice_identity_present(&keychain) && !import_notice_signing_cert(&keychain) {
        return None;
    }
    let _ = Command::new("/usr/bin/security")
        .args([
            "set-key-partition-list",
            "-S",
            "apple-tool:,apple:,codesign:",
            "-s",
            "-k",
            NOTICE_SIGNING_PASS,
        ])
        .arg(&keychain)
        .status();
    Some(keychain.to_string_lossy().into_owned())
}

fn notice_identity_present(keychain: &Path) -> bool {
    let out = Command::new("/usr/bin/security")
        .args(["find-identity", "-p", "codesigning"])
        .arg(keychain)
        .output();
    out.ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("Bronze Notice"))
        .unwrap_or(false)
}

fn import_notice_signing_cert(keychain: &Path) -> bool {
    let dir = std::env::temp_dir().join("bronze-notice-signing");
    let _ = std::fs::create_dir_all(&dir);
    let cfg = dir.join("openssl.cnf");
    let key = dir.join("key.pem");
    let cert = dir.join("cert.pem");
    let p12 = dir.join("cert.p12");
    if std::fs::write(&cfg, NOTICE_OPENSSL_CNF).is_err() {
        return false;
    }
    let req = Command::new("/usr/bin/openssl")
        .args(["req", "-new", "-x509", "-days", "3650", "-nodes", "-config"])
        .arg(&cfg)
        .arg("-keyout")
        .arg(&key)
        .arg("-out")
        .arg(&cert)
        .status();
    if !req.map(|s| s.success()).unwrap_or(false) {
        return false;
    }
    let export = Command::new("/usr/bin/openssl")
        .args(["pkcs12", "-export", "-inkey"])
        .arg(&key)
        .arg("-in")
        .arg(&cert)
        .arg("-out")
        .arg(&p12)
        .args(["-passout", "pass:p12pass", "-name", "Bronze Notice"])
        .status();
    if !export.map(|s| s.success()).unwrap_or(false) {
        return false;
    }
    Command::new("/usr/bin/security")
        .arg("import")
        .arg(&p12)
        .args(["-k"])
        .arg(keychain)
        .args([
            "-P",
            "p12pass",
            "-T",
            "/usr/bin/codesign",
            "-T",
            "/usr/bin/security",
        ])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn unlock_notice_keychain(keychain: &Path) {
    let _ = Command::new("/usr/bin/security")
        .args(["unlock-keychain", "-p", NOTICE_SIGNING_PASS])
        .arg(keychain)
        .status();
}

fn user_keychains() -> Vec<String> {
    let out = Command::new("/usr/bin/security")
        .args(["list-keychains", "-d", "user"])
        .output();
    let Ok(out) = out else {
        return Vec::new();
    };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim().trim_matches('"');
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .collect()
}

fn set_user_keychains(paths: &[String]) {
    if paths.is_empty() {
        return;
    }
    let mut cmd = Command::new("/usr/bin/security");
    cmd.args(["list-keychains", "-d", "user", "-s"]);
    for path in paths {
        cmd.arg(path);
    }
    let _ = cmd.status();
}

const NOTICE_SIGNING_PASS: &str = "bronze-notice-local";

const NOTICE_OPENSSL_CNF: &str = r#"[ req ]
distinguished_name = dn
x509_extensions = ext
prompt = no
[ dn ]
CN = Bronze Notice
O = Bronze
[ ext ]
basicConstraints = critical,CA:FALSE
keyUsage = critical,digitalSignature
extendedKeyUsage = critical,codeSigning
"#;

const NOTICE_HELPER_PLIST: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0">
<dict>
  <key>CFBundleIdentifier</key>
  <string>app.bronze.desktop.notice</string>
  <key>CFBundleName</key>
  <string>Bronze</string>
  <key>CFBundleDisplayName</key>
  <string>Bronze</string>
  <key>CFBundleExecutable</key>
  <string>BronzeNotice</string>
  <key>CFBundleIconFile</key>
  <string>AppIcon</string>
  <key>CFBundleVersion</key>
  <string>0.0.0</string>
  <key>CFBundleShortVersionString</key>
  <string>0.0.0</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>LSUIElement</key>
  <true/>
  <key>NSPrincipalClass</key>
  <string>NSApplication</string>
  <key>NSUserNotificationAlertStyle</key>
  <string>banner</string>
  <key>NSHighResolutionCapable</key>
  <true/>
  <key>NSUserNotificationsUsageDescription</key>
  <string>Bronze shows a local banner when a capture finishes or is blocked.</string>
</dict>
</plist>
"#;

fn swift_runtime_dirs() -> Vec<PathBuf> {
    let out = Command::new("swift")
        .arg("-print-target-info")
        .output()
        .expect("swift -print-target-info");
    let text = String::from_utf8_lossy(&out.stdout);
    let mut dirs = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim().trim_matches(',').trim_matches('"');
        if trimmed.contains("lib/swift") && Path::new(trimmed).is_dir() {
            dirs.push(PathBuf::from(trimmed));
        }
    }
    dirs
}
