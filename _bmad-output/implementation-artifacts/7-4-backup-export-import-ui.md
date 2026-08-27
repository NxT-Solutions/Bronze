# Story 7.4: Backup export import UI

Status: ready-for-dev

## Story

As a user,
I want Back Up Now, restore preview, and export/import flows,
So that data is portable.

**Requirements:** DAT-002, DAT-003, SET-001, QUE-008
**ADRs:** ADR-008
**Dependencies:** Stories 4.5, 4.6, 6.3
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** storage APIs exist
**When** library/settings UI calls them
**Then** native picker is Rust-owned
**And** automatic schedule cannot be off
**And** secret-content warning on queue export

**Failure / recovery:**
WebView path strings rejected.

**Security / privacy / diagnostics / a11y / i18n / data:**
Capability split library vs settings.

**Automated verification:**
- `pnpm --filter desktop test -- portability`
- `cargo test -p bronze-desktop portability`

**Signed-build / manual evidence:**
E2E mocked picker

**Non-goals:**
cloud backup


## Dev Agent Record

### Agent Model Used

### File List
