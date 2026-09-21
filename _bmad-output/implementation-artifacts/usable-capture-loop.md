# Usable capture loop

Status: done

## Intent

Capture selected text from the last external app and show recent items in the menu bar extra.

**Requirements:** CAP-002, CAP-003, CAP-004, CAP-008, WIN-003, WIN-004
**Stories:** 3-2 (double-tap FSM), 5-1 (status extra), 5-3 (capture-only does not reveal the panel)
**ADRs:** Proposed ADR-002 / ADR-009 / ADR-018 remain unresolved. File and image attachments stay out (ADR-001).

## Overview

Capture (status menu, Bronze app menu, or Shift double-tap) reads AX selected text from the last non-Bronze PID. The provider creates an application AX element for that PID, walks at most 16 ancestors, and falls back to system-wide focused AX when that PID has no selection or no focused element. Own PID, Bronze, and `bronze-desktop` are skipped. Capture does not steal source focus and does not force-show the panel.

Every `ShortcutActionId` ships an enabled default (`docs/12` §4). Event-tap live fire is `capture.selection` Shift double-tap, either side, gap 250 ms, max hold 400 ms. Clipboard fallback stays `manual`.

The status extra left-click is Show. The menu lists the latest five overview items (click copies via the Plain profile; titles flattened and capped at 48 characters), then Capture, Help, and Quit. The inbox keeps `[hidden]` source, empty, and composer-error slots unrendered. Captured rows still show catalog `capture.source` (`From {appName}`) when a name is present.

## File list

- `bronze-platform-macos/src/ax_live.rs` — last-external PID, `read_capture_selection`, ancestor walk
- `bronze-platform-macos/src/bridge.rs` — shared event-tap enable and drain
- `bronze-settings/src/schema.rs` — default Shift double-tap enabled
- `bronze-settings/src/shortcuts.rs` — seeded defaults for every `ShortcutActionId`
- `apps/desktop/src-tauri/src/lib.rs` — capture pump, status extra `bronze-status`
- `apps/desktop/src-tauri/src/live_session.rs` — `LiveAxHost` uses `read_capture_selection`
- `apps/desktop/src/chrome.css` — `[hidden]` source / empty / composer-error stay unrendered
- `packages/ui/src/lib/settings-schema.ts` — `standardChordEnabled` default true
- `packages/i18n/locales/*/app.json` — `settings.shortcuts.live`, `help.capture.composer`
- `README.md` — Capture, Shift double-tap, status extra
- `docs/07-macos-capture-reliability.md` §4.3 / §9.1 — last-external target and walk
- `docs/06-system-architecture.md` §8.2 — seeded defaults; event-tap live fire is `capture.selection`
- `docs/12-settings-and-shortcuts.md` §3 — shipped double-tap default
- `docs/05-ux-ui-interaction-spec.md` — status extra and live capture chord

## Notes

- Status Capture while another app is focused is the reliable path; activating Bronze first can still miss if last-external is empty.
- Electron and some IDEs leave `AXSelectedText` empty on the focused leaf; the ancestor walk is the recovery in this build.
- File and image ingest, and attaching files to an existing record, need a new ADR plus a threat-model update before any picker or copy-into-app-data work.
- Human stories 3.9, 3.10, 5.5, and 9.3 stay backlog.
