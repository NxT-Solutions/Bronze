# Story 9.1: Reproducible debug packaging

Status: done

## Story

As a developer,
I want a documented local package command with checksums,
So that builds are repeatable.

**Requirements:** SEC-005
**ADRs:** ADR-011, ADR-002 Proposed
**Dependencies:** Story 1.8
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** verify passes
**When** packaging script is added
**Then** it records arch as arm64 unless DG-01 says otherwise
**And** SBOM generation stub or cargo/pnpm list captured
**And** no get-task-allow in release config

**Failure / recovery:**
Do not notarize or use Apple Developer credentials.

**Security / privacy / diagnostics / a11y / i18n / data:**
No network updater.

**Automated verification:**
- `pnpm verify`
- `test -f tooling/package-debug.sh`

**Signed-build / manual evidence:**
local script output

**Non-goals:**
notarization, Developer ID, universal2


## Dev Agent Record

### Agent Model Used

Hedgehog authored loop / layer-eng (cursor-grok).

### File List

- `apps/desktop/src-tauri/src/packaging.rs` — arm64 until DG-01, SBOM stub, no get-task-allow
- `apps/desktop/src-tauri/entitlements/macos.release.plist` — sandbox false, no get-task-allow
- `docs/13-testing-quality-release.md` — local package command
- `tooling/package-debug.sh` — join-owned script

### Notes

- ADR-002 / DG-01 remain Proposed. No notarization. Script path is join-owned.
