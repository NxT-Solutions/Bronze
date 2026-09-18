use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rustc-check-cfg=cfg(bronze_native_linked)");
    link_bronze_native();
    tauri_build::build();
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
    println!("cargo:rustc-link-lib=framework=NaturalLanguage");
    println!("cargo:rustc-link-arg=-Wl,-weak_framework,FoundationModels");
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
    let signed = Command::new("/usr/bin/codesign")
        .args([
            "--force",
            "--deep",
            "--sign",
            "-",
            "--identifier",
            "app.bronze.desktop.notice",
        ])
        .arg(&app)
        .status()
        .expect("codesign BronzeNotice.app");
    assert!(
        signed.success(),
        "codesign must bind Info.plist onto BronzeNotice.app"
    );
}

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
