# Story 1.5: Empty Tauri app with CSP deny

Status: ready-for-dev

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


## Dev Agent Record

### Agent Model Used

### File List
