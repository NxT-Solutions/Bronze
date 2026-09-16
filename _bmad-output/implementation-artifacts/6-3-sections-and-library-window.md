# Story 6.3: Sections and library window

Status: done

## Story

As a user,
I want active section in the panel and a library window for archive/trash/search,
So that capabilities stay least-privilege.

**Requirements:** QUE-001, QUE-008, WIN-005, SEC-002
**ADRs:** ADR-010, ADR-007
**Dependencies:** Stories 5.2 and 6.2
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** queue UI exists
**When** library window is added
**Then** quick cannot import/export/backup
**And** library can paginate and archive
**And** empty/loading/read-only states exist

**Failure / recovery:**
Capability leak is S0.

**Security / privacy / diagnostics / a11y / i18n / data:**
Per-window commands tested negative.

**Automated verification:**
- `cargo test -p bronze-desktop capabilities`
- `pnpm --filter desktop test -- library`

**Signed-build / manual evidence:**
IPC denial tests

**Non-goals:**
import UI (epic 7)


## Dev Agent Record

### Agent Model Used

Hedgehog authored loop / layer-eng (cursor-grok).

### File List

- `apps/desktop/src-tauri/src/capabilities.rs` — quick denies import/export/backup; library paginates and archives
- `apps/desktop/src-tauri/tauri.conf.json` — hidden `library` window
- `packages/ui/src/components/library.tsx` — empty/loading/ready/read-only states
- `apps/desktop/src/library.html` — archive/trash/search surfaces
- `apps/desktop/src/index.html` — active section in the quick panel

### Notes

- Import UI is a non-goal (epic 7). Proposed ADR-002 / ADR-009 / ADR-018 remain unresolved.
