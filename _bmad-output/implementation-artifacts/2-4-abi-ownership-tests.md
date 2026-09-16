# Story 2.4: ABI ownership tests

Status: done

## Story

As a developer,
I want empty, embedded NUL, invalid UTF-8, large payload, cancel, shutdown-race tests,
So that bridge memory is proven before capture.

**Requirements:** CAP-004, SEC-006
**ADRs:** ADR-004, ADR-015
**Dependencies:** Story 2.3
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** linked app
**When** ABI conformance suite runs
**Then** each case completes exactly once or cancels
**And** 10k round-trip smoke optional if runtime allows; otherwise document follow-up

**Failure / recovery:**
Leak or double-free fails the story.

**Security / privacy / diagnostics / a11y / i18n / data:**
No content in diagnostics.

**Automated verification:**
- `swift test --package-path native/macos/BronzeNative`
- `cargo test -p bronze-platform-macos`

**Signed-build / manual evidence:**
tests; sanitizers when compatible

**Non-goals:**
capture matrix


## Dev Agent Record

### Agent Model Used

Hedgehog authored loop / layer-eng (cursor-grok). Docs layer records rust-layer evidence only.

### File List

- `native/macos/BronzeNative/Sources/BronzeNative/include/BronzeNative.h` — owned copy/free, probe begin/complete/cancel, outstanding, cancel/not-found/shutting-down statuses (CAP-004)
- `native/macos/BronzeNative/Sources/BronzeNative/BronzeNative.swift` — process-global ownership tables; complete XOR cancel; shutdown cancels open probes
- `native/macos/BronzeNative/Tests/BronzeNativeTests/BronzeNativeTests.swift` — XCTest ownership/cancel/shutdown cases
- `bronze-platform-macos/src/{abi,abi_stub,bridge,lib}.rs` — façade `OwnedUtf8` / `ProbeTerminal`; stub mirrors contract (do not re-enable `abi-stub` on bronze-desktop)
- `apps/desktop/src-tauri/src/abi_ownership.rs` — linked-app suite: empty, embedded NUL, invalid UTF-8, 1 MiB, cancel, shutdown-race, 10k round-trip, content-free debug (SEC-006)

### Notes

- Linked `bronze-desktop` is the Swift subject. 10k probe round-trip is implemented (not a follow-up).
- UI layer was a no-op.
- Proposed ADR-002 / ADR-009 / ADR-018 remain unresolved.
