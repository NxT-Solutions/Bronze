# Story 3.2: Double-tap FSM tests

Status: ready-for-dev

## Story

As a developer,
I want pure modifier FSM with property tests,
So that gesture logic is proven without the tap.

**Requirements:** CAP-002, A11Y-001
**ADRs:** ADR-005
**Dependencies:** Story 1.4
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** Swift package exists
**When** FSM is implemented with table/property tests
**Then** valid pair emits one trigger; invalid sequences emit zero
**And** disabled by default
**And** timing window allows ≥500 ms
**And** no key stream retained

**Failure / recovery:**
Stuck state or wall-clock dependence fails.

**Security / privacy / diagnostics / a11y / i18n / data:**
Non-timed routes remain required. No character logging.

**Automated verification:**
- `swift test --package-path native/macos/BronzeNative --filter FSM`

**Signed-build / manual evidence:**
unit tests; not 1000 physical trials

**Non-goals:**
live CGEventTap


## Dev Agent Record

### Agent Model Used

### File List
