# Story 2.1: Versioned C ABI types

Status: ready-for-dev

## Story

As a developer,
I want fixed-width ABI version, tagged enums, pointer-plus-length UTF-8,
So that Rust and Swift share one contract.

**Requirements:** CAP-004, SEC-002
**ADRs:** ADR-004
**Dependencies:** Story 1.4
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** BronzeNative package exists
**When** ABI headers/module are added
**Then** version check fails closed on mismatch
**And** no NUL-terminated string reliance
**And** no unwind across boundary documented in tests

**Failure / recovery:**
Invalid UTF-8 rejected. Double completion forbidden.

**Security / privacy / diagnostics / a11y / i18n / data:**
Sensitive buffers have no debug description.

**Automated verification:**
- `swift test --package-path native/macos/BronzeNative`

**Signed-build / manual evidence:**
unit tests

**Non-goals:**
AX, event tap, pasteboard


## Dev Agent Record

### Agent Model Used

### File List
