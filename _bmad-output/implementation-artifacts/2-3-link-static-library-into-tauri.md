# Story 2.3: Link static library into Tauri

Status: done

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

Hedgehog authored loop / layer-eng (cursor-grok). Docs layer records rust-layer evidence only.

### File List

- `apps/desktop/src-tauri/build.rs` — `swift build` of `BronzeNative`, links `libBronzeNative.a`, sets `bronze_native_linked`
- `apps/desktop/src-tauri/Cargo.toml` — `bronze-platform-macos` with `default-features = false` (no `abi-stub`)
- `apps/desktop/src-tauri/src/lib.rs` — startup `NativeRuntime::start()` + `check_abi_version` (CAP-004); tests: no helper/sidecar/`externalBin` (ADR-004, SEC-005 slot); linked ABI check under `bronze_native_linked`
- `Cargo.lock`

### Notes

- One TCC subject: in-process static lib; sidecar/helper is fail-closed.
- Debt #2 **resolved by design** (keep, not a stub-enable): `bronze-platform-macos` default feature `abi-stub` remains for crate-local tests. `bronze-desktop` links `libBronzeNative.a` with `default-features = false`. Do not re-enable `abi-stub` on `bronze-desktop`. Enforced by `apps/desktop/src-tauri/src/lib.rs` (`default-features = false` assertion) and documented on `bronze-platform-macos/src/abi_stub.rs`.
- UI layer was a no-op (no WebView surface for linking).
- Proposed ADR-002 / ADR-009 / ADR-018 remain unresolved.
