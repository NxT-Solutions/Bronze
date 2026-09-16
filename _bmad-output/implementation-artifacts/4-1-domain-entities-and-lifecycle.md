# Story 4.1: Domain entities and lifecycle

Status: done

## Story

As a developer,
I want section/item entities with closed lifecycle and content_language,
So that UI cannot invent states.

**Requirements:** QUE-001, QUE-002, QUE-003, I18N-003, DAT-001
**ADRs:** ADR-008, ADR-014
**Dependencies:** Story 1.3
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** bronze-domain exists
**When** entities are implemented
**Then** lifecycle is queued|copied|active|done|skipped|trashed with a transition table
**And** content_language is BCP 47 or und defaulting to und
**And** invalid transitions fail

**Failure / recovery:**
Silent trim/normalize of body fails tests.

**Security / privacy / diagnostics / a11y / i18n / data:**
UTC timestamps. Locale-neutral enums.

**Automated verification:**
- `cargo test -p bronze-domain`

**Signed-build / manual evidence:**
unit tests

**Non-goals:**
SQLite yet


## Dev Agent Record

### Agent Model Used

Hedgehog authored loop / layer-eng (cursor-grok). Docs layer records rust-layer evidence only.

### File List

- `bronze-domain/src/entities.rs` — Section/Item; closed lifecycle; BCP 47 / `und`; no body trim
- `bronze-domain/src/lib.rs` — exports

### Notes

- UI layer was a no-op. SQLite is story 4.2. Proposed ADR-002 / ADR-009 / ADR-018 remain unresolved. Also set epic-4 in-progress.
