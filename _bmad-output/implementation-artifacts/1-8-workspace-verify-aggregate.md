# Story 1.8: Workspace verify aggregate

Status: ready-for-dev

## Story

As a developer,
I want one local verify command covering format, lint, types, unit, i18n, planning-checks,
So that later stories have a gate.

**Requirements:** G-01 process, SEC-001 config
**ADRs:** ADR-003
**Dependencies:** Stories 1.2–1.7
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** workspaces exist
**When** root verify script is added
**Then** `pnpm verify` runs Biome/tsc/Vitest/i18n validate/cargo test/planning-checks
**And** signing/TCC tasks are not Turbo-cached
**And** verify fails if any required step fails

**Failure / recovery:**
Do not weaken a failing check to go green.

**Security / privacy / diagnostics / a11y / i18n / data:**
planning-checks remain required.

**Automated verification:**
- `pnpm verify`

**Signed-build / manual evidence:**
command output

**Non-goals:**
GitHub Actions beyond local script


## Dev Agent Record

### Agent Model Used

### File List
