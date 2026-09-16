---
title: 'Story 3.2: Double-tap FSM tests'
type: 'feature'
created: '2026-09-16'
status: 'done'
review_loop_iteration: 0
followup_review_recommended: true
context:
  - _bmad-output/implementation-artifacts/epic-3-context.md
  - AGENTS.md
warnings:
  - oversized
deferred: []
baseline_revision: '5f591706bf9cb06a07fd63e3884faaf57d4e198b'
---

<intent-contract>

## Intent

**Problem:** CAP-002 / A11Y-001 require an optional Shift double-tap whose timing, side rules, and cancel/reset behavior are proven without a live tap. BronzeNative has only the ABI scaffold; there is no pure FSM, so Story 3.3 cannot feed a tested machine and invalid sequences could emit triggers or retain a key stream.

**Approach:** Add a timestamp-injected, Shift-only double-tap FSM in BronzeNative with table and property tests. Default config is disabled; gap config must accept ≥500 ms; one valid pair emits one trigger; invalid sequences emit zero.

## Boundaries & Constraints

**Always:**
- Preserve CAP-002, A11Y-001, ADR-005 in types, tests, and notes.
- P0 modifier is Shift only (`docs/12-settings-and-shortcuts.md` §7). Identify sides with Carbon `kVK_Shift` / `kVK_RightShift` in a mapper; FSM transitions use `left | right`, never raw hex.
- Default `enabled == false`. Defaults: gap 250 ms (allowed 150–900, must accept 500), max hold 400 ms (100–1500), debounce 30 ms, refractory 500 ms (100–1500), side `either`.
- Trigger on the second matching Shift **release**. Inject monotonic nanoseconds (`CGEventTimestamp` shape); never `Date` / wall clock.
- One valid pair → one trigger; any invalidating event → zero for that candidate. Reset clears timestamps and sides.
- Events are `{kind, side?, timeNs}` only. No character, no retained key stream, no logging of keys/content.
- Non-timed routes stay required: do not remove, wrap, or gate ABI/chord/menu/composer on this FSM (A11Y-001).
- Test names or type names include `FSM` so `swift test --package-path native/macos/BronzeNative --filter FSM` selects them.
- Leave `_bmad-output/implementation-artifacts/sprint-status.yaml` untouched.

**Block If:**
- Satisfying an AC would require a live `CGEventTap`, Input Monitoring / TCC grant, or 1,000 physical trials.

**Never:**
- Live event tap, AX, AppKit, pasteboard, ABI additions to `BronzeNative.h`, settings persistence, first-tap feedback, or action mapping (`capture.selection` vs `app.togglePanel`).
- Persist or log keycodes/characters; suppress or mutate a key stream; recommend disabling Secure Input.
- Adopt Proposed ADR-002, ADR-009, or ADR-018. Copy Cooper source or Copper trade dress.
- Write or revert `sprint-status.yaml`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| VALID_PAIR | enabled; either; two same-or-mixed Shift down/up within hold and gap | exactly one trigger on second up | none |
| DISABLED_DEFAULT | `DoubleTapConfig.defaults`; otherwise valid pair | zero triggers; state stays idle | none |
| WINDOW_500 | gapMs=500; interval == 500 ms | one trigger | none |
| HOLD_GAP_BOUNDS | hold==maxHold and gap==gapMs; then hold+1 or gap+1 | equals accept; plus cancel / idle, zero trigger | none |
| INVALID_CANCEL | mid-candidate: non-Shift, other modifier, left+right overlap, missed release, reversed timestamp | zero trigger; return idle | cancel, no retain |
| SIDE_POLICY | `same` with L then R; `left` with right taps | zero | none |
| REFRACTORY | valid pair then immediate second pair before refractory ends | still one trigger | ignore candidates |
| RESET_WAKE | mid-candidate `reset` (wake/disable/settings) then later valid pair | zero until reset; one after a full new pair | none |
| DEBOUNCE_DUP | same side/kind within 30 ms | ignore; candidate remains completable | none |
| NO_STREAM | any sequence then inspect FSM | only current state + at most the live candidate times/sides | no `[keyCode]` / Character history |
| TIME_INJECTED | identical event times after a real-world pause | identical outputs | no wall-clock read |
| PROPERTY_ARBITRARY | generated sequences (sides, other-mod, cancel, equal/reversed/large times, reset) | trigger iff exact accepted pattern; never stuck (reset → idle, empty times) | deterministic |

</intent-contract>

## Code Map

- `native/macos/BronzeNative/Sources/BronzeNative/BronzeNative.swift:1` -- ABI-only; do not add FSM or frameworks here
- `native/macos/BronzeNative/Sources/BronzeNative/include/BronzeNative.h:1` -- C ABI; do not add FSM symbols
- `native/macos/BronzeNative/Package.swift:4` -- existing static lib + `BronzeNativeTests`; new Swift file auto-compiles
- `native/macos/BronzeNative/Tests/BronzeNativeTests/BronzeNativeTests.swift:1` -- ABI tests; leave; `--filter FSM` must not depend on them
- `docs/07-macos-capture-reliability.md:192` -- FSM states, defaults, ten transition rules, property generator (authority)
- `docs/07-macos-capture-reliability.md:152` -- timestamps are `CGEventTimestamp`; Carbon VK for side; no magic numbers
- `docs/12-settings-and-shortcuts.md:59` -- `modifierTap` shape; P0 Shift; gap slider includes 500 ms
- `docs/12-settings-and-shortcuts.md:125` -- default off
- `docs/18-adrs.md:219` -- ADR-005: listenOnly later; second release; no persist/log; disabled default
- `docs/13-testing-quality-release.md:46` -- table/property coverage list (this story: FSM only, not SPSC)
- `docs/04-functional-spec.md:38` -- F-CAP-02; 0.5 s window; non-timed routes
- `docs/03-prd.md:62` -- CAP-002; `docs/03-prd.md:106` -- A11Y-001
- `docs/10-accessibility-conformance-plan.md:29` -- EN 301 549 5.8/5.9
- `_bmad-output/implementation-artifacts/3-2-double-tap-fsm-tests.md` -- original ACs / `swift test --filter FSM`
- `_bmad-output/implementation-artifacts/3-3-listen-only-event-tap-thread.md` -- next consumer; keep `handle`/`reset` tap-free
- `_bmad-output/implementation-artifacts/spec-3-1-permission-snapshot-enum.md` -- prior epic 3 pattern; no Swift in 3.1
- `_bmad-output/implementation-artifacts/sprint-status.yaml` -- orchestrator-owned; never write or revert

## Tasks & Acceptance

**Execution:**
- `native/macos/BronzeNative/Sources/BronzeNative/DoubleTapFSM.swift` -- add `DoubleTapConfig` (defaults + in-range gap/hold/refractory including 500 ms), `ModifierSide` + Carbon keycode mapper, injected-time events (`down`/`up`/`cancel`/`reset`), states matching docs/07 §6, `handle`/`reset`; disabled config ignores input; no stream storage
- `native/macos/BronzeNative/Tests/BronzeNativeTests/DoubleTapFSMTests.swift` -- XCTest type/methods containing `FSM`; table-test every I/O row; property generator over docs/07 §6 event classes and the listed invariants

**Acceptance Criteria:**
- Given the Story 1.4 Swift package, when FSM table/property tests run, then a valid pair emits one trigger and invalid sequences emit zero.
- Given `DoubleTapConfig.defaults`, when read, then the gesture is disabled.
- Given a config whose acceptance window is 500 ms, when a pair uses that interval, then one trigger is emitted.
- Given processed events, when FSM storage is inspected, then no key stream or characters are retained.
- Given A11Y-001, when this story ships, then existing non-timed surfaces are unchanged and unused by the FSM.

## Spec Change Log

## Review Triage Log

### 2026-09-16 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 3: (high 0, medium 1, low 2)
- defer: 0
- reject: 14: (high 0, medium 0, low 14)
- addressed_findings:
  - `[medium]` `[patch]` SIDE_POLICY tests never reached `isAllowedSecondSide` for `.left`/`.right`; added left-then-right and right-then-left reject rows
  - `[low]` `[patch]` VALID_PAIR/TIME_INJECTED only counted triggers; assert first three edges are quiet and the second `up` fires
  - `[low]` `[patch]` idle + `up` had no table row; added `idle-up` to INVALID_CANCEL

### 2026-09-16 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 3: (high 0, medium 2, low 1)
- defer: 0
- reject: 16: (high 0, medium 0, low 16)
- addressed_findings:
  - `[medium]` `[patch]` matching-side release (`side == firstSide` / `secondSide`) had no failing row; added crossed-release first/second cancel rows and a follow-on pair that must stay at zero triggers
  - `[medium]` `[patch]` DEBOUNCE_DUP only duplicated downs; added a same-side `up` inside 30 ms that still completes
  - `[low]` `[patch]` debounce equality at exactly 30 ms was untested; added an ignore-and-complete row at `30 * ms`

### 2026-09-16 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 3: (high 0, medium 1, low 2)
- defer: 0
- reject: 17: (high 0, medium 0, low 17)
- addressed_findings:
  - `[medium]` `[patch]` `DoubleTapConfig` fields were mutable and `debounceMs`/`msToNs` could trap on negative or overflowing values; fields are now `let`, init rejects negative/overflow debounce, `msToNs` saturates
  - `[low]` `[patch]` NO_STREAM only inspected idle storage; now asserts firstDown/firstUp `liveCandidate` times/sides and no key-stream labels mid-gesture
  - `[low]` `[patch]` Removed the no-op `contains(where: { _ in false })` branch in `mirrorContainsKeyStream`

## Design Notes

States: Idle → FirstDown → FirstUp → SecondDown → Triggered → Refractory → Idle. Trigger on second up.

Compare intervals with injected `timeNs` only: hold and gap succeed when `delta <= limit`; `+ 1 ns` fails. Reversed time (`timeNs < last`) cancels.

Debounce table (docs/07 rule 6 — pick one deterministic table): same side and kind within 30 ms → ignore (do not move last timestamp). Any other unexpected transition (other side, opposite kind that is not the legal next edge, both sides down) → cancel to Idle.

```text
enabled=false                    -> ignore, stay Idle
Idle + allowed Shift down        -> FirstDown
FirstDown + same up, hold OK     -> FirstUp
FirstUp + allowed down, gap OK   -> SecondDown
SecondDown + matching up, hold OK -> trigger, Refractory
cancel | timeout | overlap | other-mod | reset -> Idle
Refractory until refractoryNs    -> ignore downs
```

`ModifierSide(carbonKeyCode:)` accepts only `kVK_Shift` / `kVK_RightShift`. Other modifiers arrive as `cancel` with no keycode stored.

## Verification

**Commands:**
- `swift test --package-path native/macos/BronzeNative --filter FSM` -- expected: exit 0; VALID_PAIR, DISABLED_DEFAULT, WINDOW_500, cancel/side/refractory/reset/debounce/no-stream/time-injected/property rows pass
- `swift test --package-path native/macos/BronzeNative` -- expected: exit 0; existing ABI tests still pass
- `rg -n 'Date\\(\\)|CFAbsoluteTime|gettimeofday' native/macos/BronzeNative --glob '!**/.build/**'` -- expected: no FSM wall-clock reads
- `rg -n 'CGEventTap|CGEvent\\.tap' native/macos/BronzeNative --glob '!**/.build/**'` -- expected: no matches

## Auto Run Result

Status: done

Summary: Follow-up review of the already-implemented Shift double-tap FSM (CAP-002 / A11Y-001 / ADR-005). Three patches landed: immutable overflow-safe `DoubleTapConfig`, mid-candidate `liveCandidate` inspection, and a dead Mirror branch removed. No intent gap, no spec re-derivation, no deferred ledger edits.

Files changed:
- `native/macos/BronzeNative/Sources/BronzeNative/DoubleTapFSM.swift` — `DoubleTapConfig` fields are `let`; init rejects negative or overflowing `debounceMs`; `msToNs` is overflow-safe
- `native/macos/BronzeNative/Tests/BronzeNativeTests/DoubleTapFSMTests.swift` — constructor overflow/negative debounce rows; mid-gesture NO_STREAM inspect; dead Mirror continue removed
- `_bmad-output/implementation-artifacts/spec-3-2-double-tap-fsm-tests.md` — this follow-up review pass and result

Review findings this pass:
- patches applied: 3 (high 0, medium 1, low 2)
- items deferred: 0 (existing `deferred: []` left untouched)
- items rejected: 17 (high 0, medium 0, low 17)

Follow-up review recommendation: true (patched high=0, medium=1, low=2; score `3 × 1 + 1 × 2` = 5)

Verification:
- `swift test --package-path native/macos/BronzeNative --filter FSM` — exit 0; 16 tests, 0 failures
- `swift test --package-path native/macos/BronzeNative` — exit 0; 28 tests (12 ABI + 16 FSM), 0 failures
- `rg Date()/CFAbsoluteTime/gettimeofday` on `native/macos/BronzeNative` excluding `.build` — no matches
- `rg CGEventTap|CGEvent.tap` — one pre-existing ABI comment in `BronzeNative.swift` ("No … CGEventTap"); no FSM tap

Residual risks:
- `cancel`/`reset` still clear refractory immediately (intent treats them as wake/disable, not candidates)
- Property soup does not carry a full subsequence oracle (constructed/mutated buckets still assert iff)
- Orchestrator-owned `sprint-status.yaml` was not written or reverted

