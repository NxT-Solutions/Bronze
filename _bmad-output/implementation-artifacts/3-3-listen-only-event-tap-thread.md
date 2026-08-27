# Story 3.3: Listen-only event tap thread

Status: ready-for-dev

## Story

As a developer,
I want session listenOnly tap on a dedicated run-loop thread,
So that optional gesture can be enabled later.

**Requirements:** CAP-002, CAP-004
**ADRs:** ADR-004, ADR-005
**Dependencies:** Stories 2.3 and 3.2
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** FSM exists
**When** tap thread is implemented
**Then** callback does no AX, DB, window, clipboard, or log work
**And** SPSC is single-producer
**And** tap disable resets FSM

**Failure / recovery:**
If Input Monitoring is denied, degrade; keep chord/menu routes (even if unimplemented UI).

**Security / privacy / diagnostics / a11y / i18n / data:**
Never suppress events. No keycodes persisted.

**Automated verification:**
- `swift test --package-path native/macos/BronzeNative --filter EventTap`

**Signed-build / manual evidence:**
unit/fake tests; physical 1k trials are human

**Non-goals:**
prompting TCC


## Dev Agent Record

### Agent Model Used

### File List
