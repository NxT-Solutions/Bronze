# Changelog

All notable changes to this project will be documented in this file.
This file is auto-updated by the release workflow.

## Unreleased

- Homebrew and GitHub `.pkg` upgrades no longer re-download ~850 MB of title models. Those files live in Application Support and download once, with a progress bar, when you pick an engine.
- Ad-hoc updates need Accessibility and Input Monitoring added again.

## v0.2.1 - 2026-09-29

### What's new

- Accessibility and Retest now show Not requested, plus the running path and code identity.
- This release no longer uses hardened runtime, so Add in System Settings can stick. Installed 0.2.0 cannot become trusted — quit it, install 0.2.1, then add Accessibility again.
- Copy no longer pulses the card ring.
- The Dock and Get Info icon is a flat full-bleed plate.
- WhatsApp selection works again.

## v0.2.0 - 2026-09-28

### What's new

- Capture from WhatsApp and other apps even when the selection is empty.
- Sort the queue newest or oldest. Cards move instead of jumping.
- Show more pages from the last row.
- The interface is available in nine more languages.
- Titles keep a reserved slot and show progress while a model loads.

## v0.1.2 - 2026-09-25

- The Dock icon fills the tile.

## v0.1.1 - 2026-09-25

- The package installs Bronze.app to /Applications and is not relocatable.
- The UI catalog is inside the app.

## v0.1.0 - 2026-09-25

- First public macOS packages (`bronze-macos-arm64.pkg` and `bronze-macos-x86_64.pkg`; ADR-002 Accepted). These packages are not a notarization claim.
- Title engines: bundled GGUF tiers, imported GGUF, local Ollama, and hosted providers as exclusive sources.
- Settings shows the running version. Check for updates reads GitHub latest (ADR-023 Proposed).
- Homebrew cask target is `NxT-Solutions/nxt-solutions-packages`.
- Pull-request CI covers JS, docs, portable Rust, and macOS Rust.
