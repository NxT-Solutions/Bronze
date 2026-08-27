# Story 7.1: Settings window schema

Status: ready-for-dev

## Story

As a user,
I want searchable grouped settings with per-field reset,
So that I can change behavior safely.

**Requirements:** SET-001, WIN-005
**ADRs:** ADR-016
**Dependencies:** Stories 1.7 and 5.2
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** settings capability exists
**When** settings window is implemented
**Then** schema matches docs/12 including ShortcutActionId and daily|weekly backup
**And** export preview flags sensitive literals
**And** no credentials/tokens/paths exported

**Failure / recovery:**
Saving invalid shortcut without rollback fails.

**Security / privacy / diagnostics / a11y / i18n / data:**
Keyboard accessible. Catalog strings.

**Automated verification:**
- `pnpm --filter desktop test -- settings`
- `cargo test -p bronze-settings`

**Signed-build / manual evidence:**
component/unit

**Non-goals:**
shortcut recorder (next)


## Dev Agent Record

### Agent Model Used

### File List
