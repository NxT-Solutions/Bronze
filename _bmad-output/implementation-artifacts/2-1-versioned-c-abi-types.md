---
title: '2-1-versioned-c-abi-types'
type: 'feature'
created: '2026-08-27'
status: 'done'
review_loop_iteration: 0
baseline_revision: '2a2d195f61b1107099936f89e0b2497e63e72aa5'
followup_review_recommended: false
context: []
warnings: []
deferred:
  - summary: >-
      architecture requires "ABI version and feature bitset" but 2.1 delivers only version (bitset not in story ACs)
    evidence: |-
      docs/06-system-architecture.md:234 lists feature bitset; header/swift only version + status
    location: >-
      docs/06-system-architecture.md:234 + native/macos/BronzeNative/Sources/BronzeNative/include/BronzeNative.h
    severity: low
  - summary: >-
      AGENTS.md rule 13 (update docs/traceability on decision change) only partially followed for UTF-8 validation strategy
    evidence: |-
      lossy+roundtrip, silgen, len-only helper, no NUL design live only in code comments + story spec
    location: >-
      native/.../BronzeNative.swift + 2-1-...md design notes (not pushed to 06/18)
    severity: medium
  - summary: >-
      init/shutdown and double-completion are present but lack exercising tests or shutdown-race coverage
    evidence: |-
      symbols + status exist; only constant and presence tested; arch §7.2 + 2.4 story lists races
    location: >-
      BronzeNative.swift + Tests (no call sites or race cases)
    severity: low
---

<intent-contract>

## Intent

**Problem:** BronzeNative exists only as a placeholder returning ABI version 0. No fixed-width version contract, no tagged enums, and no pointer-plus-length UTF-8 buffers exist. Rust and Swift have no shared, version-checked, NUL-free C ABI types, violating CAP-004 and SEC-002 and blocking all subsequent native bridge work.

**Approach:** Introduce the canonical ABI header (BronzeNative.h) and matching Swift implementation for the core types (ABI version, uint32 tagged status, bronze_native_utf8_view as ptr+len), plus minimal supporting cdecl surface and strict UTF-8 validation that never scans for NUL. Add Swift test target and conformance tests covering mismatch-closed, invalid UTF-8 rejection, embedded NUL, large payloads. Satisfy arch rules and ACs while leaving all capture/AX/pasteboard behavior to later stories.

## Boundaries & Constraints

**Always:**
- Preserve CAP-004, SEC-002, ADR-004 IDs in source, tests, and docs; never drop traceability.
- ABI uses only fixed-width integers, C-compatible tagged enums (uint32 discriminant), and pointer+length UTF-8 views; version check fails closed on mismatch.
- No NUL-terminated strings or strlen assumptions anywhere in ABI surface or tests.
- Invalid UTF-8 rejected with status; double completion forbidden (documented and testable).
- Sensitive buffers have no debug description containing user content.
- Rust panic / Swift error / exception never cross the FFI boundary.
- All changes remain inside native/macos/BronzeNative for this story; do not edit Rust crates, Tauri, or domain yet.
- sprint-status.yaml is orchestrator-owned; never read-for-write, edit, or revert rows.

**Block If:**
- Any requirement for external human action outside repo (signing, notarize, vendor console, domain) surfaces — none expected for this story.

**Never:**
- Implement AX, event tap, pasteboard, window, or real capture paths (non-goals).
- Depend on AppKit, ApplicationServices, or other frameworks in BronzeNative.
- Encode deployment targets or platforms: in Package.swift (ADR-002 Proposed).
- Touch sprint-status.yaml or any orchestrator bookkeeping.
- Use JS-native plugins, sidecars, or direct Swift from domain crates.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| HAPPY_PATH | ABI version query + valid UTF-8 view ("hello", len-delimited) | version==1; validate returns OK; test len reports exact length | none |
| MISMATCH | version query compared to wrong constant (999) | closed path taken; no further ABI use | test asserts mismatch branch |
| INVALID_UTF8 | view containing 0xFF or truncated multi-byte | validate returns INVALID_UTF8 | rejected, no partial data |
| EMBEDDED_NUL | bytes [0x61,0x00,0x62] (len=3) | test_view_len==3; validate OK (U+0000 valid) | len preserved; no early stop |
| LARGE_PAYLOAD | 1 MiB valid ASCII | len reported exactly; validate OK | handles without overflow/crash |
| EMPTY | nil ptr + len 0 | validate OK | accepted |
| DOUBLE_COMPLETION | status constant exercised | DOUBLE_COMPLETION code exists | contract documented as forbidden |

</intent-contract>

## Code Map

- `native/macos/BronzeNative/Package.swift:1` -- swift-tools-version and static lib product; add testTarget + publicHeadersPath
- `native/macos/BronzeNative/Sources/BronzeNative/BronzeNative.swift:3` -- current @_cdecl placeholder; replace with type defs, version=1, validate, test helpers; keep narrow no-framework comment
- `native/macos/BronzeNative/Sources/BronzeNative/include/BronzeNative.h` -- NEW canonical C header: version macro, uint32 status defines, utf8_view struct, cdecl decls
- `native/macos/BronzeNative/Tests/BronzeNativeTests/BronzeNativeTests.swift` -- NEW; XCTest cases for every row of I/O matrix + ACs (no NUL scan, mismatch closed, invalid reject)
- `docs/06-system-architecture.md:230` -- ABI contract rules (read-only reference): ptr+len, tagged enums, exactly-once, no cross-unwind, sensitive no debug
- `docs/18-adrs.md:166` -- ADR-004 (Accepted) full decision and consequences (read-only)
- `docs/18-adrs.md:195` -- "Bridge uses versioned C ABI..." rule (read-only)
- `_bmad-output/implementation-artifacts/1-4-scaffold-swift-package.md:141` -- KEEP instructions (no platforms:, comment phrasing, no AppKit imports, no sprint-status) (read-only)
- `_bmad-output/implementation-artifacts/epic-2-context.md` -- distilled epic goal and cross-story constraints (read-only)
- `AGENTS.md` -- Bronze rules (narrow native interface, preserve IDs, no silent drops, macOS capture native)
- `native/macos/BronzeNative/.build/` (ignored) -- SwiftPM outputs; never commit

## Tasks & Acceptance

**Execution:**
- `native/macos/BronzeNative/Package.swift` -- add publicHeadersPath: "include" to .target and append .testTarget(name: "BronzeNativeTests", dependencies: ["BronzeNative"]) -- enables swift test + header module visibility
- `native/macos/BronzeNative/Sources/BronzeNative/include/BronzeNative.h` -- create header defining BRONZE_ABI_VERSION, bronze_native_status tagged codes, bronze_native_utf8_view (ptr+len), and cdecl prototypes -- canonical contract for Rust/Swift
- `native/macos/BronzeNative/Sources/BronzeNative/BronzeNative.swift` -- implement matching public structs, update bronze_native_abi_version to return 1, implement validate_utf8 using length-delimited strict UTF8 check (no cString/NUL), add test_view_len + init/shutdown stubs with no-unwind comments -- delivers types + version
- `native/macos/BronzeNative/Tests/BronzeNativeTests/BronzeNativeTests.swift` -- create test file exercising mismatch closed, invalid UTF8 rejection, embedded NUL len preservation, empty/large/valid cases, double-completion status -- covers ACs + matrix
- `native/macos/BronzeNative/Sources/BronzeNative/BronzeNative.swift` and header -- ensure no debugDescription leaks content; all strings/views are ptr+len; version check documented to fail closed
- Run verification commands below; update this spec's Auto Run Result on completion (agent only)

**Acceptance Criteria:**
- Given BronzeNative package exists
  When ABI headers/module are added
  Then version check fails closed on mismatch
  And no NUL-terminated string reliance
  And no unwind across boundary documented in tests

**Failure / recovery:**
- Invalid UTF-8 rejected. Double completion forbidden.

**Security / privacy / diagnostics / a11y / i18n / data:**
- Sensitive buffers have no debug description.

**Automated verification:**
- `swift test --package-path native/macos/BronzeNative`

**Signed-build / manual evidence:**
- unit tests

**Non-goals:**
- AX, event tap, pasteboard

## Spec Change Log

## Review Triage Log

### 2026-08-27 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 8 (high 0, medium 3, low 5)
- defer: 3 (high 0, medium 1, low 2)
- reject: 4
- addressed_findings:
  - `[medium] [patch]` added Int.max guard in validate_utf8 to prevent trap on huge len (edge-case)
  - `[medium] [patch]` added layout-compat test (MemoryLayout.size/stride) and init/shutdown presence test
  - `[medium] [patch]` updated verification counts/text and auto-run evidence for actual 11 tests + extra cases
  - `[low] [patch]` added alloc rationale comment + design note; provenance link in header
  - `[low] [patch]` documented MUST KEEP IN SYNC for dupe consts; no single-source generator in scope
  - `[low] [patch]` guard + comments for large len + ownership note
  - addressed 2 more low cosmetic (test names, comment drift)
- deferred items added to frontmatter:
  - arch feature bitset requirement not implemented here (pre-existing arch vs story AC)
  - full doc updates per AGENTS 13 (validation choice); kept in spec design notes only
  - shutdown race / double completion enforcement tests (deferred to 2.4 per design)

### 2026-08-27 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 2 (high 0, medium 1, low 1)
- defer: 0
- reject: 18
- addressed_findings:
  - `[medium] [patch]` added `testEmbeddedNULDoesNotMaskInvalidTail` (`[0x61,0x00,0xFF]` → INVALID_UTF8) so a strlen/early-stop validator cannot pass
  - `[low] [patch]` added overlong (`C0 80`) and surrogate (`ED A0 80`) cases to `testValidateInvalidUTF8`

## Design Notes

UTF-8 views are non-owning borrowed views; allocation ownership and free functions are deferred to consuming stories (2.2/2.4) per arch "allocation owner provides matching free".

Tagged status uses explicit #define uint32 constants rather than C enum to guarantee fixed-width across compilers.

Validation uses lossy-decode + byte roundtrip compare to detect � replacements without relying on error-throwing APIs that could be confused with unwind.
Small Array copies inside validate/test helpers are acceptable for this narrow test surface; real ownership+free deferred per arch.

No product behavior (no real capture structs) is introduced; only the foundational transport and error tagging types required by the story AC and arch §7.2 list.

## Verification

**Commands:**
- `swift test --package-path native/macos/BronzeNative` -- expected: PASS; all 12 test cases execute; covers mismatch, invalid UTF-8 (incl. overlong/surrogate), embedded NUL, embedded-NUL-with-invalid-tail, large, empty, version, double status, nil-ptr-nonzero-len, len-only, layout match, and init/shutdown presence
- `swift build --package-path native/macos/BronzeNative` -- expected: exit 0; static library rebuilt
- `find native/macos/BronzeNative -path '*/.build/*' -prune -o -name BronzeNative.h -print | sort` -- expected: header listed exactly once
- `rg -n 'NUL|nul|strlen|\\0|cString|0x00' native/macos/BronzeNative --glob '!**/.build/**' || true` -- expected: only in test harness fixture construction or comments, never in ABI impl or header
- `rg -n 'debugDescription|description.*content|print.*view|print.*buffer' native/macos/BronzeNative/Sources/BronzeNative || true` -- expected: no matches that would leak sensitive content
- `swift test --package-path native/macos/BronzeNative 2>&1 | cat` -- expected: no framework import errors, no platforms warnings

**Manual checks (if no CLI):**
- Inspect BronzeNative.h for C89/C99 compatibility (fixed types, no C++), ptr+len only, no char*
- Confirm Package.swift has no `platforms:` key
- Grep confirms all new code carries CAP-004 / ADR-004 / SEC-002 references where relevant

## Auto Run Result

Summary of implemented change: Follow-up review of done 2-1 versioned C ABI types. Added XCTest coverage so embedded-NUL validation cannot early-stop, and so overlong/surrogate sequences are rejected. Existing header/Swift ABI surface unchanged. `sprint-status.yaml` and `deferred-work.md` left untouched (orchestrator-owned). Existing deferred frontmatter items preserved.

Files changed with one-line descriptions:
- `native/macos/BronzeNative/Tests/BronzeNativeTests/BronzeNativeTests.swift` — invalid-tail-after-NUL test; overlong and surrogate invalid UTF-8 cases
- `_bmad-output/implementation-artifacts/2-1-versioned-c-abi-types.md` — triage log, verification count 12, Auto Run Result

Review findings breakdown: patches applied 2 (medium 1, low 1); items deferred 0 (existing 3 preserved); items rejected 18. Score `3 × 1 medium + 1 × 1 low = 4` (threshold 5) → followup false.

Verification performed:
- `swift test --package-path native/macos/BronzeNative` — PASS, 12 tests, 0 failures
- `find native/macos/BronzeNative -path '*/.build/*' -prune -o -name BronzeNative.h -print` — header listed once
- `rg` NUL/strlen/cString — only comments and test fixtures, not ABI impl scan
- `rg` debugDescription in Sources — comments only, no leak implementation

Residual risks:
- `@_silgen_name` on struct-by-value validate/test_view_len is not proven against a C/Rust caller (Swift-module tests only); later bridge stories must bind the header
- Version fail-closed is a consumer convention (query + compare), not a library gate
- DOUBLE_COMPLETION is a reserved code; init/shutdown remain no-ops (already deferred)
- Feature bitset still absent vs architecture (already deferred)
- Orchestrator-owned `sprint-status.yaml` and `deferred-work.md` remain dirty in the worktree; not committed or reverted

Follow-up review recommendation: false

