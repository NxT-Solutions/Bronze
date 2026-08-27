# Story 2.4: ABI ownership tests

Status: ready-for-dev

## Story

As a developer,
I want empty, embedded NUL, invalid UTF-8, large payload, cancel, shutdown-race tests,
So that bridge memory is proven before capture.

**Requirements:** CAP-004, SEC-006
**ADRs:** ADR-004, ADR-015
**Dependencies:** Story 2.3
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** linked app
**When** ABI conformance suite runs
**Then** each case completes exactly once or cancels
**And** 10k round-trip smoke optional if runtime allows; otherwise document follow-up

**Failure / recovery:**
Leak or double-free fails the story.

**Security / privacy / diagnostics / a11y / i18n / data:**
No content in diagnostics.

**Automated verification:**
- `swift test --package-path native/macos/BronzeNative`
- `cargo test -p bronze-platform-macos`

**Signed-build / manual evidence:**
tests; sanitizers when compatible

**Non-goals:**
capture matrix


## Dev Agent Record

### Agent Model Used

### File List
