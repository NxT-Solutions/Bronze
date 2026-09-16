# Story 6.2: Item list lifecycle and menus

Status: done

## Story

As a user,
I want to edit, complete, skip, and trash items from buttons/menus,
So that the queue is usable without drag.

**Requirements:** QUE-002, QUE-003, QUE-006, A11Y-002, A11Y-004, A11Y-006
**ADRs:** ADR-012
**Dependencies:** Story 6.1
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** items exist
**When** cards use semantic list/article and real buttons
**Then** Move Up/Down exist; drag is optional
**And** accessible name contains visible label
**And** keyboard operates all actions

**Failure / recovery:**
Clickable div fails the story.

**Security / privacy / diagnostics / a11y / i18n / data:**
200% text still usable. No hover-only actions.

**Automated verification:**
- `pnpm --filter desktop test -- queue`
- `pnpm --filter desktop test -- a11y`

**Signed-build / manual evidence:**
axe/keyboard tests

**Non-goals:**
VoiceOver sign-off


## Dev Agent Record

### Agent Model Used

Hedgehog authored loop / layer-eng (cursor-grok).

### File List

- `bronze-storage/src/queue.rs` — complete/skip/trash/edit/move up-down; drag not required
- `packages/ui/src/components/item-list.tsx` — semantic `ul`/`article`/`button`; keyboard actions
- `packages/i18n/locales/*/app.json` — `queue.item.*` action labels

### Notes

- VoiceOver sign-off is a non-goal. Proposed ADR-002 / ADR-009 / ADR-018 remain unresolved.
