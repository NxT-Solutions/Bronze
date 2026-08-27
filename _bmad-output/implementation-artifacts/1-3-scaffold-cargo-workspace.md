# Story 1.3: Scaffold Cargo workspace

Status: ready-for-dev

## Story

As a developer,
I want Cargo workspace crates with rustfmt and Clippy deny-warnings,
So that Rust domain has a home.

**Requirements:** DAT-001 (slot), SEC-002 (slot)
**ADRs:** ADR-003, ADR-008, ADR-010
**Dependencies:** Story 1.1
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** toolchain pin
**When** crates are created
**Then** `bronze-domain`, `bronze-capture`, `bronze-storage`, `bronze-settings`, `bronze-diagnostics`, `bronze-platform`, `bronze-platform-macos` compile empty
**And** Clippy `-D warnings` on workspace
**And** no SQL, AX, or Tauri commands yet

**Failure / recovery:**
Compilation failure blocks. Do not allow unused-mut warnings.

**Security / privacy / diagnostics / a11y / i18n / data:**
No macOS framework imports outside bronze-platform-macos.

**Automated verification:**
- `cargo fmt --check`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`

**Signed-build / manual evidence:**
local cargo logs

**Non-goals:**
schema, ABI, event tap


## Dev Agent Record

### Agent Model Used

### File List
