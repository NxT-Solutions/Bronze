# Bronze agent instructions

Applies to every file below this directory.

## Mission

Build Bronze from the approved requirements and architecture. Bronze is a deliberate selection-to-action work queue. It is not a general clipboard recorder, task manager, note vault, or AI client.

## Operating rules

1. Read `README.md`, `docs/03-prd.md`, relevant subsystem spec, and `docs/18-adrs.md` before changing code.
2. Do not perform Git operations unless user explicitly asks in current task.
3. Preserve requirement IDs in issues, implementation notes, tests, and release evidence.
4. Keep macOS capture code native and behind a narrow typed interface. Never put DB, window, or clipboard work in event-tap callback.
5. Prefer AX selection. Clipboard simulation is bounded fallback, not primary path.
6. Never silently drop capture requests. Every request receives ID, terminal result, local diagnostic stage, and accessible feedback.
7. No network access by default, telemetry, account, remote fonts, CDN, hosted AI, or analytics. Any later network feature needs ADR, explicit opt-in, visible destination/payload class, and threat-model update.
8. No arbitrary filesystem paths from WebView. Validate capabilities, IPC schema, resource ownership, size, MIME, and canonical path in Rust.
9. Use semantic HTML and owned shadcn React Aria primitives correctly. Do not use clickable `div` elements. Do not remove focus outlines without equal `:focus-visible` replacement.
10. Every user-facing string belongs in locale catalog. No sentence concatenation. Test pseudo-locales and RTL.
11. New behavior ships with tests proportional to risk. Capture, migration, export, permission, security, and accessibility paths need failure tests.
12. Keep files cohesive. Split native actors, domain services, IPC adapters, and UI features before monoliths form.
13. Update docs and traceability when decisions or requirements change.

## Required verification before handoff

Run relevant formatter, static analysis, unit tests, integration tests, WebView accessibility tests, and native smoke tests. Report commands and evidence. Do not claim success from compilation alone. Never claim WCAG, VoiceOver, or platform conformance without current built-artifact evidence.

## Independent implementation and provenance rule

Do not copy Copper screenshots, video frames, text, icons, name, or pixel-identical trade dress. Cooper source was inspected during research, so do not claim a legal clean-room process. Bronze v1 copies no Cooper source code, even where Apache-2.0 would permit reuse with conditions. Reimplement from behavior-level requirements and record source provenance for architecture decisions.

<!-- bmad:context -->
<!-- Verified 2026-08-27 against f815cff. Managed by bmad-project-context; edits inside this block are replaced on refresh. Keep anything you want preserved outside the markers. -->

## bronze-app

Local-first macOS selection-to-action queue. Tauri 2, React, Rust, in-process Swift. Planning lives in `docs/` and `_bmad-output/planning-artifacts/`. Stories in `_bmad-output/planning-artifacts/epics.md`.

## Policy

- Work only on `bmad/bronze-autonomous` unless the user names another branch. Never push.
- Never accept ADR-002, ADR-009, or ADR-018 silently.
- Never claim WCAG, VoiceOver, or notarization without `docs/evidence/` artifacts.
- Never copy Cooper source or Copper trade dress.

## Where things are

- Requirements: `docs/03-prd.md` (IDs are authority)
- ADRs: `docs/18-adrs.md`
- Capture: `docs/07-macos-capture-reliability.md`
- BMAD PRD/UX/arch: `_bmad-output/planning-artifacts/`
- Source map: `_bmad-output/planning-artifacts/SOURCE-MAP.md`

## Running and verifying

- Planning pack: `python3 tooling/planning-checks.py`
- After workspace exists: `pnpm verify`

## Known pitfalls

- Event-tap callbacks must not touch AX, DB, windows, or clipboard.
- `SettingsV1.capture.standardChord` is not the full shortcut registry.
- Human stories 3.9, 3.10, 5.5, 9.3 are not agent-completable.

<!-- /bmad:context -->
