# Story 2.2: Rust macOS façade

Status: ready-for-dev

## Story

As a developer,
I want bronze-platform-macos safe façade over the ABI,
So that domain crates never call Swift directly.

**Requirements:** CAP-004
**ADRs:** ADR-004
**Dependencies:** Stories 1.3 and 2.1
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** ABI types exist
**When** façade crate is implemented
**Then** init/shutdown and version query work
**And** bronze-domain has no macOS imports

**Failure / recovery:**
Panic at FFI boundary is contained.

**Security / privacy / diagnostics / a11y / i18n / data:**
No content logging.

**Automated verification:**
- `cargo test -p bronze-platform-macos`
- `cargo clippy -p bronze-platform-macos -- -D warnings`

**Signed-build / manual evidence:**
unit tests

**Non-goals:**
real AX queries


## Dev Agent Record

### Agent Model Used

### File List
