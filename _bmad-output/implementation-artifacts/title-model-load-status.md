# Title-model load status

Status: done

## Intent

Settings → General Title engine shows whether the local GGUF worker is idle, loading, hashing, ready, missing, or failed, without blocking capture.

**ADRs:** ADR-019 stays Proposed (`compact_title` first, selectable offline GGUF refine). ADR-017 stays Accepted (no runtime Hub). ADR-002, ADR-009, and ADR-018 stay Proposed.
**Human gates:** stories 3.9, 3.10, 5.5, and 9.3 stay backlog. No WCAG, VoiceOver, or notarization claim.

## Overview

The title worker already logs `bronze-title:` stages. `apply_diag` maps those stages to `{ tier, phase, reason }` and publishes them on the Settings-only Tauri event `title-engine-status`. Settings also snapshots the same DTO with command `title_engine_status` when the pane opens.

Phases: idle (extractive, no spinner); loading / hashing (spinner plus catalog Loading `{engine}` / Checking `{engine}`); ready (no spinner, catalog loaded copy); missing (existing vendor-command copy); failed (catalog `bad_hash` / `timeout` / `unreadable`, no huge paths). Refine logs do not flip a ready engine to failed. Focus stays on `#title-model`. Reduce Motion stops decorative spin; the live region still updates.

A running `tauri dev` must fully quit and restart once so the new command and event register.

## File list

- `bronze-title-model/src/status.rs` — `EnginePhase`, `apply_diag`, snapshot
- `apps/desktop/src-tauri/src/title_refine.rs` — `TITLE_ENGINE_STATUS_EVENT`, `title_engine_status`
- `apps/desktop/src/settings.html` — `span.title-engine-spinner`, `#title-model-status`
- `apps/desktop/src/settings-live.mjs` — lifecycle format / bind / refresh
- `apps/desktop/src/chrome.css` — `.title-engine-spinner`
- `packages/i18n/locales/{en,nl,fr,de,es,it,en-XA,ar-XB}/app.json`
- `docs/05-ux-ui-interaction-spec.md` §9 / §11
- `docs/06-system-architecture.md` IPC
- `docs/07-macos-capture-reliability.md`
- `docs/12-settings-and-shortcuts.md`
- `docs/18-adrs.md` ADR-019 (Status: Proposed)
- `README.md`
