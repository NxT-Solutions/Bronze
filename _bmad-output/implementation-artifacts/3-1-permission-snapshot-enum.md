# Story 3.1: Permission snapshot enum

Status: ready-for-dev

## Story

As a developer,
I want closed permission states without prompting,
So that health UI later has a typed model.

**Requirements:** SET-003, SET-004, CAP-010
**ADRs:** ADR-005, ADR-015
**Dependencies:** Story 1.3
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** crates exist
**When** permission enum is implemented
**Then** values are unknown|not_requested|denied|granted_unverified|healthy|degraded|unavailable|requires_relaunch
**And** preflight APIs are wrapped but not called to prompt
**And** granted is not treated as healthy

**Failure / recovery:**
Unknown platform result maps to unknown/degraded, never healthy.

**Security / privacy / diagnostics / a11y / i18n / data:**
No key/content in snapshots.

**Automated verification:**
- `cargo test -p bronze-settings permission`
- `cargo test -p bronze-platform-macos permission`

**Signed-build / manual evidence:**
unit tests; not TCC UI

**Non-goals:**
System Settings deep link UI


## Dev Agent Record

### Agent Model Used

### File List
