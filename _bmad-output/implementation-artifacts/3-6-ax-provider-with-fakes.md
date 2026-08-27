# Story 3.6: AX provider with fakes

Status: ready-for-dev

## Story

As a developer,
I want AX-first provider using fixtures,
So that secure fields never leak.

**Requirements:** CAP-005, CAP-006, CAP-007, CAP-009
**ADRs:** ADR-006
**Dependencies:** Story 3.5
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** coordinator exists
**When** AX provider is implemented against fakes
**Then** secure/unknown protection fail closed with zero content in ABI/log/store
**And** whitespace preserved
**And** exclusion runs before content query
**And** empty/zero-width is no_selection without trim

**Failure / recovery:**
Any content on protected_content is S0.

**Security / privacy / diagnostics / a11y / i18n / data:**
Diagnostics have no text/title/URL/hash of secrets.

**Automated verification:**
- `cargo test -p bronze-capture ax`
- `swift test --package-path native/macos/BronzeNative --filter AX`

**Signed-build / manual evidence:**
fixtures; not real Safari/Chrome matrix

**Non-goals:**
synthetic Cmd-C


## Dev Agent Record

### Agent Model Used

### File List
