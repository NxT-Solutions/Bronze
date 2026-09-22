# Queue title fit

Status: done

## Intent

Queue, library, and status-card titles are a short descriptive line that displays in full with no ellipsis.

**ADRs:** ADR-019 stays Proposed (`compact_title` first, selectable offline GGUF refine). Term-frequency best-sentence scoring is unchanged. ADR-002, ADR-009, and ADR-018 stay Proposed.
**Human gates:** stories 3.9, 3.10, 5.5, and 9.3 stay backlog. No WCAG, VoiceOver, or notarization claim.

## Overview

`compact_title` and `TitleABI.clampTitle` use a 40-character word-boundary clamp with no ellipsis glyph. Local refine uses the same clamp. That length fits one title row in the 400px Quick Panel (20px chrome + 16px card padding each side ≈ 328px at `--text-body` 0.9375rem). Queue and library `article [data-slot="title"]` use `text-overflow: clip`. Show more / Show less apply to the body only. The source row still ellipsizes. Status-menu `tray_item_title` stays a 48-character cap with an ellipsis.

Existing `items.title` rows keep their stored string until recapture or refine. After rust lands, fully quit and restart `pnpm --filter desktop tauri dev` so the new clamp loads.

## File list

- `bronze-domain/src/title.rs` — `TITLE_MAX_CHARS` 40, word-boundary clamp
- `native/macos/BronzeNative/Sources/BronzeNative/TitleABI.swift` — matching clamp
- `apps/desktop/src/chrome.css` — title `text-overflow: clip`
- `docs/05-ux-ui-interaction-spec.md` §4
- `docs/07-macos-capture-reliability.md`
- `docs/08-data-model-and-portability.md`
- `docs/18-adrs.md` ADR-019
- `README.md`
