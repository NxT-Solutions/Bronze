# Story 1.1: Pin toolchain manifests

Status: ready-for-dev

## Story

As a developer,
I want pinned Node, pnpm, Rust, and Swift identity files,
So that later stories share one bootstrap.

**Requirements:** G-01 (process), SEC-005 (reproducible later)
**ADRs:** ADR-003; ADR-002 remains Proposed
**Dependencies:** none (canary)
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** a clean `bmad/bronze-autonomous` tree
**When** toolchain pin files are added
**Then** `package.json` packageManager, Node engines, `rust-toolchain.toml` matching docs/20, and Swift tools version notes exist
**And** no application windows, capture, or SQLite yet
**And** docs/20 versions remain the recorded host baseline

**Failure / recovery:**
If a pin disagrees with docs/20, fail the story and update the pin or the bootstrap record — do not silently raise macOS deployment target (ADR-002).

**Security / privacy / diagnostics / a11y / i18n / data:**
No secrets in pin files. No telemetry SDK. No user-facing strings.

**Automated verification:**
- `python3 tooling/planning-checks.py`
- `npx --yes markdownlint-cli2 'docs/**/*.md' README.md AGENTS.md`
- `test -f package.json`
- `test -f rust-toolchain.toml`

**Signed-build / manual evidence:**
file existence only; no signed app

**Non-goals:**
Tauri window, UI, native capture, CI cloud, Intel universal2


## Dev Agent Record

### Agent Model Used

### File List
