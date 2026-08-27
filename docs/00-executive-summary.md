# Executive summary

## Decision

Build Bronze as a new macOS-first application. Do not fork Cooper as production base. Reuse behavior-level ideas, not implementation or trade dress.

## What product is

Bronze is deliberate local workflow:

`selected content → captured item → optional prompt/note → ordered queue → copy to target → explicit completion`

Core advantage is continuity. User parks useful context and future prompts without switching into a full task manager or passively recording clipboard history.

## Why market has room

Existing products each own part: PopClip/ActionClip selected-text actions; Antinote/SideNotes scratchpads; Pastebot/PastePal paste queues; Things quick entry; Raycast/BetterTouchTool configuration. Copper packages exact loop but costs $39 and publishes little accessibility evidence. Cooper copies concept but demonstrates reliability, security, data, and accessibility failures. macOS 26 now includes clipboard history, so Bronze must win on deliberate semantic queue, not raw history.

Positioning:

> Offline, local-first, accessibility-first macOS selection-to-action queue for AI and human workflows.

## Product boundary

P0 includes text capture, manual entry, ordered sections, edit/reorder/copy/complete/undo, search, configurable triggers, permission diagnostics, local persistence, backup/export, accessibility, i18n architecture, and signed distribution.

P0 excludes images/files, OCR, sync, hosted or built-in AI, reusable prompt marketplace, passive clipboard recording, complex project management, and non-macOS platforms.

## Hard engineering problem

Keystroke detection is not enough. Reliable capture needs:

1. Listen-only native trigger monitor with explicit Input Monitoring status.
2. Serial state machine with configurable double-tap timing and reset rules.
3. AX-first selected-text acquisition with secure-field rejection.
4. Clipboard-copy fallback only after modifiers release, using `changeCount`, guaranteed key-up cleanup, and bounded waits. P0 never attempts automatic clipboard restoration because `NSPasteboard` has no atomic compare-and-swap.
5. Queued requests; no `AtomicBool` drop behavior.
6. Stable signing identity as necessary—never sufficient—for TCC continuity, plus signed-upgrade tests and accurate recovery when macOS resets trust.
7. Stage-level local diagnostics and visible accessible failure recovery.

## Proposed stack

- Tauri 2 desktop shell; Rust workspace for core, macOS adapter, store, and typed IPC.
- React + TypeScript UI; owned shadcn components using React Aria base; Tailwind tokens.
- pnpm + Turborepo for JS/TS workspace; Cargo workspace for Rust.
- Biome for JS/TS format/lint; rustfmt, Clippy, cargo-nextest or cargo test.
- SQLite WAL with versioned transactional migrations, FTS5, tombstone/undo, relative asset paths, and deterministic exports.
- i18next/ICU-style messages, BCP 47 locales, `Intl`, RTL and pseudo-locale tests.
- Vitest, Testing Library, axe-core, Playwright/WebDriver where viable, Rust unit/integration tests, and manual macOS/assistive-technology matrix.

Exact dependency versions must be locked when implementation starts. Dated registry snapshot belongs in [system architecture](06-system-architecture.md), not treated as policy.

## Accessibility claim policy

“Fully WCAG proof” is not valid product language. WCAG defines web-page conformance; native shell, permissions, global shortcuts, and platform interaction extend beyond it. Target evidence:

- WebView content conforms to WCAG 2.2 AA.
- Whole app maps to relevant EN 301 549 clauses 5, 11, and 12.
- VoiceOver meets Apple evaluation criteria.
- Full Keyboard Access, Voice Control, Switch Control, 200% text resize, 400%/320 CSS px reflow, contrast, Differentiate Without Color, motion, transparency, IME, keyboard layout, and RTL journeys pass.
- Publish row for every WCAG 2.2 A/AA criterion, native/EN mapping, and known limitations. Commission independent audit before public conformance claim.

## Delivery strategy

Start with risk spikes, not UI polish: permission identity, event tap, AX selection across app matrix, clipboard fallback, panel behavior across Spaces/displays, and signed update continuity. Then build domain/store, UI, settings, conformance, and release hardening. Each phase has entry/exit evidence in [agentic implementation plan](14-agentic-implementation-plan.md).

## Principal risks

| Risk | Control |
| --- | --- |
| Selection APIs vary across apps | AX-first provider chain, compatibility matrix, clipboard fallback, source-specific diagnostics |
| TCC permission mismatch after update | Stable bundle ID/team/signature; signed test upgrade path |
| Secure Input suppresses monitoring | Detect or infer; explain; expose menu/chord/manual alternatives |
| Clipboard damage | AX/manual path first; avoid sentinel; synthetic fallback off by default; no automatic restoration in P0 |
| Keyboard gesture inaccessible/unreliable | Optional configurable double Shift; standard chord/status-menu/manual composer alternatives |
| WebView compromise reaches native APIs | strict CSP, minimal per-window capabilities, typed validation, no arbitrary paths |
| “Accessible” claim outruns evidence | release-gated conformance matrix and external audit |
| Scope expands into clipboard/task/AI suite | explicit non-goals and P0/P1/P2 gates |
