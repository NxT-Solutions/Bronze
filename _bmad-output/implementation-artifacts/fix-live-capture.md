# Live capture in tauri dev

Status: done

## Intent

Menu Capture and Shift double-tap persist selected text in the running `pnpm --filter desktop tauri dev` app, or show a visible failure.

**Requirements:** CAP-002, CAP-003, CAP-004, CAP-008, WIN-003
**ADRs:** Proposed ADR-002 / ADR-009 / ADR-018 remain unresolved. File and image attachments stay out (ADR-001). No synthetic Cmd+C.

## Overview

Last-external PID is `NSWorkspace.shared.frontmostApplication` (`bronze_native_frontmost_pid` / `note_external_focus`). The capture pump samples it every 20 ms. Menu Capture and Shift double-tap snapshot that PID before `prompt_on_first_capture_path`.

The listen-only tap is created on the `bronze.event-tap` run-loop thread. While health is not Listening, the pump retries start plus enable at most once per second. `flagsChanged` uses a Shift keycode when present, otherwise a `.maskShift` edge.

The AX walk queries `AXSelectedText`, then `AXSelectedTextRange` plus `AXStringForRange`, on allowed roles including `AXComboBox`. Persist emits `capture-result` `{ terminal, reason }` for every outcome and `queue-changed` only on Saved. The inbox `#capture-status` copies catalog `capture.announce.saved|rejected|denied|protected|failed`. Capture-only does not reveal or focus the panel.

Every `ShortcutActionId` ships an enabled default (`docs/12` §4). Event-tap live fire is `capture.selection` Shift double-tap, either side, gap 250 ms, max hold 400 ms. Clipboard fallback stays `manual`.

## File list

- `native/macos/BronzeNative/Sources/BronzeNative/FrontmostABI.swift` — `bronze_native_frontmost_pid`
- `native/macos/BronzeNative/Sources/BronzeNative/EventTapEngine.swift` — tap on `runTapLoop`; Shift keycode or `.maskShift`
- `bronze-platform-macos/src/ax_live.rs` — NSWorkspace last-external PID; selected text then range
- `apps/desktop/src-tauri/src/lib.rs` — pump retry; snapshot before prompt; `capture-result`
- `apps/desktop/src-tauri/src/live_session.rs` — `CaptureResultDto`
- `apps/desktop/src/queue-live.mjs` — `listenCaptureResult` / `applyCaptureResult`
- `apps/desktop/src/index.html` — `#capture-status` and catalog message slots
- `packages/i18n/locales/*/app.json` — `capture.announce.*`
- `README.md` — Capture, Shift double-tap, visible status
- `docs/07-macos-capture-reliability.md` — PID source, tap thread, retry, inbox feedback
- `docs/05-ux-ui-interaction-spec.md` — catalog announce strings
- `docs/06-system-architecture.md` — persist emit contract

## Notes

- After granting Accessibility or Input Monitoring, quit Bronze fully and relaunch `pnpm --filter desktop tauri dev`.
- Select text in another app. Capturing Bronze’s own WebView does not prove the path.
- A persist `Err` does not emit `capture-result`.
- File and image ingest stay out until a new ADR plus a threat-model update.
- Human stories 3.9, 3.10, 5.5, and 9.3 stay backlog.
