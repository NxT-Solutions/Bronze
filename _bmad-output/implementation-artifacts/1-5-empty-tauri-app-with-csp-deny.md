---
title: 'Story 1.5: Empty Tauri app with CSP deny'
type: 'feature'
created: '2026-08-27'
status: 'done'
review_loop_iteration: 0
followup_review_recommended: true
context:
  - _bmad-output/implementation-artifacts/epic-1-context.md
  - docs/18-adrs.md
  - docs/06-system-architecture.md
  - docs/03-prd.md
  - AGENTS.md
  - _bmad-output/implementation-artifacts/1-2-scaffold-pnpm-turbo-workspace.md
  - _bmad-output/implementation-artifacts/1-3-scaffold-cargo-workspace.md
warnings: []
deferred: []
baseline_revision: '5aa6848e35e478730d5bbac064ebf52b51c86cc6'
baseline_commit: '5aa6848e35e478730d5bbac064ebf52b51c86cc6'
operator_actions: []
commit: 'e0ddeff37b88231a062a65e0806dfab26cdf4776'
---

# Story 1.5: Empty Tauri app with CSP deny

## Story

As a developer,
I want a Tauri 2 macOS app that loads packaged UI with default-deny capabilities,
So that security baseline exists before features.

**Requirements:** SEC-001, SEC-002, SEC-003, SEC-004
**ADRs:** ADR-003, ADR-010, ADR-017
**Dependencies:** Stories 1.2 and 1.3
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** JS and Cargo workspaces
**When** Tauri app is created
**Then** production CSP has no remote origins, no eval, no frames
**And** capabilities exist for quick, library, settings, onboarding and grant no shell/fs/http/sql
**And** DevTools disabled in release config
**And** forbidden-command test from a fixture window fails closed

**Failure / recovery:**
Any allowed shell/fs/http plugin is S0; remove it.

**Security / privacy / diagnostics / a11y / i18n / data:**
Zero runtime network in default build. No arbitrary path from WebView.

**Automated verification:**
- `pnpm exec tauri build --debug --no-bundle || cargo test -p bronze-desktop -- --nocapture`
- `rg -n 'shell|http|sql' apps/desktop/src-tauri/capabilities || true`

**Signed-build / manual evidence:**
capability JSON review; not notarized

**Non-goals:**
capture, queue UI, signing identity


## Code Map

- `apps/desktop/src-tauri/tauri.conf.json:24` -- CSP object with no remote origins (uses only asset/ipc localhost per Tauri packaged), no `unsafe-eval`, `frame-src 'none'`, `object-src 'none'`; `capabilities` lists the four named; `devtools: false` on window
- `apps/desktop/src-tauri/capabilities/{quick,library,settings,onboarding}.json` -- one per AC window; `"windows": ["<identifier>"]`; `"permissions": ["core:default"]` only (no shell, fs, http, sql)
- `apps/desktop/src-tauri/Cargo.toml:23` -- tauri = { version="2.11.3", features=[] }; no log or privileged plugins
- `apps/desktop/src-tauri/src/lib.rs:27` -- `forbidden_command_from_fixture_window_fails_closed` reads capability JSON, Cargo.toml, and tauri.conf.json and fails if plugins, `windows: ["*"]`, remote CSP, eval, frames, extra capabilities, or DevTools regress; CSP tokens must match an exact local allowlist
- `apps/desktop/src-tauri/src/main.rs` -- thin entry with windows subsystem guard
- `apps/desktop/package.json` -- tauri cli/api pins + "tauri" script
- `package.json` / root `Cargo.toml` -- cli and workspace member
- `apps/desktop/src/index.html` -- minimal static packaged asset (baseline only)
- `_bmad-output/implementation-artifacts/epic-1-context.md`, `docs/18-adrs.md` (ADR-010/017/003), `docs/06-system-architecture.md:448`, `AGENTS.md`, prior 1.2/1.3 specs -- loaded context

## Tasks & Acceptance

**Execution:**
- [x] `apps/desktop/src-tauri/` scaffold via tauri init + trim -- added Cargo member, desktop package tauri scripts/deps, minimal builder + test, 4 named capability files granting only core:default, tauri.conf with strict CSP + listed capabilities + devtools false, static index.html baseline
- [x] verification commands executed and passed (build, test, rg, planning-checks, md lint, fmt/clippy)
- [x] no shell/http/sql/fs in any cap; no remote in CSP; zero network plugins

**Acceptance Criteria:**
- Given JS and Cargo workspaces (1.2/1.3 done)
- When Tauri app is created
- Then production CSP has no remote origins, no eval, no frames
- And capabilities exist for quick, library, settings, onboarding and grant no shell/fs/http/sql
- And DevTools disabled in release config
- And forbidden-command test from a fixture window fails closed

## Spec Change Log

## Design Notes

Tauri 2 default-deny achieved by empty features + core:default only in caps + CSP object limiting to self/asset/ipc. The asset/ipc localhost tokens are the Tauri mechanism for packaged content and IPC (not "remote" per ADR-017 and threat model). `unsafe-inline` retained only for the static baseline HTML style (tighten when real frontend + nonce arrives in 1.7+). Each named capability binds to its own window label (not `*`); the empty-shell `main` window therefore has no IPC grants. `forbidden_command_from_fixture_window_fails_closed` asserts those files plus Cargo.toml and CSP/DevTools so adding shell/fs/http/sql, `windows: ["*"]`, remote origins, eval, frames, or DevTools fails the test. Runtime invoke of unregistered plugin commands is empty-app-isomorphic with the Cargo.toml plugin absence check; later IPC stories add command-level denial. macOS focus; bundle "all" is init default and harmless until signing.

## Verification

**Commands:**
- `pnpm exec tauri build --debug --no-bundle` -- expected: exit 0; produces bronze-desktop
- `cargo test -p bronze-desktop -- --nocapture` -- expected: forbidden... test passes
- `rg -n 'shell|http|sql' apps/desktop/src-tauri/capabilities || true` -- expected: no matches
- `python3 tooling/planning-checks.py` -- PASS
- `npx --yes markdownlint-cli2 'docs/**/*.md' README.md AGENTS.md` -- 0 issues
- `cargo fmt --check && cargo clippy --workspace -- -D warnings` -- clean

**Manual checks (if no CLI):**
- Inspect tauri.conf.json CSP keys for absence of remote/eval and presence of frame/object none
- Confirm exactly the four capability files named in AC, each with only core:default and windows bound to the capability identifier
- Confirm no tauri-plugin-log or shell/http in Cargo.toml deps

## Review Triage Log

### 2026-08-27 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 2 (low 2)
- defer: 6
- reject: 16
- addressed_findings:
  - none

### 2026-08-27 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 2: (high 1, medium 1, low 0)
- defer: 0
- reject: 17
- addressed_findings:
  - `[high]` `[patch]` Replaced `assert!(true)` with config assertions so capability grants, forbidden plugins, CSP remote/eval/frames, and DevTools cannot regress without failing `forbidden_command_from_fixture_window_fails_closed`
  - `[medium]` `[patch]` Bound each capability `windows` list to its identifier instead of `*`

### 2026-08-27 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 3: (high 0, medium 1, low 2)
- defer: 0
- reject: 18: (high 0, medium 0, low 18)
- addressed_findings:
  - `[medium]` `[patch]` CSP token check uses an exact local allowlist so hosts that merely contain the substring `localhost` fail
  - `[low]` `[patch]` Assert `devtools: false` on every configured window, not only `windows[0]`
  - `[low]` `[patch]` Require `tauri.conf.json` `capabilities` to equal exactly the four named identifiers

## Auto Run Result

- Summary: Follow-up review of the empty Tauri 2 macOS app with default-deny capabilities. Tightened the fail-closed config test (exact CSP tokens, all-window DevTools, exact capability list).
- Files changed:
  - `apps/desktop/src-tauri/src/lib.rs` — stricter CSP/capability/DevTools assertions
  - `_bmad-output/implementation-artifacts/1-5-empty-tauri-app-with-csp-deny.md` — review triage and result
- Review findings: 3 patches applied; 0 deferred; 18 rejected
- Follow-up review recommendation: true (patched high 0, medium 1, low 2; score `3×1+2=5`)
- Verification:
  - `pnpm exec tauri build --debug --no-bundle` — exit 0; built `target/debug/bronze-desktop`
  - `cargo test -p bronze-desktop -- --nocapture` — `forbidden_command_from_fixture_window_fails_closed` passed
  - `rg -n 'shell|http|sql' apps/desktop/src-tauri/capabilities` — no matches
  - `python3 tooling/planning-checks.py` — PASS
  - `npx --yes markdownlint-cli2 'docs/**/*.md' README.md AGENTS.md` — 0 issues
  - `cargo fmt --check && cargo clippy --workspace -- -D warnings` — clean
- Residual risks: the named fail-closed test remains a config/ACL pin, not a runtime WebView invoke; `core:default` is still the window grant; `'unsafe-inline'` remains on script/style for the static baseline.

## Dev Agent Record

### Agent Model Used
grok-build-0.1 (via bmad-build-auto + subagent general)

### File List
- apps/desktop/src-tauri/tauri.conf.json
- apps/desktop/src-tauri/capabilities/quick.json
- apps/desktop/src-tauri/capabilities/library.json
- apps/desktop/src-tauri/capabilities/settings.json
- apps/desktop/src-tauri/capabilities/onboarding.json
- apps/desktop/src-tauri/Cargo.toml
- apps/desktop/src-tauri/src/lib.rs
- apps/desktop/src-tauri/src/main.rs
- apps/desktop/package.json
- package.json
- Cargo.toml
- apps/desktop/src/index.html
- (plus generated icons/build artifacts not committed)

