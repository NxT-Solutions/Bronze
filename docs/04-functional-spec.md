# Functional specification

Requirement IDs come from [PRD](03-prd.md). “Must” means v1 release gate.

## 1. Core entities

- **Workspace:** local store and global settings; v1 has one workspace.
- **Section:** ordered queue container with title, state, color token, timestamps.
- **Item:** immutable identity plus editable content, BCP 47/`und` content language, lifecycle, order, source, and revision.
- **Capture request:** transient state machine with ID, trigger, target app, timestamps, provider attempts, terminal result.
- **Output profile:** serialization and post-copy behavior.
- **Shortcut:** semantic action, trigger form, key/modifiers or modifier-tap settings, scope.

## 2. Capture flow

### F-CAP-01 Standard chord

1. User selects text in source app.
2. User invokes configured global chord or menu command.
3. Trigger service acknowledges locally within target latency and creates capture request.
4. Trigger ingress snapshots prepublished frontmost external process identity/activation generation, active destination plus accept-capture generation, app-policy/settings revisions, trigger route, and monotonic time before request waits behind older work. Focused element/window identity is acquired only when provider starts because AX work is forbidden in event callback.
5. At provider start, reject expired/changed target or policy, acquire focused element/window identity inside snapshotted target, then query allowed element and bounded ancestors. Empty/missing selection on child is inconclusive. If provider returns stable non-empty selection, persist it.
6. Only after the full ancestor/range provider chain returns truly unsupported or empty, and fallback is enabled, wait until physical modifiers release, then run clipboard fallback.
7. Normalize only line endings according to explicit policy; preserve whitespace/content.
8. In one transaction, require snapshotted destination still exists and accepts capture, then insert item/source/undo state there. If unavailable, do not retarget; return `destination_unavailable` and keep one bounded pending-content token for Retry/Copy/Discard. Publish result from coordinator memory; append content-free diagnostic independently so diagnostic-store failure cannot roll back saved content.
9. Announce “Captured to {section}” and optionally show panel/toast.
10. Return focus according to panel behavior setting.

Acceptance:

- Rapid repeated triggers queue in arrival order. No request disappears.
- Duplicate text is allowed; optional duplicate warning cannot block by default.
- Empty/only-zero-width/unsupported selection returns `no_selection`, leaves store unchanged, and offers manual entry. AX/manual routes do not mutate clipboard; enabled experimental synthetic fallback may already have changed shared pasteboard and performs no restorative write.
- Secure field returns `protected_content`; no content, title, or keystrokes are logged.
- Excluded app returns `app_excluded` before selection retrieval. The exclusion list is `SettingsV1.privacy.excludedBundleIds`, edited in Settings Privacy as a searchable multi-select of installed apps plus a rust-owned Finder `.app` picker when the Applications scan misses one (no free-text bundle dump; no filesystem paths to WebView).
- Stale target/focus/policy or over-age queued request returns `target_changed`/`capture_expired`; it never retargets newer focus or destination section. macOS exposes no immutable trigger-time selection snapshot: within unchanged control, captured text is selection observed at bounded provider-read time. This limitation is documented and covered by change-during-capture tests.

### F-CAP-02 Modifier double tap

Disabled by default for first-run accessibility neutrality; onboarding may offer it.

State machine accepts two releases of configured left/right modifier when each hold and pair interval are within user-adjustable windows and no non-modifier key occurred. It resets on other key, timeout, tap disable/re-enable, permissions change, sleep/wake, session switch, keyboard attach, and impossible flag transition. First tap may show optional subtle feedback.

Always expose standard chord, menu-bar command, and manual entry. Timing UI supports at least 0.5-second acceptance window, matching EN 301 549 clause 5.8 design signal.

### F-CAP-03 Clipboard fallback

Fallback must:

1. Record pasteboard `changeCount`; P0 does not snapshot or restore prior clipboard content.
2. Wait until modifiers are physically up.
3. Send exact Command down, C down/up, Command up with guaranteed cleanup on all exits.
4. Poll `changeCount` with bounded adaptive backoff up to configurable internal ceiling (initial target 2 s).
5. Read supported textual types without writing a sentinel.
6. Require a stable post-copy generation during read. If another mutation races, retry within deadline or fail without writing to pasteboard.
7. Leave copied selection on clipboard and explain that OS clipboard history, Universal Clipboard, or third-party managers may retain it.
8. Report timeout, injection denial, unsupported format, or concurrent mutation separately.

## 3. Manual composer

### F-QUE-01 Add

- Composer is multiline by default.
- `⌘Enter` adds; configurable plain Enter adds only when single-line mode selected. This avoids IME and accidental submission problems.
- During composition (`isComposing`), Enter never submits.
- Add preserves content, assigns active section and next order key, defaults content language to `und` unless user explicitly chooses a valid canonical BCP 47 tag, saves transactionally, clears composer only after success, and announces result.
- Failure retains input and offers retry/copy.

### F-QUE-02 Edit

- Enter/explicit Edit opens inline or dedicated editor.
- Escape cancels after confirmation only when unsaved changes would be lost.
- Revisions permit undo and conflict-safe update.
- Markdown is stored as source text; preview sanitizes and never fetches remote resources.
- Editor exposes optional content-language override. Bronze does not silently detect language; changing metadata never translates or normalizes body text.

## 4. Selection, ordering, and lifecycle

### F-QUE-03 Select

- Single click focuses/selects according to platform convention.
- `⌘Click`, Shift range, and Select All supported.
- Checkbox-like multi-select control exposes `aria-checked`; completion control is separate to avoid ambiguity.
- Selection count is announced and visible.

### F-QUE-04 Reorder

- Drag/drop, keyboard move (`⌥⌘↑/↓` default only if conflict-free), and Move menu produce same command.
- Live region announces final position: “Moved item 2 of 5 to position 4.”
- Ordering uses stable gapped/fractional keys with periodic transactional rebalance.

### F-QUE-05 Copy

Copy command builds immutable output from selected item IDs in visible queue order. Profiles:

- plain: items joined by chosen separator;
- bullet Markdown: `- item` with continuation indentation;
- numbered Markdown: stable sequence;
- prompt block: configurable headings, source label policy, separators.

Profile management lives in Copy settings. User can create, duplicate, rename, edit, reset built-ins, and delete custom profiles. Format options are validated discriminated data, never executable templates: bounded separators, Markdown marker/start rules, literal prompt headings/delimiters, and provenance-label policy. Default profile cannot be deleted until replacement is selected. Locale change never silently rewrites stored custom or seeded output literals.

Captured context is untrusted downstream text. Default prompt-block profile labels and deterministically delimits/quotes context separately from user-authored instructions, escaping its own delimiter without changing stored raw item. Exact assembled-output preview is available before copy. This reduces accidental prompt injection but does not make hostile text safe; Bronze never auto-pastes or executes it.

After successful pasteboard write, selected output profile alone applies `postCopyAction` (`unchanged`, `copied`, `active`, or `done`) and `advancePolicy` (`keep` or `nextQueued`) atomically with command receipt. Built-in default uses `copied` plus `keep`; explicit Copy and Advance uses `nextQueued`. Optional return focus targets external app active immediately before Bronze activation, never item provenance, and never synthesizes paste. Failure changes no lifecycle or focus progression.

### F-QUE-06 Complete, skip, trash, undo

- Complete is explicit button/menu/shortcut and reversible.
- Skip removes from active progression but preserves item.
- Trash creates tombstone and retains for configured period.
- Undo stack records command plus inverse; cross-session recovery covers at least most recent destructive operation.
- Empty Trash requires explicit confirmation and cannot cascade-delete non-trashed descendants.

## 5. Sections and search

- Active section receives capture/manual adds.
- Section archive hides it from active panel but not search/export.
- Move section/item commands validate destination and preserve identity.
- Search uses local FTS; quoted phrase and field filters can be P1. Results expose section and status.
- Search query itself never leaves device or enters diagnostics.

## 6. Window behavior

Panel modes:

- **summon:** show near configured edge on pointer/frontmost-app display;
- **pinned:** remain visible across focus changes;
- **auto-hide:** hide after blur delay unless dialog/menu/edit is active.

Position uses visible work area, not raw screen bounds. Persist explicit physical `left|right|top` edge/width, not direction-relative leading/trailing or absolute coordinates alone. Localized UI may describe edge, but RTL never mirrors stored physical side. On display removal or scale change, clamp fully into available work area. Join all Spaces/full-screen only when platform behavior tested and user enables it. Restore prior app focus after capture-only flow; do not steal focus for passive success toast.

## 7. Permissions and diagnostics

Permission center shows each capability independently:

| Capability | Why | Test |
| --- | --- | --- |
| Input Monitoring | observe configured global modifier trigger | receive known self-test trigger without recording content |
| Accessibility | read selected text and/or synthesize copy | AX focused-element query and explicit sample capture |
| Launch at Login | optional startup | platform registration status |

Statuses: `unknown`, `not_requested`, `denied`, `granted_unverified`, `healthy`, `degraded`, `unavailable`, `requires_relaunch`. “Granted” is not “healthy.” User can retest; failures link to specific System Settings guidance and safe alternative.

Diagnostics event fields match the persisted `diagnostic_events` schema: timestamp, request ID, stage, result code, duration, trigger kind, provider kind, permission state, source bundle ID only when policy permits, app schema version, queue depth, overflow count, tap health, store result code, and build ID. Never selected content, window title, URL, clipboard payload, keystroke stream, or secret.

## 8. Import/export and recovery

- Export writes new atomic directory/archive; never silently overwrites.
- Manifest includes schema/export version, creation time, checksum list, locale-independent timestamps, sections/items with content-language metadata, settings-safe subset, and assets.
- Import validates schema, sizes, hashes, paths, UTF-8/Unicode, and duplication strategy before transaction.
- Preview shows additions/conflicts and never executes embedded content.
- Backup restore first snapshots current DB, verifies integrity, migrates copy, then swaps atomically.

## 9. Error model

Every command returns typed code, user-safe localized message key, retryability, and optional diagnostic ID. UI never displays raw Rust/SQLite/path errors. Fatal store errors put app in read-only recovery mode with export/support options; they do not keep accepting unsaved captures.

## 10. State and concurrency invariants

1. One serial capture coordinator owns request order.
2. One store writer serializes migrations and mutations; reads may use separate connection pool under WAL.
3. UI receives typed domain events and applies targeted cache updates; no global refetch after each mutation.
4. Native callback holds no UI/store locks and performs no blocking work.
5. IPC trusts neither window nor payload; authorization and validation occur at boundary.
6. Item order is total and deterministic within section.
7. Content is never silently trimmed, normalized, translated, or sent over network.
