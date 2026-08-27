# Story 1.4: Scaffold Swift package

Status: ready-for-dev

## Story

As a developer,
I want BronzeNative Swift package skeleton,
So that native work has a package before ABI.

**Requirements:** CAP-001 (slot)
**ADRs:** ADR-004
**Dependencies:** Story 1.1
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** Xcode present
**When** Package.swift is added under native/macos/BronzeNative
**Then** package builds a static library with ABI version placeholder
**And** no event tap, AX, or AppKit window code yet

**Failure / recovery:**
If SwiftPM cannot build, stop; do not embed source in Tauri without a package.

**Security / privacy / diagnostics / a11y / i18n / data:**
No logging of strings. No JS surface.

**Automated verification:**
- `swift build --package-path native/macos/BronzeNative`

**Signed-build / manual evidence:**
local swift build

**Non-goals:**
CGEventTap, AX, status item


## Dev Agent Record

### Agent Model Used

### File List
