# Story 3.8: Content-free diagnostic schema

Status: ready-for-dev

## Story

As a developer,
I want typed diagnostic_events without content fields,
So that CAP-010 is structurally true.

**Requirements:** CAP-010, SEC-006, SUP-002
**ADRs:** ADR-015
**Dependencies:** Stories 1.3 and 3.5
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** diagnostics crate exists
**When** schema is implemented
**Then** type system rejects String payloads for selected text
**And** seeded secret scan of logs passes
**And** diagnostic write failure does not roll back a saved item fake

**Failure / recovery:**
Content type in schema is S0.

**Security / privacy / diagnostics / a11y / i18n / data:**
Support bundle preview comes in a later epic.

**Automated verification:**
- `cargo test -p bronze-diagnostics`

**Signed-build / manual evidence:**
type/unit tests

**Non-goals:**
export UI


## Dev Agent Record

### Agent Model Used

### File List
