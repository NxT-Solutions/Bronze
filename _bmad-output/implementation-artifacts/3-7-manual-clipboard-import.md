# Story 3.7: Manual clipboard import

Status: ready-for-dev

## Story

As a developer,
I want explicit Create from Clipboard,
So that users can capture without AX.

**Requirements:** CAP-003, CAP-005
**ADRs:** ADR-006
**Dependencies:** Story 3.5
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** coordinator exists
**When** manual clipboard command is added
**Then** it reads text only after explicit action
**And** synthetic fallback remains off
**And** stale generation fails clipboard_changed

**Failure / recovery:**
Do not restore clipboard. Do not ingest files/images.

**Security / privacy / diagnostics / a11y / i18n / data:**
Shared pasteboard disclosure not required for manual path beyond help later.

**Automated verification:**
- `cargo test -p bronze-capture clipboard_manual`

**Signed-build / manual evidence:**
unit tests

**Non-goals:**
synthetic Cmd-C enablement


## Dev Agent Record

### Agent Model Used

### File List
