# Bronze agent instructions

Applies to every file below this directory.

## Mission

Build Bronze from the approved requirements and architecture. Bronze is a deliberate selection-to-action work queue. It is not a general clipboard recorder, task manager, note vault, or AI client.

## Operating rules

1. Read `README.md`, `docs/03-prd.md`, relevant subsystem spec, and `docs/18-adrs.md` before changing code.
2. Do not perform Git operations unless the user explicitly asks, except `hedgehog verify` which is allowed to commit a passing task.
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

Run relevant formatter, static analysis, unit tests, integration tests, WebView accessibility tests, and native smoke tests. Report commands and evidence. Do not claim success from compilation alone. Never claim WCAG, VoiceOver, or notarization without current built-artifact evidence.

## Independent implementation and provenance rule

Do not copy Copper screenshots, video frames, text, icons, name, or pixel-identical trade dress. Cooper source was inspected during research, so do not claim a legal clean-room process. Bronze v1 copies no Cooper source code, even where Apache-2.0 would permit reuse with conditions. Reimplement from behavior-level requirements and record source provenance for architecture decisions.

<!-- bmad:context -->
<!-- Verified 2026-08-27 against f815cff. Managed by bmad-project-context; edits inside this block are replaced on refresh. Keep anything you want preserved outside the markers. -->

## bronze-app

Local-first macOS selection-to-action queue. Tauri 2, React, Rust, in-process Swift. Planning lives in `docs/` and `_bmad-output/planning-artifacts/`. Stories in `_bmad-output/planning-artifacts/epics.md`.

## Policy

- Work on a feature branch from `main`. Do not commit on `main`. Never push `main`. Push a feature branch only when the user asks to open or update a PR.
- Never accept ADR-002, ADR-009, or ADR-018 silently.
- Never claim WCAG, VoiceOver, or notarization without `docs/evidence/` artifacts.
- Never copy Cooper source or Copper trade dress.
- Execution orchestrator is **Hedgehog** (`hedgehog next` / `hedgehog verify`). Do not resume `bmad-loop` run `6a79` or older runs.

## Where things are

- Requirements: `docs/03-prd.md` (IDs are authority)
- ADRs: `docs/18-adrs.md`
- Capture: `docs/07-macos-capture-reliability.md`
- BMAD PRD/UX/arch: `_bmad-output/planning-artifacts/`
- Source map: `_bmad-output/planning-artifacts/SOURCE-MAP.md`

## Running and verifying

- Planning pack: `python3 tooling/planning-checks.py`
- After workspace exists: `pnpm verify`
- Task loop: `hedgehog status` then `hedgehog next` then `hedgehog verify <task-id>`

## Known pitfalls

- Event-tap callbacks must not touch AX, DB, windows, or clipboard.
- `SettingsV1.capture.standardChord` is not the full shortcut registry.

<!-- /bmad:context -->

## Hedgehog loop

State for the next build step lives in `.hedgehog/hedgehog.db`. Recover with `hedgehog status`.

```bash
hedgehog status
hedgehog next
hedgehog verify <task-id>
```

An agent reporting success never moves a task — only a passing `hedgehog verify` does.

Agent files: `.cursor/agents`. Skills: `.cursor/skills` and `.claude/skills`. Project context: `HEDGEHOG.md`. Design authority for chrome: `hig/`. Accessibility authority for WebView chrome: `wcag/`.

| Agent | Use when | File |
| --- | --- | --- |
| `planner` | New scope / adopt intake | `planner.md` |
| `tweaker` | Next change on an adopted repo | `tweaker.md` |
| `reviewer` | Layer/phase review (read-only) | `reviewer.md` |
| `bootstrap` | Greenfield workspace only — do not run on Bronze | `bootstrap.md` |

## Apple HIG (all agents)

Cursor, Claude Code, Codex, Grok, and any future OSS agent use the same
library. Do not keep a second copy of these rules in a vendor folder.

- Start: `hig/SKILL.md`
- Principles, macOS, tokens, surfaces: `hig/principles.md`, `hig/macos.md`, `hig/bronze.md`, `hig/screens.md`
- Adapter map: `hig/README.md`
- Cursor rule: `.cursor/rules/apple-hig.mdc` (chrome globs)
- Cursor / Claude skills: `.cursor/skills/apple-hig/`, `.claude/skills/apple-hig/` (pointers only)
- Claude Code entry: `CLAUDE.md`

Applies on any Settings, Library, Help, queue, CSS, typography, motion,
or “Apple-like” change.

## WCAG 2.2 (all agents)

Cursor, Claude Code, Codex, Grok, and any future OSS agent use the same
library. Do not keep a second copy of these rules in a vendor folder.

- Start: `wcag/SKILL.md`
- POUR, criteria, tokens, surfaces: `wcag/principles.md`, `wcag/criteria.md`, `wcag/bronze.md`, `wcag/screens.md`
- Adapter map: `wcag/README.md`
- Cursor rule: `.cursor/rules/wcag.mdc` (chrome globs)
- Cursor / Claude skills: `.cursor/skills/wcag/`, `.claude/skills/wcag/` (pointers only)
- Claude Code entry: `CLAUDE.md`

WebView target is WCAG 2.2 AA. Meet AAA where tokens or markup allow.
Do not claim WCAG, VoiceOver, or notarization without `docs/evidence/`.

Applies on any Settings, Library, Help, queue, CSS, contrast, focus,
keyboard, label, or “WCAG / a11y” change.

Do not run Hedgehog `full-stack-app` / `pwa-app` / `landing-page` cores. Bronze is the **adopted** core: keep Tauri 2, Rust, in-process Swift, and existing ADRs.

## Code intelligence

GitNexus indexes this repository for agents. It is developer tooling, not part of the desktop app. ADR-017 still means the product has no network by default. Grok reads this file. GitNexus does not install a separate Grok editor.

<!-- gitnexus:start -->
# GitNexus — Code Intelligence

This project is indexed by GitNexus as **Bronze**.

> Index stale? Run `node .gitnexus/run.cjs analyze --index-only` from the project root — it auto-selects an available runner. No `.gitnexus/run.cjs` yet? Bootstrap with `npx`, `bunx`, or `pnpm dlx` — e.g. `bunx gitnexus@latest analyze` (npm 11 npx crash; #1939).

## Always Do

- **MUST run impact before editing.** Use `impact({target: "symbolName", direction: "upstream"})` or `node .gitnexus/run.cjs impact "symbolName" --direction upstream --repo .`; report callers, processes, and risk. Never substitute grep for graph analysis.
- **MUST analyze graph changes before committing.** Use `detect_changes({scope: "all"})` (MCP) or `node .gitnexus/run.cjs detect-changes --scope all --repo .` (CLI fallback). `partial: true` or `truncated: true` is not a clean check — a zero means unseen, not unaffected; re-run it. For regression review: `detect_changes({scope: "compare", base_ref: "main"})` or `node .gitnexus/run.cjs detect-changes --scope compare --base-ref "main" --repo .`.
- MUST warn on HIGH/CRITICAL `risk` pre-edit; never use `riskSharedAxes` to waive a HIGH/CRITICAL `risk` warning. Compare File/symbol: MCP File omits axes; Graph-RAG expands File.
- **MUST treat `risk: UNKNOWN` as unresolved, not as low.** An empty caller set is not evidence the symbol is unused — it can also mean the callers are not resolvable by the index (plain-object property access, dynamic dispatch, cross-language calls). `impact` pairs `UNKNOWN` with a `riskNote` saying so. Confirm with a text search before treating the symbol as safe to change or delete; do not proceed on the strength of a zero.
- **MUST use `query({search_query: "concept"})` for concepts/flows, `context({name: "symbolName"})` for a named symbol, or `impact` for blast radius, on read-only callers, dependencies, imports, or execution flow.** Graph first; text search only for empty/`UNKNOWN`/literals.
- For security review, `explain({target: "fileOrSymbol"})` lists taint findings (source→sink flows; needs `analyze --pdg`).

## Never Do

- NEVER edit a function, class, or method before MCP/CLI impact analysis.
- NEVER ignore HIGH or CRITICAL risk warnings from impact analysis, and never read `UNKNOWN` as an all-clear — it means the walk could not answer, which is the one verdict that requires confirming by other means.
- NEVER rename symbols with find-and-replace — use `rename` which understands the call graph.
- NEVER commit before MCP/CLI graph change analysis.

## Resources

| Resource | Use for |
| --- | --- |
| `gitnexus://repo/Bronze/context` | Codebase overview, check index freshness |
| `gitnexus://repo/Bronze/clusters` | All functional areas |
| `gitnexus://repo/Bronze/processes` | All execution flows |
| `gitnexus://repo/Bronze/process/{name}` | Step-by-step execution trace |

## CLI

| Task | Read this skill file |
| --- | --- |
| Understand architecture / "How does X work?" | `.claude/skills/gitnexus-exploring/SKILL.md` |
| Blast radius / "What breaks if I change X?" | `.claude/skills/gitnexus-impact-analysis/SKILL.md` |
| Trace bugs / "Why is X failing?" | `.claude/skills/gitnexus-debugging/SKILL.md` |
| Rename / extract / split / refactor | `.claude/skills/gitnexus-refactoring/SKILL.md` |
| Tools, resources, schema reference | `.claude/skills/gitnexus-guide/SKILL.md` |
| Index, status, clean, wiki CLI commands | `.claude/skills/gitnexus-cli/SKILL.md` |

<!-- gitnexus:end -->
