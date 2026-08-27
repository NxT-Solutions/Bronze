# Story 5.1: Status item and menu

Status: ready-for-dev

## Story

As a user,
I want an accessible status menu with Capture, New Note, Show, Settings, Quit,
So that I can work without a global hook.

**Requirements:** CAP-003, WIN-004, A11Y-001, I18N-001
**ADRs:** ADR-007, ADR-014
**Dependencies:** Stories 1.5, 1.6, 2.3
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** empty Tauri app
**When** status item is added
**Then** menu items have localized accessible names
**And** Capture uses last-external-target snapshot contract even if capture is stubbed
**And** menu works when event tap is off

**Failure / recovery:**
Missing accessible name fails.

**Security / privacy / diagnostics / a11y / i18n / data:**
Native strings from same catalog ownership.

**Automated verification:**
- `cargo test -p bronze-platform-macos status_item`
- `pnpm verify`

**Signed-build / manual evidence:**
unit; VoiceOver is human

**Non-goals:**
real capture success


## Dev Agent Record

### Agent Model Used

### File List
