---
title: 'Story 3.1: Permission snapshot enum'
type: 'feature'
created: '2026-09-16'
status: 'done'
review_loop_iteration: 0
followup_review_recommended: false
context:
  - _bmad-output/implementation-artifacts/epic-3-context.md
  - AGENTS.md
warnings:
  - oversized
deferred: []
baseline_revision: '7f21bc484a7e11348696202e6f75784d50ae70b9'
---

<intent-contract>

## Intent

**Problem:** Health UI, capture diagnostics, and later TCC evidence need a closed permission model (SET-003, SET-004, CAP-010), but both crates are empty scaffolds. There is no typed state, no snapshot, and no no-prompt preflight wrap, so “granted” could be treated as healthy and unknown platform results could be misread as success.

**Approach:** Add a locale-neutral closed `PermissionState` plus a content-free `PermissionSnapshot` in `bronze-settings`, and wrap Input Monitoring / Accessibility preflight in `bronze-platform-macos` so probes never prompt. Map platform bits through explicit rules; do not implement health UI, System Settings links, or BronzeNative ABI permission ops.

## Boundaries & Constraints

**Always:**
- Preserve SET-003, SET-004, CAP-010, ADR-005, ADR-015 in types, tests, and notes.
- Closed states only: `unknown | not_requested | denied | granted_unverified | healthy | degraded | unavailable | requires_relaunch`.
- Snapshot capabilities are independent: Input Monitoring, Accessibility, capture-pipeline self-test. Granted is never healthy. Unknown/error platform results map to `unknown` or `degraded`, never `healthy`.
- Preflight wrap calls only listen-event preflight and accessibility trusted-check with prompt disabled. Prompts only exist later behind a labeled Enable path (not this story).
- Snapshots hold no selected text, clipboard, titles, URLs, keycodes, key stream, paths, tokens, or secret hashes. No user-facing strings (I18N-001).
- `cargo test -p bronze-settings permission` and `cargo test -p bronze-platform-macos permission` must match by test/module name.
- Leave `_bmad-output/implementation-artifacts/sprint-status.yaml` untouched.

**Block If:**
- Implementation would require a human TCC grant, System Settings click, signing identity, or vendor console to satisfy an AC.

**Never:**
- System Settings deep-link UI, onboarding copy, or permission health center (Story 7.3).
- `CGRequestListenEventAccess`, `AXIsProcessTrustedWithOptions` with prompt true, launch-loop prompting, Screen Recording / mic / camera.
- Treat a boolean granted/trusted bit as `healthy`. Emit `healthy` only when a caller supplies an explicit self-test success (this story may construct the variant in tests; preflight mapping must not).
- Import macOS frameworks from `bronze-settings` or any crate except `bronze-platform-macos`.
- Add BronzeNative/Swift ABI permission operations, event tap, AX content query, or clipboard.
- Adopt Proposed ADR-002, ADR-009, or ADR-018. Copy Cooper source or Copper trade dress.
- Write or revert `sprint-status.yaml`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| PREFLIGHT_BOTH_TRUE | listen=true, ax=true | IM and AX = `granted_unverified`; self-test stays `unknown` | none |
| PREFLIGHT_BOTH_FALSE | listen=false, ax=false (no prompt issued) | IM and AX = `not_requested` | none |
| GRANTED_NOT_HEALTHY | any true preflight bit | never `healthy` | mapper has no granted→healthy path |
| PLATFORM_UNKNOWN | probe `Err` / unrecognized | that capability = `unknown` | never `healthy` |
| PLATFORM_PROBE_FAIL | known probe failure (not a bool) | that capability = `degraded` | never `healthy` |
| INDEPENDENT_STATES | IM true, AX false | IM `granted_unverified`, AX `not_requested` | none |
| EXPLICIT_DENIED | snapshot field set `denied` | stored `denied`; not rewritten to healthy | none |
| SELFTEST_ONLY_HEALTHY | self-test set `healthy` by explicit test result; IM/AX from preflight | only self-test may be `healthy` | preflight fields unchanged |
| CLOSED_SET | exhaust 8 variants | each names exactly the planning-checks list | no extra variants |
| NO_CONTENT | snapshot fields / Debug | capability + `PermissionState` only | compile/test rejects String content fields |
| NO_PROMPT_SURFACE | public wrap API | no prompt flag; no request-access symbol | tests use fake host only |

</intent-contract>

## Code Map

- `bronze-settings/src/lib.rs:1` -- scaffold comment only; add `mod permission` and re-export types
- `bronze-settings/Cargo.toml:1` -- keep dependency-free; types stay pure Rust
- `bronze-platform-macos/src/lib.rs:1` -- scaffold; add `mod permission`; still no AX content query
- `bronze-platform-macos/Cargo.toml:1` -- add path dep `bronze-settings = { path = "../bronze-settings" }`
- `Cargo.toml:1` -- workspace already lists both crates (read-only)
- `docs/07-macos-capture-reliability.md:40` -- independent capabilities + closed enum + preflight-without-prompt (read-only)
- `docs/12-settings-and-shortcuts.md:206` -- health statuses are not booleans; Screen Recording not used (read-only)
- `docs/03-prd.md:70` -- CAP-010; `docs/03-prd.md:121` -- SET-003/SET-004 (read-only)
- `docs/18-adrs.md:219` -- ADR-005 listen-only tap needs Input Monitoring; `docs/18-adrs.md:599` -- ADR-015 snapshot allowed, content forbidden (read-only)
- `docs/06-system-architecture.md:207` -- settings owns SET-* types; platform-macos is façade / no business rules (read-only)
- `docs/06-system-architecture.md:232` -- ABI “permission snapshot and request” is a later bridge duty; do not implement here (Epic 2 unfinished; 3.1 depends on 1.3 only)
- `docs/06-system-architecture.md:259` -- full-stack pure Rust FFI rejected; this story wraps two preflight bools behind a host trait, tests never hit TCC
- `tooling/planning-checks.py:36` -- canonical 8 permission names (read-only; keep Rust variant serde/display names snake-aligned if you add string forms)
- `native/macos/BronzeNative` -- no permission symbols; do not add Swift this story (read-only)
- `_bmad-output/implementation-artifacts/3-1-permission-snapshot-enum.md` -- original story ACs (read-only)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` -- orchestrator-owned; never write or revert

## Tasks & Acceptance

**Execution:**
- `bronze-settings/src/permission.rs` -- create closed `PermissionState`, content-free `PermissionSnapshot` `{ input_monitoring, accessibility, capture_pipeline_self_test }`, and pure mappers (`from_preflight_granted`, `from_platform_unknown`, `from_platform_probe_failure`); `is_healthy` true only for `healthy` -- typed model for later health UI
- `bronze-settings/src/lib.rs` -- `mod permission; pub use permission::{PermissionSnapshot, PermissionState};` -- public crate surface
- `bronze-settings/src/permission.rs` -- unit tests whose names include `permission` covering every I/O row that is crate-pure (closed set, granted≠healthy, unknown/degraded, independence, no content) -- AC verification filter
- `bronze-platform-macos/Cargo.toml` -- path-depend on `bronze-settings` -- façade maps into the typed model
- `bronze-platform-macos/src/permission.rs` -- `PreflightHost` with `listen_event_access` and `accessibility_trusted` returning `Result<bool, PreflightError>`; `snapshot_from_preflight` uses settings mappers; `MacosPreflightHost` wraps `CGPreflightListenEventAccess` and `AXIsProcessTrusted` / `AXIsProcessTrustedWithOptions` with prompt forced false; do not expose request-access -- wrap without prompting
- `bronze-platform-macos/src/lib.rs` -- `mod permission` and re-export wrap + snapshot helper
- `bronze-platform-macos/src/permission.rs` -- unit tests whose names include `permission` using a fake host for remaining I/O rows (both true/false, independent, unknown, probe fail, no-prompt surface) -- never invoke the live host in tests

**Acceptance Criteria:**
- Given the workspace crates from Story 1.3, when `bronze-settings` is built, then `PermissionState` is a closed enum of exactly those eight values and `PermissionSnapshot` exposes three independent capability fields with no content/key slots.
- Given a true listen-event or AX trusted preflight bit, when mapped, then the capability is `granted_unverified` and `is_healthy()` is false.
- Given an unrecognized or failed platform probe, when mapped, then the capability is `unknown` or `degraded` and never `healthy`.
- Given the platform wrap, when unit tests run, then only preflight/trusted-check entry points exist, prompt is never enabled, and no test requires TCC UI.
- Given SET-003, when Automation and Launch at Login are absent from P0, then they are omitted from this snapshot; Screen Recording is not a capability.
- Given CAP-010 / ADR-015, when a snapshot is formatted or inspected, then it cannot carry selected text, keys, titles, URLs, or paths.

## Spec Change Log

## Review Triage Log

### 2026-09-16 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 0
- defer: 0
- reject: 17: (high 0, medium 0, low 17)
- addressed_findings:
  - none

### 2026-09-16 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 1: (high 0, medium 0, low 1)
- defer: 0
- reject: 22: (high 0, medium 0, low 22)
- addressed_findings:
  - `[low]` `[patch]` cfg-gate the `MacosPreflightHost` re-export in `bronze-platform-macos/src/lib.rs` so the crate surface does not name a macos-only type on other targets

## Design Notes

Boolean macOS preflight cannot express the full enum. Map only what the bit means; keep other variants constructible for later revoke/self-test/relaunch paths.

```text
Ok(true)  -> granted_unverified   # never healthy
Ok(false) -> not_requested        # this story never prompts, so false ≠ denied
Err(Unknown) -> unknown
Err(Fail)    -> degraded
healthy      -> explicit self-test result only
```

`AXIsProcessTrusted()` is the no-prompt entry; if `WithOptions` is used, prompt must be false. Do not add a prompt parameter to the Rust wrap.

## Verification

**Commands:**
- `cargo test -p bronze-settings permission` -- expected: exit 0; closed enum, granted≠healthy, independence, no-content tests pass
- `cargo test -p bronze-platform-macos permission` -- expected: exit 0; fake-host mapping + no-prompt surface; live TCC not required
- `cargo fmt --check` -- expected: exit 0
- `cargo clippy -p bronze-settings -p bronze-platform-macos -- -D warnings` -- expected: exit 0
- `python3 tooling/planning-checks.py` -- expected: PASS (docs unchanged unless a drift you introduced)

## Auto Run Result

Status: done

Summary of implemented change: Follow-up review of the already-shipped permission snapshot enum. `bronze-settings` still owns the closed `PermissionState` and content-free `PermissionSnapshot`; `bronze-platform-macos` still wraps no-prompt Input Monitoring / Accessibility preflight through `PreflightHost` and settings mappers. This pass applied one patch: cfg-gate the `MacosPreflightHost` re-export.

Files changed with one-line descriptions:
- `bronze-settings/src/permission.rs` — closed eight-state enum, three-field snapshot, pure mappers, `permission_*` tests
- `bronze-settings/src/lib.rs` — re-export `PermissionState` / `PermissionSnapshot`
- `bronze-platform-macos/src/permission.rs` — `PreflightHost`, `snapshot_from_preflight`, macos-only live host, fake-host tests
- `bronze-platform-macos/src/lib.rs` — re-export wrap types; `MacosPreflightHost` is `cfg(target_os = "macos")`
- `bronze-platform-macos/Cargo.toml` / `Cargo.lock` — path dependency on `bronze-settings`
- `_bmad-output/implementation-artifacts/epic-3-context.md` — compiled epic context
- `_bmad-output/implementation-artifacts/spec-3-1-permission-snapshot-enum.md` — this spec and review record
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — orchestrator-owned row; not written or reverted by this run

Review findings breakdown:
- patches applied: 1 low (`MacosPreflightHost` re-export cfg)
- items deferred: 0 (existing `deferred: []` left unchanged)
- items rejected: 22 low (sprint-status orchestrator bookkeeping; live FFI always returns a Boolean; later merge/timestamp/serde/ABI/health-UI/capability-enum/diagnostics; false→`not_requested` by contract; constructible `denied`/`healthy`/`requires_relaunch`; test denylist extras)

Follow-up review recommendation: false (patched this pass: high 0, medium 0, low 1; score `3 × 0 + 1 × 1 = 1`, threshold 5)

Verification performed:
- `cargo test -p bronze-settings permission` — exit 0; 8 passed
- `cargo test -p bronze-platform-macos permission` — exit 0; 7 passed
- `cargo fmt --check` — exit 0 after rustfmt reorder of the cfg re-export
- `cargo clippy -p bronze-settings -p bronze-platform-macos -- -D warnings` — exit 0
- `python3 tooling/planning-checks.py` — PASS (requirement-parity, enum/schema, local-links, traceability)

Residual risks:
- Live `MacosPreflightHost` maps every FFI Boolean through `!= 0` and never returns `PreflightError`; unknown/degraded remain injectable only via `PreflightHost`.
- Boolean preflight still cannot express `denied` / `requires_relaunch`; those variants stay constructible for later stories.
- `_bmad-output/implementation-artifacts/sprint-status.yaml` stays orchestrator-owned and was not included in this run's commit.

