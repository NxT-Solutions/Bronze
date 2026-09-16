# Simple queue overview

Status: done

## Intent

Quick Panel is a scannable inbox (composer + active items + row Copy). Capture persists AX selection into the same store without stealing focus. NSStatusItem offers Show, Capture, Settings, Quit. Settings / Library / Help recreate after close.

**Requirements:** CAP-003, CAP-004, WIN-003, WIN-004, QUE-001, QUE-005
**Stories:** 5-1 (status item), 5-3 (capture-only silence), 7-3 (composer remains if capture is denied)
**ADRs:** Proposed ADR-002 / ADR-009 / ADR-018 remain unresolved.

## Overview

The Quick Panel is the inbox: composer, active items (`queued` / `copied` / `active`), and a per-row Copy control. Complete, skip, trash, edit, and move live in overflow. Done and skipped items leave this list.

Menu-bar status item and the Bronze app menu offer Show, Capture, Settings, and Quit (app menu also keeps Library and Help). Capture reads AX selected text and writes the same SQLite store as the composer. Capture does not steal source focus and does not force-show the panel. Show reveals the panel. Closed Settings / Library / Help windows recreate on the next open.

Seeded shortcuts stay disabled. Search stays substring-only (`QUE_007_COMPLETE=false`). Human stories 3.9, 3.10, 5.5, and 9.3 stay backlog.

## File list

- `apps/desktop/src-tauri/src/live_session.rs` — `AX_CAPTURE_LIVE`, `list_overview`, persist selection via Store
- `apps/desktop/src-tauri/src/lib.rs` — `on_capture_requested` persist, status item, recreate chrome windows
- `bronze-platform-macos/src/ax_live.rs` — focused AX selected text; secure field fail-closed
- `apps/desktop/src/index.html` — wordmark, unlabeled composer, inbox empty state, row Copy + overflow
- `apps/desktop/src/queue-live.mjs` — `list_overview_items`, `queue-changed`
- `README.md` — what you will see in `tauri dev`
- `docs/07-macos-capture-reliability.md` §4.3 — persist + no panel reveal + status menu
- `docs/06-system-architecture.md` §8.1 step 6 — debug show vs capture-only

## Notes

- 5-1: live `NSStatusItem` title Bronze; left-click Show; menu Show, Capture, Settings, Quit.
- 5-3: capture-only still does not reveal or focus the Quick Panel.
- 7-3: Accessibility / Input Monitoring denial still leaves the manual composer.
