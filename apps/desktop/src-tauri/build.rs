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
            "--product",
            "BronzeNative",
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
}

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
