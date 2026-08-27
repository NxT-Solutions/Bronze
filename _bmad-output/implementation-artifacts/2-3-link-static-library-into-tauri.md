# Story 2.3: Link static library into Tauri

Status: ready-for-dev

## Story

As a developer,
I want the debug app links BronzeNative in-process,
So that one TCC subject exists.

**Requirements:** SEC-005 (identity later), CAP-001 slot
**ADRs:** ADR-004, ADR-011
**Dependencies:** Stories 1.5 and 2.2
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** façade and Tauri app exist
**When** the app is built
**Then** release-like bundle contains no helper executable
**And** startup calls ABI version check

**Failure / recovery:**
Link failure blocks; do not switch to a sidecar.

**Security / privacy / diagnostics / a11y / i18n / data:**
One process identity.

**Automated verification:**
- `cargo test -p bronze-desktop`
- `swift build --package-path native/macos/BronzeNative`

**Signed-build / manual evidence:**
local debug build; not Developer ID

**Non-goals:**
notarization, App Group


## Dev Agent Record

### Agent Model Used

### File List
