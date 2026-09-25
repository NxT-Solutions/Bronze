# Changelog

All notable changes to this project will be documented in this file.
This file is auto-updated by the release workflow.

## Unreleased

- Work in progress.

## v0.1.1 - 2026-09-25

- The package installs Bronze.app to /Applications and is not relocatable.
- The UI catalog is inside the app.

## v0.1.0 - 2026-09-25

- First public macOS packages (`bronze-macos-arm64.pkg` and `bronze-macos-x86_64.pkg`; ADR-002 Accepted). These packages are not a notarization claim.
- Title engines: bundled GGUF tiers, imported GGUF, local Ollama, and hosted providers as exclusive sources.
- Settings shows the running version. Check for updates reads GitHub latest (ADR-023 Proposed).
- Homebrew cask target is `NxT-Solutions/nxt-solutions-packages`.
- Pull-request CI covers JS, docs, portable Rust, and macOS Rust.
