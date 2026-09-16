# macOS capture reliability

Version: 1.0 planning baseline  
Status: implementation and release contract  
Related: [PRD](03-prd.md), [functional specification](04-functional-spec.md), [system architecture](06-system-architecture.md), [ADRs](18-adrs.md)

## 1. Objective

Reliable means:

- Every trigger observed by Bronze ingress receives request ID. Secure Input, revoked Input Monitoring, OS suppression, or conflicting global shortcut can make gesture unobservable; health/status and non-monitoring alternatives expose that limitation without pretending unseen input existed.
- Every request reaches exactly one terminal result.
- Supported-source success reaches G-01 target of at least 99.9 percent.
- No normal keyboard event is consumed or modified.
- Secure/password content is never queried or captured.
- No selected content or keystroke enters diagnostics.
- P0 clipboard fallback performs no automatic restoration and writes nothing after its synthetic copy.
- Failure is visible, localized, announced, and recoverable.

This document implements CAP-001 through CAP-010, G-01, G-02, SET-002 through SET-004, SEC-006, A11Y-001, A11Y-006, and F-CAP-01 through F-CAP-03.

## 2. Provider policy

Provider order:

1. App exclusion and protected-content guard.
2. macOS Accessibility selected-text provider.
3. If enabled, bounded synthetic-copy provider.
4. Explicit manual clipboard import.
5. Manual composer.

AX is primary. Synthetic copy is disabled until user understands clipboard and simulated-input effects. Manual paths remain available regardless of Input Monitoring, timed gesture, or source compatibility.

P0 captures plain text. It preserves Unicode scalar content, line breaks, indentation, and intentional leading/trailing whitespace. No trim, NFC/NFD normalization, translation, smart-quote conversion, Markdown parsing, or remote lookup occurs.

Empty string and selection consisting only of configured zero-width formatting/control characters produce no_selection. Ordinary whitespace-only selection remains content; do not implement emptiness using trim.

## 3. Permission model

Permissions are independent:

| Capability | API/preflight | Needed for | Not needed for |
| --- | --- | --- | --- |
| Input Monitoring/listen-event access | CGPreflightListenEventAccess and CGRequestListenEventAccess | passive double-modifier event tap | manual composer, status menu, registered chord |
| Accessibility trust | AXIsProcessTrustedWithOptions (prompt option on request) | AX selected-text query; synthetic input path where required | manual composer; explicit clipboard import |
| Screen Recording | never called (no CGRequestScreenCaptureAccess, CGPreflightScreenCaptureAccess, or NSScreenCapture) | not used (ADR-001) | every Bronze capture and settings path |
| Global shortcut registration | Tauri global-shortcut result | standard chord | status menu/manual composer |
| Launch at Login | platform registration API | optional startup | all capture behavior after manual launch |

Permission state is a closed enum, not a single linear path. Valid values:

    unknown | not_requested | denied | granted_unverified | healthy | degraded | unavailable | requires_relaunch

“Granted” never implies healthy. Health requires operation-level self-test. Permission service (SET-003, SET-004, CAP-003, CAP-010, ADR-001, ADR-005):

- On native runtime start, requests Accessibility (`AXIsProcessTrustedWithOptions` with `kAXTrustedCheckOptionPrompt`) and Input Monitoring (`CGRequestListenEventAccess`) when preflight is not already granted. Requests run off the event-tap callback thread.
- On the first capture path, repeats that request once if still ungranted. Launch is not a prompt loop.
- Permission-health Retest always calls those request APIs again. macOS may refuse a second Input Monitoring dialog; Accessibility may re-prompt on some OS versions.
- If the OS will not re-prompt after that attempt, a System Settings deep-link is the fallback — after the request, not instead of it. Bronze never auto-opens Settings and never deep-links Screen Recording.
- Never requests Screen Recording. Health UI keeps Screen Recording as Not used.
- Explains why each used capability is needed and what remains if it is denied. Denial leaves the manual composer.
- Rechecks when app becomes active, settings opens, wake occurs, or provider reports denial.
- Recreates event tap after newly granted Input Monitoring.
- Detects revocation through failed operation/preflight and degrades without crash.
- Never repeatedly prompts on launch.
- Never claims Bronze can override macOS or Secure Keyboard Entry.

Accessibility reference: [AXIsProcessTrustedWithOptions](https://developer.apple.com/documentation/applicationservices/1459186-axisprocesstrustedwithoptions). Listen-event references: [CGPreflightListenEventAccess](https://developer.apple.com/documentation/coregraphics/cgpreflightlisteneventaccess%28%29), [CGRequestListenEventAccess](https://developer.apple.com/documentation/coregraphics/cgrequestlisteneventaccess%28%29).

## 4. Trigger paths

### 4.1 Standard chord

CAP-001 uses Tauri global-shortcut plugin inside privileged Rust core for ordinary configurable accelerators. WebViews have no raw plugin registration/listen capability; they call typed Bronze settings commands. Registration transaction:

1. Parse candidate into semantic shortcut schema.
2. Reject Bronze-internal duplicate, invalid modifier-only form, unsupported key, and known reserved chord.
3. Attempt new registration while old registration remains active where API permits.
4. Offer user-ended/adjustable Test mode (recommended initial window 5–30 seconds) with visible Stop/Skip; user invokes candidate without timing pressure.
5. Save setting after registration. Mark it tested only after successful invocation; skipping retains an explicit untested warning rather than blocking a Slow Keys/motor user.
6. Unregister old chord after new chord commits.
7. On any failure, roll back candidate and retain old registration.

Tauri isRegistered reports this application’s registration state; another app/system owner may still make chord unusable. Test mode is release behavior, not diagnostics-only. [Tauri global shortcut](https://v2.tauri.app/plugin/global-shortcut/), [global-shortcut JavaScript API](https://v2.tauri.app/reference/javascript/global-shortcut/)

Shortcut storage:

~~~text
action
trigger kind: accelerator | modifier_double_tap | disabled
modifier set
key mode: physical | logical
physical code or logical key
modifier side when relevant
double-tap timing
schema version
~~~

Display glyph/name recomputes from current input source. Spoken localized form accompanies glyphs. Keyboard layout change invalidates display cache and reruns ambiguity warning.

### 4.2 Modifier double tap

CAP-002 is optional and disabled on first run. Onboarding may offer it after standard route works. It is an efficiency enhancement, never sole route.

Implementation uses CoreGraphics event tap:

- Location: session event tap.
- Option: listenOnly.
- Placement: tail append unless compatibility spike proves different need.
- Event mask: flagsChanged plus optional keyDown solely to cancel a candidate.
- Callback always returns incoming event unchanged.
- No character translation.
- No key stream retention.
- No event suppression.

Apple APIs: [CGEvent tap creation](https://developer.apple.com/documentation/coregraphics/cgevent/tapcreate%28tap%3Aplace%3Aoptions%3Aeventsofinterest%3Acallback%3Auserinfo%3A%29), [listenOnly](https://developer.apple.com/documentation/coregraphics/cgeventtapoptions/listenonly), [flagsChanged](https://developer.apple.com/documentation/coregraphics/cgeventtype/flagschanged).

### 4.3 Status menu

CAP-003 menu path must retain target even if opening status menu activates Bronze:

- Native target tracker observes NSWorkspace application activation.
- lastExternalPID updates whenever non-Bronze app becomes active.
- Opening status menu loads/publishes ingress target before any Bronze activation.
- Capture command consumes that snapshot; it never substitutes whichever app becomes frontmost afterward.
- Capture persists AX selected text into the same queue store as the composer, then announces a content-free saved result. It does not reveal or focus the Quick Panel (WIN-003 capture-only).
- Status-item left-click is Show. The status menu is Show, Capture, Settings, and Quit. Seeded shortcuts, including the double-modifier gesture, stay disabled; Capture is the status-menu and Bronze-menu command.
- Capture stores CAP-008 app-name provenance from the focused process (`proc_name` for the AX element's PID). Bronze / `bronze-desktop` is omitted. URL and window title are not stored. Provenance failure never fails a valid text capture.
- The inbox shows catalog `capture.source` (`From {appName}`) on captured rows that have a name. Composer rows have no source line.
- If target exited, return target_lost and open manual composer.

### 4.4 Manual routes

- New Note opens empty multiline composer.
- Create from Clipboard reads current text only after explicit action.
- Both work without Input Monitoring or Accessibility.
- Cmd-Enter submits except during IME composition.

### 4.5 Trigger-time ingress context

Native target tracker and Rust coordinator prepublish fixed-size `CaptureIngressContext`: external target PID, interned bundle-identity token, application activation generation, destination UUID plus accept-capture generation, app-policy/settings revisions, and context generation. Update uses seqlock or equivalent atomic snapshot protocol. Every invoked/observed trigger path loads stable context and assigns request ID/monotonic time before request can wait behind older work. Modifier event-tap path writes fields into preallocated channel record; standard-chord and menu callbacks submit same immutable record schema through serialized coordinator entry and never produce into event-tap SPSC channel. If consistent context cannot be read within bounded attempts, request terminates `context_unavailable`.

Event-tap callback performs no NSWorkspace, AX, database, allocation, or string work. Focused element/window identity cannot be captured safely there; provider acquires it later inside snapshotted process, then enforces age and identity revalidation. Status-menu path snapshots last external target before Bronze activation. Standard chord, modifier gesture, and menu use same ingress contract.

## 5. Event-tap runtime

### 5.1 Thread

Swift creates CFMachPort and CFRunLoopSource on dedicated thread. Run loop:

1. Install tap.
2. Signal readiness.
3. Receive minimal event set.
4. Feed preallocated FSM state.
5. Push only Trigger with fixed-size ingress context, Reset, or TapDisabled records to bounded single-producer channel; after each enqueue attempt, release-publish completed-ingress sequence atomically.
6. On shutdown, disable tap, remove source, drain/cancel, release objects.

Callback restrictions:

- p99 below 1 ms.
- No main-thread dispatch awaited.
- No AX or NSWorkspace call.
- No database, filesystem, Tauri event, translation, or log.
- No dynamic string construction.
- No lock that UI/capture coordinator may hold.
- No panic/exception across C callback.

Channel capacity is finite for safety, but CAP-004 forbids unaccounted loss. This SPSC channel has exactly one producer: dedicated event-tap thread. Other trigger routes must not write it. Every recognized tap trigger first receives source-scoped contiguous monotonic sequence; opaque request ID is reconstructible from prepublished session/route prefix plus sequence without allocation. Producer attempts bounded enqueue, then release-stores sequence as `ingressCompletedSequence`; successful records carry same request ID/sequence. Consumer acquire-loads completion watermark, drains visible records through that watermark, and only then reconciles missing sequences, synthesizing one terminal `trigger_queue_overflow` receipt with reconstructed request ID per gap. Release/acquire ordering makes successful enqueue visible before corresponding completion watermark. Tail timer runs same watermark-then-drain handshake so trailing overflow resolves even when no later trigger arrives; it never guesses from elapsed time alone. Coordinator merges event-tap, standard-chord, and menu records into deterministic total order by monotonic trigger time plus documented route/sequence tie-break. Wraparound and concurrent producer/consumer interleavings require model/property tests. Never use one `AtomicBool` or aggregate-only counter that hides which requests failed.

### 5.2 Timestamp and key identity

Use CGEventTimestamp, elapsed nanoseconds since system startup, for all gesture intervals. Never use wall clock. [CGEventTimestamp](https://developer.apple.com/documentation/coregraphics/cgeventtimestamp)

Use public Carbon virtual-key constants for left/right modifier identity; no magic numbers in domain code. Track left and right state separately. App-generated synthetic copy events carry private source tag and cannot enter modifier FSM.

### 5.3 Disable and recovery

Event tap can receive tapDisabledByTimeout or tapDisabledByUserInput:

1. Reset FSM and all pressed state.
2. Record content-free health event.
3. Attempt CGEventTapEnable once when permission remains granted.
4. If still disabled, tear down and recreate after bounded backoff.
5. Mark degraded after repeated failure; keep chord/menu.
6. Never busy-loop.

Reset/reinstall also occurs after:

- sleep/wake;
- fast-user/session switch;
- screen lock/unlock where observable;
- keyboard device attach/detach;
- input-source change;
- permission change;
- impossible flag transition;
- app update/relaunch.

Reference: [tap-disabled timeout event](https://developer.apple.com/documentation/coregraphics/cgeventtype/tapdisabledbytimeout).

## 6. Double-tap FSM

Default settings:

| Setting | Default | Allowed |
| --- | ---: | ---: |
| Inter-tap gap | 250 ms | 150–900 ms |
| Maximum press hold | 400 ms | 100–1,500 ms |
| Minimum transition debounce | 30 ms | internal |
| Refractory period | 500 ms | 100–1,500 ms |
| Side | either, same side not required | left, right, either, same side |

Settings UI supports at least 500 ms acceptance window. Timed gesture is optional because A11Y-001 requires non-timed routes.

~~~mermaid
stateDiagram-v2
  [*] --> Idle
  Idle --> FirstDown: allowed Shift down
  FirstDown --> FirstUp: same Shift up within max hold
  FirstDown --> Idle: timeout or invalid event
  FirstUp --> SecondDown: allowed Shift down within gap
  FirstUp --> Idle: gap timeout or invalid event
  SecondDown --> Triggered: matching Shift up within max hold
  SecondDown --> Idle: non-Shift key, overlap, timeout
  Triggered --> Refractory
  Refractory --> Idle: refractory expires
~~~

Transition rules:

1. Trigger on second release, not second press. Prevents panel opening before knowing second Shift was not ordinary capitalization.
2. Any non-modifier keyDown cancels current candidate. Event’s character/keycode is not retained.
3. Any disallowed modifier transition cancels.
4. Left/right simultaneous hold cancels unless future explicitly tested mode allows it.
5. Held Shift beyond threshold cancels.
6. Duplicate/noisy transition inside debounce is ignored or cancels according to deterministic transition table.
7. Same-side setting requires both taps use same physical side.
8. Refractory ignores new candidates, preventing key bounce/duplicate trigger.
9. Reset clears all timestamps and sides.
10. FSM emits one trigger for one valid sequence.

Secure Keyboard Entry may suppress observation. Bronze honors this and never recommends disabling it. Standard chord/status/manual routes remain. [Apple Secure Keyboard Entry](https://support.apple.com/en-il/guide/terminal/trml109/mac)

Property-test generator must cover arbitrary sequences of:

    left/right modifier down/up
    other modifier down/up
    non-modifier key down/up
    timestamps including equal, reversed, large gap
    reset
    tap disabled/re-enabled
    sleep/wake

Properties: at most one trigger per valid pair, zero trigger after invalidating event, no stuck state, deterministic output, no effect from wall time.

## 7. Capture request state machine

~~~mermaid
stateDiagram-v2
  [*] --> Queued
  Queued --> ResolvingTarget
  ResolvingTarget --> Rejected: excluded, no target, queue policy
  ResolvingTarget --> QueryingAX
  QueryingAX --> Persisting: allowed non-empty selection
  QueryingAX --> WaitingModifiers: fallback enabled and eligible
  QueryingAX --> Failed: protected, denied, timeout, target lost
  WaitingModifiers --> SynthesizingCopy
  SynthesizingCopy --> WaitingClipboard
  WaitingClipboard --> Persisting: stable text generation
  Persisting --> Saved
  Persisting --> Failed
  Queued --> Cancelled
  ResolvingTarget --> Cancelled
  QueryingAX --> Cancelled
  WaitingModifiers --> Cancelled
  SynthesizingCopy --> Cancelled
  WaitingClipboard --> Cancelled
~~~

Terminal states:

    saved | rejected | failed | cancelled

Invariants:

- Request ID is assigned at trigger ingress.
- State transition is append-only and monotonic.
- Exactly one terminal state.
- Repeated request ID returns existing receipt/result.
- Rapid triggers queue in arrival order.
- Duplicate selected text is valid and creates separate item.
- Panel/toast cannot imply saved before database commit.
- Store read-only state rejects before content acquisition where possible.
- Cancellation after synthetic key-down must run key-up cleanup before terminal transition.

## 8. Target resolution

CaptureIngressSnapshot:

~~~text
request ID
target PID
interned bundle-identity token
application activation generation
active destination section ID and accept-capture generation
app-policy and settings revision
trigger timestamp
source route
~~~

ProviderContext adds focused element/window identity token only after dequeue and before content query.

Resolution:

1. Reject missing/inconsistent ingress snapshot as `context_unavailable`; never substitute current frontmost app.
2. Reject `capture_expired` when age exceeds internal all-provider ceiling, initially 1 second; ceiling is bounded release configuration, not unbounded user setting.
3. Resolve snapshotted target PID; verify process exists and bundle token/activation generation still match.
4. Revalidate app policy. Any policy-revision change rejects `policy_changed`; newly relaxed policy never escalates queued request, and newly restrictive policy takes effect immediately.
5. Apply CAP-009 exclusion before AX content query.
6. Capture AX application/focused element/window identity as provider-time context.
7. Verify snapshotted destination still exists and accepts capture; never replace it with current active section.
8. Never show/focus quick panel before AX result or fallback decision.

If target changes while request waits in queue, request uses ingress-snapshotted target, not newest frontmost app. If target terminated, it fails target_lost rather than capturing from different app.

macOS AX does not expose immutable selection snapshot tied to trigger timestamp. Bronze bounds queue age and revalidates target, focus, range/text, and policy, but a selection can change and return between observations. Product language promises capture of selection observed at provider-read time from original target, not unknowable trigger-time bytes.

## 9. AX provider

### 9.1 Algorithm

1. Check Accessibility permission.
2. Create AX application element for target PID.
3. Obtain focused UI element.
4. Read role and subrole only.
5. Build bounded, cycle-safe chain from focused element through ancestors.
6. Before any content query at each node, classify role/subrole as protected, allowed text, neutral container, or unknown content-bearing. Fail closed for `kAXSecureTextFieldSubrole`, known password/protected equivalents, or unknown content-bearing state. Unknown protection prohibits both AX content query and synthetic fallback, regardless of app category.
7. At each allowed node, query `kAXSelectedTextAttribute`. Empty or missing child value is inconclusive; continue to ancestor.
8. At each node, if direct selection is unavailable/empty, query `kAXSelectedTextRangeAttribute` and `kAXStringForRangeParameterizedAttribute` when supported.
9. Stop on first allowed non-empty selection. Only full-chain exhaustion returns no selection/unsupported.
10. Validate result type, UTF-8 conversion, byte/grapheme limit, process/focused-element identity, age, and request generation. Where selected range exists, require same range before/after text read; otherwise re-read selected text once within budget and require stable result.
11. Query optional provenance only under CAP-008 policy.
12. Return exact Unicode string content and whitespace supplied by provider plus safe metadata; do not claim original source-encoding bytes.

References: [selected text](https://developer.apple.com/documentation/applicationservices/kaxselectedtextattribute), [selected text range](https://developer.apple.com/documentation/applicationservices/kaxselectedtextrangeattribute), [secure text subrole](https://developer.apple.com/documentation/applicationservices/kaxsecuretextfieldsubrole), [AX attributes](https://developer.apple.com/documentation/applicationservices/carbon_accessibility/attributes).

### 9.2 Timeout and retry

- AX runs off event-tap and main UI thread.
- Call [`AXUIElementSetMessagingTimeout`](https://developer.apple.com/documentation/applicationservices/1459345-axuielementsetmessagingtimeout) on target application element before synchronous queries; deadline is native, not only a late-result check. If a call still wedges/fails, abandon affected worker generation and recreate provider queue without blocking later captures.
- Overall target budget supports G-02 300 ms p95.
- Only kAXErrorCannotComplete receives bounded retry, suggested 20 ms then 50 ms.
- Permission denial, unsupported attribute, invalid element, process exit, illegal argument, and protected field do not retry.
- Each callback checks request generation; late result is discarded.
- Repeated timeout marks app/provider degraded locally but does not blacklist permanently.

### 9.3 Content guard

Secure-field policy is fail closed:

- Check focused element and bounded ancestors before selected/value query.
- Classifier returns one closed enum: `protected`, `allowed_text`, `neutral_container`, or `unknown_content_bearing`. Known application/window/group/scroll containers are neutral and traversed without querying content; known non-secure text controls are allowed; password/secure equivalents are protected.
- If a relevant content-bearing role/subrole cannot be classified, return `protection_unknown`; do not query content or synthesize copy. Never treat an unfamiliar editable/browser/custom control as safe merely because it lacks `kAXSecureTextFieldSubrole`.
- protected_content contains no title, text length, hash, or excerpt.
- Never query kAXValueAttribute as substitute for selection.
- Password managers, authentication dialogs, terminal Secure Input, and browser password fields belong in mandatory negative matrix.

### 9.4 Provenance

This build records the focused process name on capture (CAP-008 app name). URL and window title stay off. Remaining source-identity fields stay opt-in globally and overridable per bundle policy:

- bundle ID and localized app name;
- safe window/document title;
- URL when source exposes supported public attribute and user enabled it.

Rules:

- CAP-009 exclusion runs before content/provenance read.
- Per-app provenance disable overrides global setting.
- Never scrape browser UI hierarchy to infer URL in P0.
- No PID persists.
- Diagnostics may include bundle ID only when diagnostics policy allows; never title/URL.
- Provenance failure never fails valid text capture.

### 9.5 Size and Unicode

Set explicit maximum selection payload, planning default 1 MiB UTF-8. Over-limit behavior is typed selection_too_large with manual copy/export route; no silent truncation. Conversion:

- Accept valid CFString.
- Preserve line endings unless explicit user policy selected.
- Preserve leading/trailing whitespace.
- Reject malformed bridge UTF-8.
- Never slice inside UTF-8 scalar or grapheme cluster.
- No normalization by default; policy/version must accompany future normalization.

## 10. Clipboard fallback

### 10.1 Safety policy

Synthetic fallback is bounded, user-enabled, per-bundle allowlisted, and off by default. Unknown app/control policy never falls back automatically. Manual clipboard import is always offered. P0 does not snapshot or restore previous clipboard contents: `changeCount` is only a generation signal and cannot provide atomic compare-and-swap or prove which process wrote a change.

NSPasteboard is shared state. changeCount detects intervening mutation but does not make restoration transactional. [NSPasteboard](https://developer.apple.com/documentation/appkit/nspasteboard), [changeCount](https://developer.apple.com/documentation/appkit/nspasteboard/changecount).

### 10.2 Transaction

~~~mermaid
sequenceDiagram
  participant C as Capture coordinator
  participant P as Pasteboard
  participant S as Source app

  C->>C: wait for all physical modifiers up
  C->>S: activate snapshotted target
  C->>C: revalidate target, focus, policy, age
  C->>P: read c0 immediately before injection
  C->>S: Command down, C down/up, Command up
  loop bounded backoff
    C->>P: read changeCount
  end
  P-->>C: stable c1 greater than c0
  C->>P: read preferred textual representation
  C->>P: recheck stable c1
  alt unchanged during read
    C->>C: persist text; leave copied selection on pasteboard
  else concurrent mutation
    C->>C: retry within deadline or fail without pasteboard write
  end
~~~

Steps:

1. Reject when request age exceeds internal all-provider ceiling (initial target 1 second) or snapshotted target/focused element/policy/destination cannot be revalidated.
2. Wait until Command, Shift, Option, Control, and configured modifier are physically released.
3. Activate original target; verify PID, bundle identity, activation generation, focused element/window, app policy, and protection classification again.
4. Read `changeCount` immediately before injection, after activation side effects settle.
5. Post exact sequence: Command down, C down, C up, Command up.
6. A scope guard posts missing key-up events on cancellation/error.
7. Tag synthetic events so Bronze event tap ignores them.
8. Poll `changeCount` using bounded adaptive backoff, ceiling initially 2 seconds.
9. Require new stable generation; source apps may write more than once.
10. Read allowlisted textual representation only, with strict byte/time caps; never snapshot images, files, custom/lazy types, or unbounded items.
11. Recheck generation after read. If it changed, retry within deadline or return `clipboard_changed`.
12. Treat result as best-effort, not causally proven: pasteboard exposes no source-PID ownership token. Wrong-content rate must remain zero in supported matrix or bundle stays denied.
13. Persist item to snapshotted destination only after valid stable text result.
14. Leave captured selection on pasteboard and show disclosed status when experimental fallback is enabled.

No sentinel, unconditional clear, snapshot, or automatic restoration. Because synthetic copy enters shared pasteboard, macOS Clipboard History, Universal Clipboard, and third-party clipboard tools may retain/distribute it. Settings/help must say so before enablement.

### 10.3 Text type policy

Preference:

1. public plain UTF-8/string representation.
2. Other textual UTI convertible locally to plain text.
3. RTF converted locally to plain text only when enabled.
4. Nontext-only result returns clipboard_unsupported_type.

Do not ingest files/images in P0. Do not execute, render HTML, resolve URLs, or fetch remote resources.

### 10.4 Secure Input and injection denial

Secure Keyboard Entry or target policy may prevent monitoring/synthetic copy. Treat this as expected provider failure:

- Run key-up cleanup.
- Make no clipboard restoration write.
- Return injection_denied or clipboard_timeout.
- Offer explicit manual copy.
- Never request user disable security feature.

## 11. Persistence and feedback

Successful provider result commits:

1. New item in snapshotted destination section with lifecycle queued and next stable rank; transaction first verifies section still exists and accepts capture. Archive/trash/delete never causes silent retarget.
2. Source/provenance under snapshotted trigger-time policy.
3. Undo entry when relevant.

Content transaction must not depend on diagnostics. After commit, coordinator publishes terminal result from memory and separately attempts typed diagnostic append. If diagnostic persistence fails, bounded in-memory health ring/status still reports it; saved content remains saved. If content transaction fails, diagnostic failure is also representable in memory even when database is full/corrupt. Only after content commit:

- emit scoped item-created event;
- show panel/toast according to setting;
- announce “Captured to section”;
- restore focus according to WIN-003;
- clear pending sensitive buffers.

If transaction fails, content remains only in bounded in-memory retry buffer long enough to offer Retry or Copy. UI must not clear user-visible text until retry/copy succeeds. Fatal store error enters read-only recovery and rejects new captures.

## 12. Typed outcomes

Stable internal result codes:

| Stage | Codes |
| --- | --- |
| Trigger | trigger_permission_denied, shortcut_conflict, trigger_queue_overflow, tap_disabled, secure_input_unavailable |
| Target/destination | no_target, context_unavailable, target_lost, app_excluded, target_changed, policy_changed, capture_expired, destination_unavailable |
| AX | accessibility_denied, focused_element_missing, protected_content, protection_unknown, ax_unsupported, ax_cannot_complete, ax_timeout, invalid_ax_value |
| Selection | no_selection, selection_too_large, invalid_text_encoding |
| Clipboard | modifiers_stuck, injection_denied, clipboard_timeout, clipboard_unsupported_type, clipboard_changed |
| Store | store_read_only, store_locked, store_full, store_corrupt, persist_failed |
| Request | cancelled, duplicate_request, internal_invariant_violation |

Every command/result carries:

- code;
- localized message key;
- retryability;
- recovery action enum;
- optional diagnostic ID;
- no raw framework/SQLite/path error in user message.

Provider results distinguish unsupported, permission, protected, timeout, target loss, and internal failure. Do not collapse to capture_failed.

## 13. Diagnostics

Allowed fields:

~~~text
timestamp
request ID
build and schema version
trigger kind
stage names and durations
provider result enums
permission snapshot
source bundle ID only when policy permits
store result enum
tap health counters
queue depth/overflow counter
~~~

Forbidden:

- selected text or excerpt;
- text length when protected-content result;
- clipboard payload, types that expose document identity, or fingerprint;
- raw keycode/character stream;
- window/document title;
- URL;
- AX element description/tree;
- filesystem path containing username/document;
- database statements with bound content.

Support bundle is local, generated on request, previewed before export, and includes no automatic upload.

## 14. Failure and recovery matrix

| Failure | Detection | Required recovery |
| --- | --- | --- |
| Input Monitoring denied/revoked | preflight/tap creation or health failure | disable modifier gesture; retain chord/menu/manual |
| Accessibility denied/revoked | trust check/AX error | manual clipboard/composer; clear explanation |
| Secure Input | known signal or expected observation/injection failure | capture nothing; retain non-hook routes |
| Tap timeout/user disable | tapDisabled event/health check | reset FSM; re-enable/recreate with backoff |
| Callback queue pressure | overflow counter | terminal overflow result; visible degraded health; no silent drop |
| Ordinary Shift typing | non-Shift key cancellation | remain idle; no panel |
| Sleep/wake impossible flags | workspace/power notification | reset state and reinstall if needed |
| Target exits | process/bundle verification | target_lost; manual composer |
| AX unsupported | typed attribute error | configured fallback/manual route |
| AX transient cannot complete | typed error | bounded 20/50 ms retry |
| Secure AX control | role/subrole guard | protected_content; zero content/provenance |
| Selection oversized | byte/grapheme validation | no truncation; manual route |
| Modifiers never release | physical-state timeout | cancel fallback; guarantee key-up cleanup |
| Pasteboard no change | deadline | clipboard_timeout; make no further Bronze write and disclose that synthetic copy may already have changed shared pasteboard |
| Concurrent clipboard write | generation changes during read | bounded retry or fail; no Bronze write/restoration follows synthetic copy |
| Promised/custom pasteboard type | representation classification | reject unsupported representation; never attempt restoration |
| Store disk full/locked | SQLite result | no success feedback; retry/copy/read-only recovery |
| Permission changed after update | signed-upgrade health test | guided retest; preserve manual flow |

## 15. Compatibility evidence

Support status is evidence, not assumption. Maintain versioned test manifest per OS/app build:

~~~text
OS version and build
Mac architecture
source app name, version, bundle ID
content surface: native text, web text, editor, terminal, PDF, secure
AX path result
fallback result
Unicode corpus result
latency distribution
known limitation and recovery
evidence date/build
~~~

Mandatory source matrix:

- TextEdit.
- Notes.
- Safari.
- Chrome.
- Firefox.
- Cursor and VS Code/Electron.
- Slack or comparable Electron text surface.
- Terminal and at least one third-party terminal.
- Preview/browser PDF selectable text.
- Password field and authentication dialog negative tests.
- Source with empty selection.
- Source that exits during query.

An app is “supported” only when named versions meet release corpus and latency target through documented provider path. Unknown/custom controls are “best effort,” never counted toward 99.9 percent denominator.

## 16. Accessibility and input compatibility

Core journey must work without double tap:

    status menu or standard chord
      → capture/manual fallback
      → edit
      → save
      → copy
      → complete

Test:

- VoiceOver on/off.
- Full Keyboard Access.
- Voice Control.
- Switch Control.
- Sticky Keys.
- Slow Keys.
- alternate layouts including non-US.
- CJK and Indic IME.
- keyboard repeat and long holds.
- external keyboard attach/detach.
- left/right modifier-specific configuration.

Shortcut recorder must not capture VoiceOver commands as candidate without explicit recording mode, and recording mode must expose Cancel/Reset by pointer and keyboard.

## 17. Performance gates

| Stage | Gate |
| --- | ---: |
| Event-tap callback | p99 below 1 ms |
| Trigger acknowledgment | p95 at or below 100 ms |
| Target resolution | p95 below 20 ms |
| AX capture and persistence total | p95 at or below 300 ms |
| Synthetic clipboard fallback | p95 at or below 2 s |
| Capture queue | zero silent drops; stable under 100 rapid synthetic triggers |
| Supported-source success | at least 99.9 percent |
| 24-hour event-tap soak | zero disabled tap, duplicate trigger, stuck modifier, leak trend |

Instrumentation uses monotonic stage timestamps and reason codes only. Benchmark report includes OS build, app versions, hardware, power state, sample count, p50/p95/p99, failures, and warm/cold distinction.

## 18. Acceptance suite

Release-blocking tests:

1. Run 1,000 valid modifier pairs per side/mode: exactly one request each, zero duplicates.
2. Run held Shift, capitalization, rapid mixed modifiers, both Shifts, ordinary typing, bounce corpus: zero false triggers.
3. Feed 100 rapid triggers: arrival order preserved; every ID terminal.
4. Enable Secure Keyboard Entry: no key/text capture; menu/manual route works.
5. Revoke Input Monitoring during run: gesture degrades without crash; chord/menu remain.
6. Revoke Accessibility during run: request returns typed denial; no repeated prompt.
7. Capture Unicode corpus in every supported source: Unicode scalar string matches provider result under documented line-ending policy; whitespace preserved.
8. Focus secure field: no content, title, length, hash, or clipboard change.
9. Exclude app: rejection occurs before AX content/provenance query.
10. Kill target during AX query: target_lost; no capture from newly frontmost app.
11. Force kAXErrorCannotComplete: exactly bounded retries, then typed failure.
12. AX unsupported: configured fallback/manual instruction; no unrelated or stale clipboard content persists as item.
13. Clipboard timeout: key-up cleanup runs; Bronze makes no further pasteboard write and reports observed postcondition without claiming original clipboard survived.
14. Concurrent clipboard mutation during read: retry/fail; Bronze performs no restoration write.
15. Clipboard history/Universal Clipboard disclosure appears before experimental fallback enablement.
16. Disk full after provider success: no success announcement; Retry/Copy retains content safely.
17. Shortcut conflict: old chord remains registered; new value not saved.
18. Sleep/wake during first tap: FSM resets; no trigger after wake from stale state.
19. VoiceOver/Full Keyboard Access journey succeeds without timing gesture.
20. Signed upgrade on clean test machine preserves or clearly re-establishes TCC health under stable identity.
21. Delay request, change focus/selection/destination/policy, and cross age ceiling: changed identity expires/rejects; unchanged-control read follows documented provider-read-time semantics; snapshotted destination remains fixed, while archived/trashed/deleted destination returns `destination_unavailable` with bounded Retry/Copy/Discard token.
22. Model ingress producer/consumer interleavings, queue-full gaps, trailing overflow, and sequence wraparound: every recognized trigger ID receives exactly one terminal receipt.

## 19. Primary sources

- [CGEvent tap creation](https://developer.apple.com/documentation/coregraphics/cgevent/tapcreate%28tap%3Aplace%3Aoptions%3Aeventsofinterest%3Acallback%3Auserinfo%3A%29)
- [CGEvent listen-only option](https://developer.apple.com/documentation/coregraphics/cgeventtapoptions/listenonly)
- [Listen-event permission preflight](https://developer.apple.com/documentation/coregraphics/cgpreflightlisteneventaccess%28%29)
- [Listen-event permission request](https://developer.apple.com/documentation/coregraphics/cgrequestlisteneventaccess%28%29)
- [CGEvent flagsChanged](https://developer.apple.com/documentation/coregraphics/cgeventtype/flagschanged)
- [CGEvent timestamp](https://developer.apple.com/documentation/coregraphics/cgeventtimestamp)
- [Tap disabled by timeout](https://developer.apple.com/documentation/coregraphics/cgeventtype/tapdisabledbytimeout)
- [Accessibility trust](https://developer.apple.com/documentation/applicationservices/1459186-axisprocesstrustedwithoptions)
- [AX selected text](https://developer.apple.com/documentation/applicationservices/kaxselectedtextattribute)
- [AX selected-text range](https://developer.apple.com/documentation/applicationservices/kaxselectedtextrangeattribute)
- [AX secure text-field subrole](https://developer.apple.com/documentation/applicationservices/kaxsecuretextfieldsubrole)
- [Apple Secure Keyboard Entry](https://support.apple.com/en-il/guide/terminal/trml109/mac)
- [NSPasteboard](https://developer.apple.com/documentation/appkit/nspasteboard)
- [NSPasteboard changeCount](https://developer.apple.com/documentation/appkit/nspasteboard/changecount)
- [NSWorkspace frontmost application](https://developer.apple.com/documentation/appkit/nsworkspace/frontmostapplication)
- [Tauri global shortcut](https://v2.tauri.app/plugin/global-shortcut/)
