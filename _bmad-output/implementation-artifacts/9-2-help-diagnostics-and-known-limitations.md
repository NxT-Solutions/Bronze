# Story 9.2: Help diagnostics and known limitations

Status: ready-for-dev

## Story

As a user,
I want local help and redacted diagnostics preview,
So that support is possible offline.

**Requirements:** SUP-001, SUP-002, SEC-006
**ADRs:** ADR-015, ADR-017
**Dependencies:** Stories 3.8 and 7.1
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** settings exist
**When** help/about/diagnostics are implemented
**Then** support bundle is previewed before export
**And** no automatic upload
**And** known limitations include human gates

**Failure / recovery:**
Secret in bundle fails.

**Security / privacy / diagnostics / a11y / i18n / data:**
Accessible HTML help, not image-only.

**Automated verification:**
- `pnpm --filter desktop test -- support`
- `cargo test -p bronze-diagnostics bundle`

**Signed-build / manual evidence:**
unit/component

**Non-goals:**
public ACR


## Dev Agent Record

### Agent Model Used

### File List
