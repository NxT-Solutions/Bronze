# Story 3.3: Listen-only event tap thread

Status: done

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

Hedgehog authored loop / layer-eng (cursor-grok). Docs layer records rust-layer evidence only.

### File List

- `native/macos/BronzeNative/Sources/BronzeNative/EventTapSPSC.swift` — bounded SPSC; single producer thread
- `native/macos/BronzeNative/Sources/BronzeNative/EventTapEngine.swift` — session listenOnly tap, dedicated run loop, pass-through callback, disable resets FSM
- `native/macos/BronzeNative/Sources/BronzeNative/EventTapABI.swift` — C ABI start/stop/health/feed/drain
- `native/macos/BronzeNative/Tests/BronzeNativeTests/EventTapTests.swift` — `--filter EventTap`
- `bronze-platform-macos/src/{abi,abi_stub,bridge,lib}.rs` — façade; stub degrades without prompt
- `apps/desktop/src-tauri/src/event_tap.rs` — linked SPSC + disable-reset tests
- `apps/desktop/src-tauri/src/lib.rs` — startup `event_tap_start()` (degrade on Input Monitoring denial)

### Notes

- UI layer was a no-op. No TCC prompt. No keycodes persisted. Proposed ADR-002 / ADR-009 / ADR-018 remain unresolved.
