# Story 4.6: Deterministic export import

Status: ready-for-dev

## Story

As a developer,
I want JSON archive plus Markdown with contentLanguage,
So that users can leave.

**Requirements:** DAT-003, SET-001, I18N-003
**ADRs:** ADR-008, ADR-014
**Dependencies:** Stories 4.2 and 4.1
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** items persist
**When** export/import run
**Then** manifest is locale-neutral and sorted
**And** preview warns that item bodies may contain secrets
**And** path traversal archives fail
**And** settings export excludes diagnostics/paths/tokens

**Failure / recovery:**
Silent overwrite forbidden.

**Security / privacy / diagnostics / a11y / i18n / data:**
No machine paths. Hostile corpus required.

**Automated verification:**
- `cargo test -p bronze-storage export`

**Signed-build / manual evidence:**
golden/fuzz tests

**Non-goals:**
file picker UI


## Dev Agent Record

### Agent Model Used

### File List
