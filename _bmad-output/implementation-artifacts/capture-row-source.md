# Capture row source

Status: done

## Intent

Show the source app on captured inbox rows.

**Requirements:** CAP-003, CAP-008
**Stories:** 5-1 (status-menu Capture), 5-3 (capture-only does not reveal the panel)
**ADRs:** Proposed ADR-002 / ADR-009 / ADR-018 remain unresolved.

## Overview

Menu-bar or Bronze-menu Capture, and Shift double-tap, persist AX selected text into the same SQLite store as the composer. The other twelve seeded `ShortcutActionId` rows stay disabled. Capture does not steal source focus and does not force-show the panel.

Each captured row stores CAP-008 app name from the focused process (`proc_name` for the AX element's PID). Bronze / `bronze-desktop` is omitted. URL and window title are not stored. The inbox shows catalog `capture.source` (`From {appName}`) when a name is present. Composer **Add** rows stay unlabeled.

## File list

- `bronze-capture/src/ax.rs` — `CapturedText.source_app_name`
- `bronze-platform-macos/src/ax_live.rs` — focused PID → process name
- `bronze-storage/src/composer.rs` — `sources.app_name` on capture; none for composer
- `bronze-storage/src/queue.rs` — `QueueItemRow.source_app_name`
- `apps/desktop/src-tauri/src/live_session.rs` — DTO `sourceAppName`
- `apps/desktop/src/index.html` — source slot, catalog `capture.source`
- `apps/desktop/src/queue-live.mjs` — `formatCaptureSource`; hide when name is absent
- `packages/i18n/locales/*/app.json` — `capture.source`
- `README.md` — Capture trigger and source line
- `docs/07-macos-capture-reliability.md` §4.3 / §9.4 — menu Capture + app-name provenance
- `docs/06-system-architecture.md` §8.2 — persist name; no shortcut chord

## Notes

- CAP-008 in this slice is app name only. Title, URL, and per-app provenance disable stay later.
- Human stories 3.9, 3.10, 5.5, and 9.3 stay backlog.
