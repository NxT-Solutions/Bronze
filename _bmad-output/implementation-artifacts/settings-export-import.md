# Settings export and import

Status: done

## Intent

Settings can export a versioned `bronze-settings` JSON file and import it back. The Export preview card lists included categories and leftover user-entered literals that may be sensitive. Rust owns the save/open panels. WebView paths are rejected.

**Requirements:** SET-001, SEC-003
**ADRs:** ADR-010 and ADR-016 accepted. Proposed ADR-002 / ADR-009 / ADR-018 remain unresolved. Import does not silently apply ADR-018.
**Human gates:** stories 3.9, 3.10, 5.5, and 9.3 stay backlog. No WCAG, VoiceOver, or notarization claim.

## Overview

`preview_settings_export` returns category/field/sensitive/excluded keys only. `export_settings_file` and `import_settings_file` accept `requestedPath: null` and open `NSSavePanel` / `NSOpenPanel` on the AppKit thread. Event-tap callbacks do not open panels or touch AX, DB, windows, or clipboard.

The document is `{ format: "bronze-settings", version: 1, exportedAt, settings, shortcuts?, profiles? }`. It is not `bronze-export`. Library archive commands stay archive-only.

Excluded from the file: app credentials, permission tokens, diagnostics, machine paths, install identity, ephemeral state (including `tested` on shortcut bindings). Preview names leftover user-entered excluded apps, app policies, capture shortcut, default profile id, custom shortcut chords, and profile literals.

Import validates size (1 MiB), JSON, format, version, and forbidden keys. Shortcut registration can fail in part without rolling back applied settings. Settings Export is the one filled primary; Import is ghost.

## File list

- `bronze-settings/src/export.rs` — document, sanitize, preview keys, parse import
- `bronze-storage/src/profiles.rs` — list/replace output profiles for the file
- `bronze-platform-macos/src/settings_file.rs` — accept path, read bytes, rust-owned pickers
- `native/macos/BronzeNative` — `NSSavePanel` / `NSOpenPanel` ABI (not event-tap)
- `apps/desktop/src-tauri/src/live_session.rs` — preview / export / import commands
- `apps/desktop/src/settings.html` — live preview lists, Export, Import
- `apps/desktop/src/settings-live.mjs` — fill preview, invoke with `requestedPath: null`
- `packages/i18n/locales/*/app.json` — `settings.export.*` / `settings.import.*`
- `docs/05-ux-ui-interaction-spec.md` §9
- `docs/08-data-model-and-portability.md` §8
- `docs/09-security-privacy-threat-model.md` §8
- `docs/12-settings-and-shortcuts.md` §11–12
- `README.md` — Settings Export / Import

## Notes

- SET-001 remains planned in `docs/15` (search, reset, and human AT evidence are broader than this file).
- After rust lands, fully quit and restart `pnpm --filter desktop tauri dev` so the new commands load.
