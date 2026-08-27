# Story 3.5: Serial capture coordinator

Status: ready-for-dev

## Story

As a developer,
I want monotonic request IDs and exactly one terminal outcome,
So that no silent drops.

**Requirements:** CAP-004, G-01, G-02
**ADRs:** ADR-006, ADR-015
**Dependencies:** Story 3.4
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** ingress exists
**When** coordinator is implemented
**Then** 20 synthetic triggers preserve order and all terminate
**And** overflow emits trigger_queue_overflow per missing ID
**And** success feedback cannot fire before persist hook

**Failure / recovery:**
Nonterminal request fails the suite.

**Security / privacy / diagnostics / a11y / i18n / data:**
Diagnostics independent of content transaction.

**Automated verification:**
- `cargo test -p bronze-capture coordinator`

**Signed-build / manual evidence:**
unit/property tests

**Non-goals:**
real AX


## Dev Agent Record

### Agent Model Used

### File List
