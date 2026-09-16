# Epic 3 Context: Capture coordinator and AX-first selection

<!-- Generated from planning artifacts. Regenerate with compile-epic-context if planning docs change. -->

## Goal

Every observed trigger gets a request ID and exactly one terminal outcome; Accessibility (AX) is the primary selection provider; synthetic clipboard fallback stays off. This epic makes capture reliable, observable, and fail-closed so later UI can show health and feedback without silent drops, retargeting, or secret leakage.

## Stories

- Story 3.1: Permission snapshot enum
- Story 3.2: Double-tap FSM tests
- Story 3.3: Listen-only event tap thread
- Story 3.4: Atomic ingress snapshot
- Story 3.5: Serial capture coordinator
- Story 3.6: AX provider with fakes
- Story 3.7: Manual clipboard import
- Story 3.8: Content-free diagnostic schema
- Story 3.9: TCC grant deny revoke evidence
- Story 3.10: Signed AX source matrix

## Requirements & Constraints

- Capture is deliberate selected text only — never screenshot, screen recording, ambient keylogging, or passive clipboard history. Do not request Screen Recording, microphone, or camera.
- Every observed trigger is queued, gets a monotonic ID, is processed once, and ends as `saved | rejected | failed | cancelled` with a reason. Queue overflow still emits a terminal result per missing ID. Zero silent drops. Local trigger ack target is ≤100 ms p95; AX capture-to-saved ≤300 ms p95.
- Success feedback must not fire before the persist hook for a supported result. Diagnostic write failure must not roll back a saved item; keep bounded in-memory terminal status.
- AX is primary. Provider chain: exclusion → AX selected text/range → optional later user-enabled synthetic copy (after shared-clipboard disclosure, stable bundle ID) → explicit Create from Clipboard → manual composer. In this epic, synthetic Cmd-C stays off. Never restore the pasteboard. Never ingest files or images. Stale clipboard generation fails closed.
- Secure Input, password fields, and unknown protection fail closed with zero content in ABI, logs, store, or diagnostics. Run exclusion before any content query. Never bypass Secure Input.
- Preserve exact Unicode and intentional leading/trailing whitespace. Empty or zero-width selection is no-selection without trimming.
- Ingress snapshot is immutable: never retarget to a later frontmost app. Inconsistent seqlock read is unavailable. Event-tap snapshot must not include the focused AX element. No titles or URLs in ingress.
- Permission health is a closed enum: `unknown | not_requested | denied | granted_unverified | healthy | degraded | unavailable | requires_relaunch`. Granted is not healthy. Unknown platform results map to unknown/degraded, never healthy. Wrap preflight APIs but do not prompt; prompts only after a labeled Enable action. Treat Input Monitoring, Accessibility, and capture-pipeline self-test as independent. Denial leaves chord, menu, and manual routes usable.
- Optional modifier double-tap is off by default, never the only path, timing window ≥500 ms, and must not retain the key stream. Every core operation has a non-timed route.
- Diagnostics expose permission, trigger, provider, fallback, persist, and feedback stages only. Schema must reject string payloads for selected text. No selected/item/clipboard text, key stream, titles, URLs, AX trees, user paths, or hashes of secrets.
- Supported-source reliability (including the 99.9% matrix) is human-evidence-backed. Do not count unknown apps. Stories 3.9 and 3.10 stay `blocked-human-validation`; do not fabricate TCC, signed-capture, or AX-matrix evidence.
- Proposed ADRs 002, 009, and 018 remain unresolved; do not adopt them silently.

## Technical Decisions

- One serial capture coordinator in Rust. CGEventTap on a dedicated CFRunLoop thread; AX on a dedicated serial queue; AppKit on main. Event-tap callback does no AX, DB, window, clipboard, IPC, allocation-heavy, or logging work. SPSC is single-producer. Disabling the tap resets the FSM. Never suppress or mutate the key stream; do not persist keycodes.
- Standard route is a registered global chord. Optional double-tap uses a session `listenOnly` tap on `flagsChanged` (plus a non-Shift cancel signal when robust cancellation is enabled). Trigger on the second valid modifier release. Pure FSM: one valid pair → one trigger; invalid sequences → zero; no wall-clock dependence.
- Call direction stays WebView → IPC → Rust → versioned C ABI → in-process Swift. Domain crates do not import macOS frameworks. WebView never sees raw AX, event-tap, or pasteboard APIs.
- Ingress is a seqlock snapshot: target PID, bundle token, activation generation, destination UUID, accept-capture generation, policy/settings revisions, route, monotonic time.
- Clipboard fallback policy is `manual | syntheticExperimental | off`. If synthetic copy is ever enabled later, it writes nothing after the copy and does not restore prior clipboard.
- Diagnostics are content-free at the type/schema boundary. Allowed: request/diagnostic IDs, timestamps, stage durations, trigger/provider/result enums, permission snapshot, build/schema version, policy-gated bundle ID, health counters.
- Permission snapshots are a typed model for later health UI — no keys or content. Locale-neutral enums; UTC millisecond timestamps; no user content in logs.

## UX & Interaction Patterns

- Capture-only success keeps source focus; do not show the panel before the capture-target snapshot.
- Every terminal result needs visible plus announced feedback (native announcement when Bronze is inactive). Failures use a stable diagnostic ID and a recovery path — never source names or content.
- Permission-late: explain why, offer retest / System Settings / Use Manual Capture. Screen Recording is unused. Denial must leave the app usable.
- “Capture” never means screenshot, recording, or keylogging. No ambient capture animation. Timing-only capture is banned.
- Protected field: private non-content error. No selection: recoverable manual/note path, never silent.

## Cross-Story Dependencies

- Within epic: 3.2 → 3.3 → 3.4 → 3.5; 3.6, 3.7, and 3.8 depend on 3.5; 3.9 on 3.1–3.3; 3.10 on 3.6.
- Depends on Epic 1 crates/Swift package and Epic 2 linked façade (3.3 needs 2.3; 3.4 needs 2.2).
- Enables Epic 5 capture-only focus/announcement (needs 3.5), Epic 7 permission health UI (needs 3.1), Epic 9 diagnostics help (needs 3.8) and the human-validation backlog (needs 3.9–3.10).
