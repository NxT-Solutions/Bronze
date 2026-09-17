# Capture keeps the snapshotted target

Status: done

## Intent

Persist the last-external app’s AX selection after menu Capture or Shift double-tap, not Bronze and not a permission dialog.

**Requirements:** CAP-002, CAP-003, CAP-005, WIN-003
**ADRs:** Proposed ADR-002 / ADR-009 / ADR-018 remain unresolved. No synthetic Cmd+C.

## Overview

`note_external_focus` runs on the 20 ms pump and immediately before a first-capture permission prompt. `read_capture_selection` does not resample frontmost. A known last-external PID is the only AX target: focused element, then `AXMainWindow` / `AXWindows` (8), with a bounded child search (24) for allowed text roles. System-wide focused AX runs only when that PID is missing, and never when the focused process is Bronze / `bronze-desktop` / own PID.

Unknown roles are not content-queried. Protected roles fail closed. Clipboard fallback stays `manual`.

## File list

- `bronze-platform-macos/src/ax_live.rs` — keep last-external; window and child walk; reject own system focus
- `bronze-platform-macos/src/lib.rs` — exports
- `apps/desktop/src-tauri/src/lib.rs` — snapshot-before-prompt still required
- `docs/07-macos-capture-reliability.md` — target and walk
- `docs/06-system-architecture.md` — persist read contract
- `README.md` — Capture keeps the other app’s selection

## Notes

- Electron/Chromium editors may still expose no AX selection; TextEdit is the known-good check.
- After granting Accessibility or Input Monitoring, quit Bronze fully and relaunch `pnpm --filter desktop tauri dev`.
- File and image ingest stay out until a new ADR plus a threat-model update.
- Human stories 3.9, 3.10, 5.5, and 9.3 stay backlog.
