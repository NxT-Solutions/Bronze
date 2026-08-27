# Story 6.1: Composer add item

Status: ready-for-dev

## Story

As a user,
I want to add a multiline note with Cmd-Enter,
So that I can park a prompt.

**Requirements:** QUE-002, CAP-003, I18N-003, A11Y-002
**ADRs:** ADR-012, ADR-014
**Dependencies:** Stories 1.7, 4.3, 5.2
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** panel and store exist
**When** composer is implemented
**Then** Cmd-Enter adds; Enter during isComposing does not
**And** failure retains draft
**And** content_language defaults to und
**And** strings from catalog

**Failure / recovery:**
Clearing composer before persist is forbidden.

**Security / privacy / diagnostics / a11y / i18n / data:**
IME-safe. Role/name tests. axe on empty/error.

**Automated verification:**
- `pnpm --filter desktop test -- composer`
- `pnpm verify`

**Signed-build / manual evidence:**
component tests; IME human later

**Non-goals:**
Markdown preview fetch


## Dev Agent Record

### Agent Model Used

### File List
