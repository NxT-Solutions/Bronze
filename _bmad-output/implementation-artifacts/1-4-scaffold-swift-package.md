---
title: 'Story 1.4: Scaffold Swift package'
type: 'feature'
created: '2026-08-27'
status: 'done'
review_loop_iteration: 1
followup_review_recommended: false
context:
  - _bmad-output/implementation-artifacts/epic-1-context.md
  - docs/18-adrs.md
  - docs/06-system-architecture.md
  - AGENTS.md
warnings: []
deferred: []
baseline_revision: '6edd60b8550e50523bb959bed20903e1bdd61582'
baseline_commit: '6edd60b8550e50523bb959bed20903e1bdd61582'
---

# Story 1.4: Scaffold Swift package

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


<intent-contract>

## Intent

**Problem:** Rust workspace scaffolding (Story 1.3) and toolchain pins (1.1) are complete, but the in-process Swift static library package required by ADR-004 has no skeleton. Subsequent stories (ABI types, event tap, AX, linking) have no SwiftPM target and no place to host native macOS code behind the narrow Rust façade in bronze-platform-macos.

**Approach:** Create exactly the package directory native/macos/BronzeNative/ with a Package.swift declaring a .static library product, a minimal Sources module that exposes only an ABI version placeholder symbol (via @_cdecl for future C ABI), ensure the package builds cleanly under SwiftPM with no framework imports or product behavior. Leave all capture/AX/AppKit/window code for later stories.

## Boundaries & Constraints

**Always:**
- Directory layout and product name exactly `native/macos/BronzeNative/` producing static library product named `BronzeNative`.
- Package.swift uses a swift-tools-version compatible with Swift 6.3.3 from bootstrap; do not duplicate version pins (root `.swift-version` is authoritative).
- The built artifact is a static library; `swift build --package-path native/macos/BronzeNative` must succeed.
- ABI version is a placeholder only (e.g. returning 0 or a simple constant); real versioned contract, symbols, and tests come in Epic 2.
- No event tap, AX, AppKit, NSPasteboard, status item, or window code of any kind (story non-goal + AGENTS rule 4).
- No user-facing strings, no logging, no JS surface, no network, no telemetry.
- Preserve requirement IDs (CAP-001 slot, ADR-004) and cross-references; never write or revert sprint-status.yaml (orchestrator-owned).
- bronze-platform-macos remains the sole Rust owner of the façade; this package is the Swift implementation detail.

**Block If:**
- SwiftPM build requires external registry packages or paid Xcode components beyond a stock Xcode install.
- Any decision on final ABI layout, ownership, or threading (deferred to 2-1/2-2).

**Never:**
- Any real capture implementation, permission logic, or AppKit activation.
- Direct imports of AppKit, ApplicationServices, CoreGraphics in this story.
- Tests exercising behavior (FSM, AX, ingress) — only the package skeleton and build.
- Adopt Proposed ADR-002, ADR-009, or ADR-018.
- Hard-code Swift version inside Package.swift or add .swift-version under the package dir.
- Extra files that would be rejected by later negative checks (e.g. stray binaries, scripts).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|---------------|----------------------------|----------------|
| HAPPY_PATH | clean tree post 1.1–1.3, Xcode present | `native/macos/BronzeNative/Package.swift` + Sources module exist; `swift build` exits 0 and emits a static library under .build/ | none |
| MISSING_PRODUCT | Package.swift declares no library or executable product | swift build fails or produces nothing matching "static library" | verification fails |
| FORBIDDEN_CODE | any CGEventTap, AXIsProcessTrusted, NSStatusItem, AppKit import | source grep catches it; build may succeed but story AC violated | must be removed before claiming pass |
| STRAY_PIN | swift-tools-version or platforms embed a version number instead of relying on .swift-version | rejected in review; pins live at repo root only | verification fails |
| BUILD_FAIL | SwiftPM cannot locate toolchain or package does not declare static lib | `swift build` exits non-zero | stop; do not proceed to embed |

</intent-contract>

## Code Map

- `native/macos/BronzeNative/Package.swift` -- declares the Swift package, static library product, and target (no `platforms:` while ADR-002 is Proposed); this is the sole file required by the AC "Package.swift is added"
- `native/macos/BronzeNative/Sources/BronzeNative/BronzeNative.swift` -- minimal module providing the ABI version placeholder symbol using @_cdecl for C ABI compatibility; contains only the placeholder and a file-level comment documenting scope
- `docs/18-adrs.md:166` -- ADR-004 (Accepted) is the governing decision: "Build BronzeNative Swift Package as static library linked into Tauri executable"
- `docs/18-adrs.md:195` -- bridge rules (versioned C ABI, explicit buffer ownership, no unwinding)
- `_bmad-output/planning-artifacts/epics.md:196` -- canonical story text and AC (source of truth for this spec)
- `docs/06-system-architecture.md:189` -- expected source tree: native/macos/BronzeNative/{Package.swift, Sources/, Tests/}
- `docs/06-system-architecture.md:230` -- ABI contract starts with "ABI version and feature bitset"
- `docs/06-system-architecture.md:211` -- ownership: BronzeNative owns CGEventTap/AX/AppKit/... (none of which exist in this story)
- `AGENTS.md:4` -- "Keep macOS capture code native and behind a narrow typed interface"
- `AGENTS.md:11` -- new behavior ships with tests proportional to risk (scaffold risk is low; build verification only)
- `_bmad-output/implementation-artifacts/epic-1-context.md:41` -- "Native: BronzeNative Swift package as static lib (ABI later); no event tap or AX yet"
- `.swift-version:1` -- authoritative Swift 6.3.3; Package.swift must not duplicate
- `bronze-platform-macos/Cargo.toml` and `src/lib.rs` -- future consumer façade slot (currently empty; no changes required in this story)
- `tooling/planning-checks.py` -- doc parity only (no native gate yet; 1.8 will aggregate)
- `.gitignore` -- already contains build/ and target/; may need **/.build/ for SwiftPM hygiene (carry from 1.3 pattern)

## Tasks & Acceptance

**Execution:**
- [x] `native/macos/BronzeNative/Package.swift` -- write the Package.swift declaring name "BronzeNative", products: [.library(name: "BronzeNative", type: .static, targets: ["BronzeNative"])], targets: [.target(name: "BronzeNative")]; omit `platforms:` so no macOS deployment minimum is encoded while ADR-002 is Proposed -- satisfies "Package.swift added" and enables static lib output
- [x] `native/macos/BronzeNative/Sources/BronzeNative/BronzeNative.swift` -- write the placeholder source with @_cdecl ABI version function returning a UInt32 placeholder value and a comment "ABI version placeholder only. No AppKit, AX, event tap, or product behavior in this story." -- delivers "with ABI version placeholder"
- [x] (dirs created as needed by file writes; no extra manifests or lockfiles for SwiftPM in v1 scaffolding)

**Acceptance Criteria:**
- Given Xcode present
- When Package.swift is added under native/macos/BronzeNative
- Then package builds a static library with ABI version placeholder
- And no event tap, AX, or AppKit window code yet

## Spec Change Log

### 2026-08-27 — bad_spec repair: omit Package.swift platforms

- **Trigger:** Review found `platforms: [.macOS(.v13)]` encodes a macOS deployment minimum while ADR-002 is Proposed (intent Never: do not adopt ADR-002; architecture: do not silently encode a minimum; STRAY_PIN: platforms must not embed a version number).
- **Amended:** Tasks & Acceptance no longer prescribe `platforms: [.macOS(.v13)]`. Design Notes and Verification now require omitting `platforms:` entirely. swift-tools-version remains the SwiftPM manifest schema (`6.0`), not a duplicate of root `.swift-version` (6.3.3).
- **Known-bad avoided:** Re-deriving a Package.swift that sets `.macOS(.v13)` (or any other deployment floor) before ADR-002 is Accepted.
- **KEEP:** Directory `native/macos/BronzeNative/` with static library product `BronzeNative`; `// swift-tools-version: 6.0` as compatible manifest schema only; `BronzeNative.swift` `@_cdecl("bronze_native_abi_version")` returning `UInt32` `0`; AC-aligned file comment; `**/.build/` in `.gitignore`; no AppKit/AX/event-tap/Tests/Rust façade/sprint-status.yaml/package-local `.swift-version`.

## Design Notes

SwiftPM static library for in-process linking requires the product type: .static. The @_cdecl attribute makes the symbol visible to Rust via extern "C" without requiring a separate C target or modulemap at this stage (full ABI shape in 2-1). No Tests/ directory is created because the AC only requires `swift build`, not `swift test` (later stories add test targets and filters). Keep the package free of any AppKit or ApplicationServices imports to satisfy the explicit non-goal and AGENTS rule. The placeholder symbol name "bronze_native_abi_version" follows the naming direction in later stories (bronze_native_*). Do not declare `platforms:` in Package.swift; ADR-002 remains Proposed and a deployment target must not be encoded here. `swift-tools-version: 6.0` is the required manifest schema compatible with root `.swift-version` 6.3.3, not a second compiler pin.

## Verification

**Commands:**
- `swift build --package-path native/macos/BronzeNative` -- expected: exit 0; produces a static library artifact under native/macos/BronzeNative/.build/... (exact layout SwiftPM-dependent)
- `swift build --package-path native/macos/BronzeNative 2>&1 | cat` -- expected: no errors about missing products or sources
- `find native/macos/BronzeNative -name '*.swift' | sort` -- expected: exactly the Package.swift + Sources/BronzeNative/BronzeNative.swift
- `rg -n 'CGEventTap|AXIsProcessTrusted|NSStatusItem|AppKit|ApplicationServices|NSPasteboard' native/macos/BronzeNative || true` -- expected: no matches
- `rg -n 'platforms:' native/macos/BronzeNative/Package.swift || true` -- expected: no matches
- `python3 tooling/planning-checks.py` -- expected: PASS (requirement parity, links, traceability)
- `npx --yes markdownlint-cli2 'docs/**/*.md' README.md AGENTS.md` -- expected: 0 issues

**Manual checks (if no CLI):**
- Inspect Package.swift for static product declaration, `swift-tools-version` as manifest schema only, and no `platforms:` deployment-target declaration.
- Confirm .build/ (or equivalent) contains a .a or library artifact after build (local only; do not commit).

## Review Triage Log

### 2026-08-27 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 2 (low 2)
- defer: 6
- reject: 11
- addressed_findings:
  - [low] [patch] terminology in BronzeNative.swift comment aligned to AC phrasing ("no event tap, AX, or AppKit window code yet")
  - [low] [patch] added `**/.build/` to .gitignore (SwiftPM hygiene flagged in Code Map; mirrors 1.3 pattern)

### 2026-08-27 — Review pass
- intent_gap: 0
- bad_spec: 1 (medium 1)
- patch: 1 (low 1)
- defer: 0
- reject: 17
- addressed_findings:
  - [medium] [bad_spec] Package.swift `platforms: [.macOS(.v13)]` encoded a macOS minimum while ADR-002 is Proposed; Tasks no longer prescribe `platforms:`; code reverted for re-derive without a deployment-target declaration

### 2026-08-27 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 2 (low 2)
- defer: 0
- reject: 16
- addressed_findings:
  - [low] [patch] Code Map no longer claims Package.swift declares a platform
  - [low] [patch] Verification commands now grep Package.swift for `platforms:`

## Auto Run Result

Summary of implemented change: Follow-up review of the done 1.4 scaffold. Removed `platforms: [.macOS(.v13)]` from `native/macos/BronzeNative/Package.swift` so the skeleton does not encode a macOS deployment minimum while ADR-002 is Proposed. Spec Tasks/Design Notes/Verification updated to omit `platforms:`. `sprint-status.yaml` left untouched (orchestrator-owned).

Files changed with one-line descriptions:
- `native/macos/BronzeNative/Package.swift` — dropped `platforms:`; static library product `BronzeNative` unchanged
- `_bmad-output/implementation-artifacts/1-4-scaffold-swift-package.md` — bad_spec repair, triage, verification grep, Code Map, Auto Run Result

Review findings breakdown: patches applied 2 (low 2); items deferred 0; items rejected 16. Score `3 × 0 medium + 1 × 2 low = 2` (threshold 5) → followup false.

Verification performed:
- `swift build --package-path native/macos/BronzeNative` — exit 0, `libBronzeNative.a` present
- `find native/macos/BronzeNative -name '*.swift' -not -path '*/.build/*'` — Package.swift + Sources/BronzeNative/BronzeNative.swift
- forbidden-token `rg` — only the AC-aligned comment mentions AppKit; no imports
- `rg -n 'platforms:' native/macos/BronzeNative/Package.swift` — no matches
- `python3 tooling/planning-checks.py` — PASS (56 IDs)
- `npx --yes markdownlint-cli2 'docs/**/*.md' README.md AGENTS.md` — 0 issues

Residual risks:
- SwiftPM may still apply an implicit default platform when `platforms:` is omitted; that default is not an ADR-002 decision encoded by this story
- No `nm` proof of `bronze_native_abi_version` export (Epic 2)
- `bronze-platform-macos` not linked yet (story 2-3)
- `sprint-status.yaml` remains dirty in the worktree; orchestrator-owned, not committed or reverted

Follow-up review recommendation: false

