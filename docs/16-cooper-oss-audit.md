# Cooper OSS audit

Status: research baseline  
Audited: 2026-08-27  
Repository: [TouchMyBar/cooper](https://github.com/TouchMyBar/cooper)  
Scope: current repository, release history, macOS capture path, floating-window behavior, persistence/export, security, accessibility, tests, licensing, and reusable implementation lessons for Bronze

## Executive conclusion

Cooper is useful as a compact prototype and failure corpus. It is not a sound production foundation for Bronze.

Strong ideas worth retaining:

- Tauri with a Rust/native boundary and React UI.
- Local SQLite persistence.
- A modifier-only macOS event tap instead of character-decoding keyboard libraries.
- Main-thread execution for macOS window and synthetic-input operations.
- Retry and health checks for a disabled event tap.
- Relocation/quarantine diagnosis before asking for macOS privacy permissions.
- Accessory-app, tray, single-instance, autostart, and native-vibrancy patterns.

Core parts requiring clean reimplementation:

- Permission model.
- Trigger state machine and capture queue.
- Selected-text acquisition.
- Clipboard fallback and preservation.
- Global shortcut registration and settings.
- Floating panel placement and Spaces behavior.
- Database migrations, asset model, backup, and export.
- Tauri command trust boundaries and CSP.
- React state architecture.
- Accessibility and internationalization.
- Testing, signing, and release supply chain.

Current app contains multiple deterministic reasons for apparently missed keystrokes or captures: it silently drops capture attempts while busy, fires fallback capture before physical modifiers are released, uses the same chord for two competing actions, depends on a fixed 700 ms clipboard window, handles only one of the macOS permissions required by its architecture, and reports listener health rather than end-to-end capture health.

## 1. Verified repository snapshot

Facts as of 2026-08-27:

- Repository created 2026-08-03.
- Default branch: `main`.
- Current `main`: [`14ff307023b0`](https://github.com/TouchMyBar/cooper/commit/14ff307023b0e94d48a209db578a3e411bee7d10), authored 2026-08-26.
- Current release: [`v0.3.5`](https://github.com/TouchMyBar/cooper/releases/tag/v0.3.5), published 2026-08-26.
- Repository metadata at audit time: 8 stars, 1 fork, 2 open issues, 1 listed contributor, 1 unprotected branch.
- License: Apache-2.0.
- Latest three-platform packaging workflow completed successfully: [Actions run 32929537255](https://github.com/TouchMyBar/cooper/actions/runs/32929537255).

Primary metadata endpoints:

- [Repository API](https://api.github.com/repos/TouchMyBar/cooper)
- [Commits API](https://api.github.com/repos/TouchMyBar/cooper/commits?per_page=20)
- [Releases API](https://api.github.com/repos/TouchMyBar/cooper/releases?per_page=20)
- [Issues API](https://api.github.com/repos/TouchMyBar/cooper/issues?state=all&per_page=100)

Version history matters when reproducing user reports:

- `v0.2.1` used `rdev` on macOS and could terminate with `EXC_BREAKPOINT` / `SIGTRAP` when keyboard input reached HIToolbox. See [issue #3](https://github.com/TouchMyBar/cooper/issues/3).
- `v0.3.0` compiled the raw listener out on macOS. Fallback chords remained, but double-Shift did not work there.
- `v0.3.5` introduced the custom Core Graphics event tap and related setup flow.

Therefore, failure reports must record exact installed version and binary identity. An older Cooper build and current `main` have materially different macOS behavior.

Maturity signals:

- Issue #3 remains open after the release claims it is fixed.
- Maintainer describes Cooper as a side project, says progress may be slow, and says they are not looking to accept outside contributions: [issue #2 response](https://github.com/TouchMyBar/cooper/issues/2#issuecomment-5384321155).
- Release has a universal macOS build, but hosted CI can prove compilation and packaging only. It cannot exercise TCC permissions, other applications' accessibility trees, actual keyboard devices, pasteboard behavior, Spaces, or window focus.
- Current macOS fix commit documents physical verification on macOS 12.7.4 Intel. This is not coverage for Apple silicon and modern macOS releases.

## 2. Architecture and dependencies

### 2.1 Repository shape

Cooper is a single-package application, not a monorepo.

- Frontend: React 18, TypeScript, Vite 5.
- Desktop shell: Tauri 2.
- UI styling: one plain CSS file; no shadcn/ui, Radix, Tailwind, or design-system package.
- Package manager state: npm `package-lock.json`.
- No Turborepo, Biome, ESLint, Prettier, router, frontend state library, i18n framework, or JavaScript test runner.

Sources:

- [package.json](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/package.json)
- [Cargo.toml](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/src-tauri/Cargo.toml)
- [tauri.conf.json](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/src-tauri/tauri.conf.json)
- [repository tree](https://api.github.com/repos/TouchMyBar/cooper/git/trees/main?recursive=1)

Locked versions observed during audit include:

- `@tauri-apps/api` 2.11.1
- `@tauri-apps/cli` 2.11.4
- React / React DOM 18.3.1
- TypeScript 5.9.3
- Vite 5.4.21
- Tauri crate 2.11.5
- rusqlite 0.31.0
- arboard 3.6.1
- enigo 0.2.1
- image 0.25.10
- window-vibrancy 0.6.0
- rdev 0.5.3 on non-macOS targets

Manifest hygiene defect: root entry in `package-lock.json` still says version `0.3.0`, while `package.json`, `Cargo.toml`, and Tauri configuration say `0.3.5`.

### 2.2 Startup and native modules

[`main.rs`](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/src-tauri/src/main.rs) performs:

1. Tauri single-instance, autostart, global-shortcut, and dialog plugin setup.
2. Command registration.
3. App-data directory creation.
4. SQLite initialization.
5. In-memory `CaptureParent` setup for branch mode.
6. macOS accessory activation policy.
7. Tray creation.
8. Fixed fallback global shortcut registration.
9. Non-macOS `rdev` listener or macOS custom event tap.
10. Initial panel positioning and optional vibrancy.

Rust responsibilities:

- `capture.rs`: capture serialization guard, fallback chords, synthetic copy, clipboard polling.
- `mac_tap.rs`: macOS double-Shift event tap and state machine.
- `mac_setup.rs`: translocation, read-only volume, quarantine, and permission diagnosis.
- `commands.rs`: every Tauri command, export, image IO, Obsidian integration, and settings writes.
- `db.rs`: schema, migrations, queries, serialized application state.
- `panel.rs`: show, hide, toggle, focus, position.
- `tray.rs`: menu, autostart, quit, clipboard capture.
- `glass.rs`: native vibrancy/acrylic.

Frontend responsibilities:

- [`Panel.tsx`](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/src/components/Panel.tsx): most application behavior, including search, grouping, composer, keyboard handling, selection, overlays, paste/drop, menus, update banner, branch mode, and rendering.
- [`store.ts`](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/src/store.ts): IPC wrappers and full-state refresh hook.
- [`keybinds.ts`](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/src/keybinds.ts): in-app shortcut parsing and formatting.
- Leaf components: item card, context menu, section switcher, settings, shortcut sheets, editor, drawing window.

### 2.3 Data flow

SQLite is treated as source of truth. Each mutation:

1. invokes a Rust command;
2. writes SQLite;
3. emits `refresh`;
4. causes `useAppState` to fetch the complete `AppState`;
5. replaces React state and rerenders the panel tree.

There is no normalized frontend cache, incremental query, optimistic update, pagination, or refresh coalescing. Rust and TypeScript models are handwritten mirrors, so a backend serialization change can break runtime behavior without a cross-language compile error.

Implementation lesson for Bronze: retain authoritative persistence, but use generated IPC types and granular queries/events. Avoid a full database snapshot after every action.

## 3. Exact macOS capture pipeline

Current `v0.3.5` path:

1. A session-level, listen-only `CGEventTap` subscribes to `FlagsChanged` and `KeyDown`.
2. Device-dependent modifier bits distinguish left Shift (`0x2`) and right Shift (`0x4`).
3. A press/release counts as a tap when held for less than 500 ms and not marked dirty by another key or modifier.
4. Two completed taps of the same side count when the second release occurs less than 400 ms after the first release.
5. Left pair calls `capture_selection`; right pair calls panel toggle.
6. Capture uses a global `AtomicBool` to allow only one in-flight attempt.
7. A worker thread creates an `arboard` clipboard, saves only its plaintext representation, and writes a zero-width sentinel string.
8. It sleeps 60 ms to let Shift release settle.
9. It schedules enigo Cmd+C generation on the macOS main thread and waits up to 3 seconds for that operation.
10. It polls plaintext clipboard state 14 times at 50 ms intervals.
11. Nonempty, non-sentinel text is trimmed and inserted into SQLite.
12. Success emits `refresh` and `captured`. Failure generally logs to stderr.

Sources:

- [mac_tap.rs](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/src-tauri/src/mac_tap.rs)
- [capture.rs](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/src-tauri/src/capture.rs)
- [panel.rs](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/src-tauri/src/panel.rs)

Useful low-level work:

- `rdev` was removed from macOS dependencies, avoiding its off-main-thread keyboard-layout translation failure.
- Current listener reads modifier flags rather than decoding characters.
- Companion bits are masked because real macOS 12 tap events include both side-specific Shift, device-independent Shift, and non-coalesced bits.
- Tap liveness is checked with `CGEventTapIsEnabled`, not inferred from receiving a non-null tap object.
- Tap installation retries, and system-disabled taps are re-enabled.
- Window and enigo operations are scheduled on the main thread.
- Eleven state-machine unit tests use real flag words observed on macOS 12.

These patterns are valuable, but they do not make the full capture path reliable.

## 4. Confirmed capture and keystroke failure modes

### 4.1 In-flight captures are silently discarded

[`capture_selection`](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/src-tauri/src/capture.rs#L112) uses a process-wide `AtomicBool`:

- first attempt sets it true;
- every subsequent attempt returns immediately while true;
- no queue, local event, error, sound, toast, or diagnostic record is produced.

Normal worst-case busy interval is roughly 60 ms plus 14 × 50 ms, around 760 ms, before clipboard, IPC, DB, and scheduling overhead. A slow or blocked pasteboard call can extend this further. A second valid double-Shift inside that window is intentionally lost.

The guard is not RAII-backed. A panic before the final `store(false)` can leave capture permanently wedged until relaunch.

Bronze requirement: serialize with a real queue/actor. Every accepted or rejected trigger gets an attempt ID and visible state. Never silently drop.

### 4.2 Fallback chord fires before physical modifiers are released

[`register_fallback_shortcuts`](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/src-tauri/src/capture.rs#L90) handles `CmdOrCtrl+Shift+C` on `ShortcutState::Pressed`.

Capture sleeps a fixed 60 ms and then synthesizes Cmd+C. If the person still holds Shift, target application receives an effective Cmd+Shift+C rather than Cmd+C. Result may be another command, no clipboard change, or unrelated application behavior.

Bronze requirement: standard global shortcuts trigger on release. Before fallback injection, verify relevant physical modifiers are up. Add timeout and explicit error state.

### 4.3 Global fallback conflicts with Cooper's own command

Cooper registers Cmd+Shift+C globally for capture while [`DEFAULT_BINDS`](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/src/keybinds.ts#L14) assigns the same chord to in-panel “Copy as List.”

When panel has focus, native global capture and frontend list copy can race over the clipboard. The unmerged optimization PR changed fallback to Cmd+Alt+C for this reason.

Bronze requirement: one canonical shortcut registry spanning native and webview actions, with duplicate/conflict validation before saving.

### 4.4 Sentinel capture destroys rich clipboard content

Current code saves only `clip.get_text()`, then calls `set_text(SENTINEL)`.

Writing plaintext replaces the pasteboard's representations. On empty or failed capture:

- an image/file/multi-item clipboard has no saved text and is cleared;
- HTML/RTF/attributed text is restored as plaintext only;
- custom UTIs disappear;
- promised or lazy representations disappear.

This contradicts the README claim that previous clipboard content is “restored untouched”: [README capture mechanics](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/README.md#L149).

Bronze requirement: AX-first capture. Clipboard fallback records `NSPasteboard.changeCount` and never writes a sentinel. If no new ownership/change appears, leave pasteboard untouched. P0 never attempts automatic restoration; `changeCount` cannot provide atomic compare-and-swap, and copied data may already reach Clipboard History, Universal Clipboard, or third-party managers.

### 4.5 Synthetic Cmd can remain logically pressed

[`press_copy_chord`](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/src-tauri/src/capture.rs#L232) performs:

1. Cmd press;
2. Unicode `c` click with `?`;
3. Cmd release.

If step 2 fails, early return skips step 3. This can leave injected modifier state stuck and affect later input.

Bronze requirement: guaranteed key-up cleanup through scope guards or an API that posts explicit independent key-down/key-up events and always attempts cleanup after partial failure.

### 4.6 Fixed clipboard deadline loses slow copies

Clipboard polling lasts 700 ms after injection. Some Electron applications, remote sessions, overloaded processes, pasteboard data providers, and Universal Clipboard operations can exceed that interval.

Bronze requirement: observe generation change with configurable bounded deadline, use adaptive polling or notification where possible, and distinguish timeout from “empty selection.”

### 4.7 Timing model rejects reasonable taps

Constants are hard-coded:

- hold must be strictly less than 500 ms;
- second release must be strictly less than 400 ms after first release.

Pair timing is release-to-release. A fast second press followed by a slightly longer hold can fail despite obvious double-tap intent. No setting or first-tap cue exists.

Bronze requirement: configurable trigger side, hold threshold, pair threshold, and trigger mode. Store normalized values with safe ranges. Unit-test boundary values.

### 4.8 Capture errors are invisible

Listener registration conflicts, event-tap failures, enigo failures, clipboard failures, timeouts, and DB errors mostly reach stderr only. UI confirms success but does not describe failed stages.

Bronze requirement: local diagnostic ring buffer and structured failure events. Settings should show permissions, listener health, last trigger, last capture method, last duration, and failure reason without recording typed content.

### 4.9 Listener health is not end-to-end health

`tapRunning` proves only that `CGEventTapIsEnabled` returned true. It does not prove:

- selected text is available;
- AX messaging works;
- synthetic input works;
- clipboard can be read;
- target app responds to Cmd+C;
- DB write works;
- UI receives refresh.

Bronze requirement: separate health indicators for trigger monitor, Accessibility, Input Monitoring, fallback injection, persistence, and panel IPC.

### 4.10 Text fidelity changes silently

Captured text is passed through `trim()`. Intentional leading/trailing whitespace and edge indentation are lost.

Bronze requirement: preserve exact Unicode string content and whitespace returned by provider under documented line-ending policy. AX returns a string, not original source-encoding bytes, so do not claim byte-for-byte source fidelity. Use non-mutating emptiness detection.

### 4.11 Fixed and incomplete shortcut settings

[`ShortcutsEditor.tsx`](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/src/components/ShortcutsEditor.tsx) marks capture, show/hide, navigation, and Escape as fixed. It edits only in-app accelerators.

Additional defects:

- Cmd and Ctrl collapse to one `mod` token.
- duplicate bindings are allowed;
- browser/webview-reserved chords are not rejected;
- native registration failures are not surfaced;
- `KeyboardEvent.key` makes bindings layout-sensitive;
- no import/export or per-context scope.

Bronze requirement: typed shortcut schema with native and in-app actions, conflict detection, physical/logical key choice, context, reset, import/export, and accessible recorder.

## 5. macOS permission and TCC defects

### 5.1 Current one-permission model conflicts with Apple's supported model

Cooper's README and setup UI say one Accessibility checkbox covers both watching Shift and injecting Cmd+C.

Apple Developer Technical Support states that a listen-only Core Graphics event tap relies on **Input Monitoring**, while Accessibility is a different privilege: [Apple DTS thread](https://developer.apple.com/forums/thread/707680). Apple documents Input Monitoring as permission for applications to monitor keyboard, mouse, or trackpad while other apps are active: [Apple support guide](https://support.apple.com/guide/mac-help/control-access-to-input-monitoring-on-mac-mchl4cedafb6/mac).

Cooper:

- checks `AXIsProcessTrusted`;
- opens only the Accessibility pane;
- deliberately avoids `CGPreflightListenEventAccess` after a macOS 12 observation;
- does not expose Input Monitoring status or request flow.

That may work under specific legacy/unsandboxed combinations, but it is not a robust modern macOS contract.

Likely Bronze permissions:

- Input Monitoring for passive global event observation.
- Accessibility for AX selected-text queries and any synthetic event posting that requires it.

Each must have independent preflight, request, explanation, status, troubleshooting, and test cases.

### 5.2 Ad-hoc identity invalidates grants

Tauri config uses signing identity `-`, producing an ad-hoc signature. It has a code identity for a given build but not a stable trusted signing identity across rebuilds/releases. Cooper itself documents that updates can leave a visibly checked Accessibility row that no longer authorizes current binary.

Bronze requirement:

- stable persistent local signing certificate for personal development builds;
- Developer ID signing and notarization for distributed builds;
- bundle identifier, executable path, and signing identity kept stable;
- clean-VM permission tests and upgrade tests;
- setup distinguishes stale TCC record from missing grant.

### 5.3 Secure Input and session transitions

Secure Input can suppress global keyboard monitoring. Cooper has no detection or explanatory state. Sleep/wake, fast user switching, screen lock, keyboard attach, tap disable, and permission revoke can also disturb listener state.

Current tap-disable callback tries to re-enable the tap but does not reset `prev_flags`, `pressed`, `dirty`, or `last_tap`. If a press or release was missed during disablement, first later tap can be consumed while state resynchronizes.

Bronze requirement: reset complete state after every monitor lifecycle transition and show a privacy-safe “input monitoring temporarily unavailable” reason where detectable.

### 5.4 Privacy copy is inaccurate

Setup says Cooper “reads nothing else,” but tap subscribes to every `KeyDown` so it can mark Shift chords dirty. Callback does not inspect characters or keycodes for ordinary keys, which is materially better than logging keys, but it still receives key-down occurrence callbacks.

Bronze requirement: precise disclosure: monitor receives modifier transitions and key-down occurrence solely to reject chords; no characters, keycodes, or content are stored. Keep callback implementation aligned with that statement and test it.

## 6. AX-first selected-text capture reference

Closed, unmerged [PR #1](https://github.com/TouchMyBar/cooper/pull/1) contains useful design work:

- [`macos.rs`](https://github.com/TouchMyBar/cooper/blob/529cfba1518d80ab80d5a29f880fe6ecc00c0924/src-tauri/src/macos.rs) queries system-wide focused element, checks secure subrole, reads `AXSelectedText`, walks parents, and exposes pasteboard generation.
- [`capture.rs`](https://github.com/TouchMyBar/cooper/blob/529cfba1518d80ab80d5a29f880fe6ecc00c0924/src-tauri/src/capture.rs) uses AX first, falls back to copy only for unsupported apps, observes pasteboard generation without a sentinel, triggers fallback chords on release, and serializes requests through a bounded worker.

Do not cherry-pick wholesale:

- PR is closed and unmerged.
- Base predates later branches, images, drawing, Obsidian, and current macOS fixes.
- Its Shift listener tracks alternating booleans and can desynchronize when a flags event is missed.
- AX path returns `Empty` immediately when focused child reports empty selected text, instead of continuing to a parent that may expose actual selection.
- Bounded queue still silently ignores `Full` unless enhanced with user-visible busy/coalescing behavior.
- Permission sequence needs validation across supported macOS versions.

Recommended Bronze selection algorithm:

1. Record capture attempt and focused application metadata only if privacy setting permits.
2. Verify Accessibility.
3. Obtain system-wide focused element.
4. Walk bounded ancestor chain.
5. At each element, reject `AXSecureTextField` or equivalent protected roles.
6. Query selected text and selected range/parameterized string alternatives.
7. Treat empty child value as inconclusive until ancestor search completes.
8. Apply short AX messaging timeout per process/element.
9. On unsupported AX only, use pasteboard-generation copy fallback.
10. Preserve exact text; store capture method and duration, not source content in diagnostics.

## 7. Floating panel behavior

### 7.1 Confirmed implementation

- Main window starts `visible: true`: [Tauri config](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/src-tauri/tauri.conf.json).
- It is transparent, undecorated, always on top, skip-taskbar, resizable, and initially 380 × 600.
- macOS app uses accessory activation policy.
- Panel positioning uses the window's current monitor, falling back to primary monitor.
- Position uses total monitor dimensions with 16 px margin, not work area.
- It docks right and vertically centers.
- `visibleOnAllWorkspaces` is absent, although Tauri exposes it on macOS: [Tauri window API](https://docs.rs/tauri/latest/tauri/window/struct.WindowBuilder.html#method.visible_on_all_workspaces).
- Toggle hides only when panel is visible **and focused**. Visible but unfocused panel is focused rather than hidden.
- No persisted monitor, edge, size, opacity, or hide-on-blur behavior.
- No explicit native first-mouse acceptance configuration.

### 7.2 Resulting risks

- Hidden panel can remain associated with old monitor; summon appears away from current work.
- Full-screen/Space changes may reveal, switch to, or fail to reveal expected panel behavior.
- Right-side Dock or menu-bar geometry can overlap panel.
- Visible but unfocused panel behaves differently from user's “show/hide” mental model.
- Launch-at-login can expose panel despite summon-only expectation.
- First click on unfocused accessory window may focus rather than activate intended control.

These can be perceived as missed shortcuts even when listener fired.

### 7.3 Bronze requirements

- Start hidden except explicit first-run onboarding.
- Pick display under pointer or frontmost application, configurable.
- Use visible work area.
- Support left/right edge, offsets, width/height, per-display memory, and reset.
- Define current-Space/all-Spaces and full-screen behavior explicitly.
- Make show/hide semantics deterministic independent of focus.
- Persist and migrate window preferences.
- Restore prior application focus on hide.
- Test multiple monitors with different scale factors and Dock positions.
- Respect Reduce Transparency and Reduce Motion.

## 8. Persistence and data-model audit

### 8.1 Current schema

[`db.rs`](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/src-tauri/src/db.rs) defines:

- `sections`: integer ID, name, created timestamp, collapsed flag.
- `items`: integer ID, optional section, content, kind, done, timestamps, optional parent, branch style, collapsed flag.
- `attachments`: integer ID, item ID, absolute path, position, created timestamp.
- `settings`: string key/string value.

Items form a self-referential tree. UI/backend impose depth 10, but database does not enforce it. Image items store an absolute filesystem path in `content`; attachments also store absolute paths.

### 8.2 Confirmed defects

#### “Single SQLite file” is false

README says everything lives in one SQLite file, but image bytes live in a separate app-data `images` directory and SQLite stores paths. See [README local/private claim](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/README.md#L24) and [image feature description](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/README.md#L86).

Backup requires database **and** asset directory.

#### Absolute paths destroy portability

Moving app data, restoring onto another Mac, changing username, or importing into another environment invalidates stored paths.

#### Migrations ignore all ALTER errors

Initialization runs additive `ALTER TABLE` statements and discards every error. Duplicate-column errors are harmless; disk-full, permission, corruption, unsupported schema, and partial migration errors are not. App can continue with a partially upgraded schema.

#### No migration versioning

No `user_version`, migration table, ordered transaction, checksum, downgrade guard, backup, or integrity verification.

#### Filesystem and DB writes are not failure-safe

Image bytes are written before DB row insertion. DB failure leaves orphan file. Deletion removes DB rows first and ignores file deletion failure, also leaving orphans.

#### Clear-completed can delete unfinished descendants

[`clear_completed`](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/src-tauri/src/commands.rs#L118) checks only whether a done item has a directly undone child.

Example:

- parent: done;
- child: done;
- grandchild: undone.

Parent qualifies for deletion because direct child is done. Foreign-key cascade deletes child and undone grandchild.

#### State fetch and grouping scale poorly

- Every mutation reloads every section, item, and attachment.
- Frontend filters all items for every section.
- Search checks a top-level item plus immediate children, not complete deep branch.
- Images are cached forever as data URLs, with no eviction after deletion.
- Base64 IPC multiplies image memory and transfer cost.

#### Settings are untyped

Settings store arbitrary strings. Validation is inconsistent. No schema version, origin, default version, policy, or import/export.

### 8.3 Bronze persistence model

Recommended minimum:

- UUID primary keys.
- `sections`: name, normalized uniqueness rule, ordering key, timestamps.
- `items`: section, parent, type, exact content, status, ordering key, timestamps, capture method.
- `assets`: UUID, relative path, MIME, byte size, SHA-256, dimensions, timestamps.
- `item_assets`: item/asset relation and position.
- typed/versioned settings or validated config document.
- indexed `section_id`, `parent_id`, `updated_at`, ordering fields.
- migration table with version, checksum, timestamp; every migration transactional.
- startup integrity check plus recoverable backup.
- soft delete/undo before permanent purge.
- orphan asset reconciliation.
- generated Rust/TypeScript contracts.
- incremental list queries and event payloads.

## 9. Export audit

### 9.1 Standard Markdown export

[`export_markdown`](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/src-tauri/src/commands.rs#L378):

- always writes `cooper-export.md`;
- overwrites existing file;
- writes image references as `images/<basename>`;
- never creates/copies the referenced `images` directory;
- renders top-level item plus direct children only;
- omits grandchildren through depth ten;
- does not provide manifest/import metadata;
- does not write atomically.

Result: export can lose previous export, contain broken images, and omit valid nested data while claiming portability.

### 9.2 Obsidian export

Obsidian path is better:

- resolves configured/current vault;
- renders recursive task tree;
- copies image items and attachments;
- writes frontmatter;
- suffixes note filename collisions.

Remaining defects:

- existing attachment basename causes copy skip regardless of file content;
- partial failure can leave copied assets without note;
- no transaction or rollback across filesystem writes;
- source database and exported note can diverge without stable sync model;
- automatic reading of Obsidian config broadens “single local file” story.

### 9.3 Bronze export requirements

- Never overwrite without explicit selection.
- Atomic temp-write + fsync + rename where practical.
- Recursive, lossless hierarchy.
- Portable folder or ZIP: Markdown, assets, and versioned JSON manifest.
- Relative asset links.
- Content-hash collision handling.
- Export preview and validation.
- Round-trip import tests.
- Accessibility-friendly completion/error UI.

## 10. Security and privacy audit

### 10.1 CSP disabled

Tauri configuration explicitly sets `csp` to `null`: [tauri.conf.json](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/src-tauri/tauri.conf.json).

Current user text is rendered through React rather than raw HTML, reducing immediate injection risk. Defense in depth is still absent, and future rich content or navigation can turn XSS into native command access.

Bronze requirement: restrictive CSP. Prefer native-side update fetch so webview does not need broad `connect-src`.

### 10.2 Arbitrary file-read command

[`image_data(path)`](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/src-tauri/src/commands.rs#L288) accepts a frontend-provided path, reads it, base64-encodes bytes, and returns them.

A compromised allowed webview can read any file accessible to process, regardless of whether path belongs to an asset row.

Bronze requirement: frontend passes asset ID only. Rust resolves canonical path under owned asset root, verifies DB ownership, rejects traversal/symlinks outside root, validates size/MIME, and returns scoped data.

### 10.3 Arbitrary recursive quarantine removal

[`mac_remove_quarantine(path)`](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/src-tauri/src/commands.rs#L1117) accepts arbitrary frontend path and runs `xattr -dr com.apple.quarantine`.

This removes a macOS security attribute recursively from any accessible target. No canonical check restricts it to running app's actual bundle.

Bronze requirement: avoid this action where possible. If retained for self-built app, compute bundle path natively, require explicit confirmation, restrict exact canonical target, never accept frontend path, and explain security consequence.

### 10.4 Unbounded image input

Base64 image commands have no byte/dimension/decompression limit and infer type from extension. Risks: memory pressure, disk exhaustion, decompression bombs, misleading MIME, and UI freeze.

Bronze requirement: streaming or bounded binary transfer, magic-byte validation, decode limits, dimensions/pixel ceiling, file-size quotas, and safe thumbnail generation.

### 10.5 Release supply chain

[`build.yml`](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/.github/workflows/build.yml):

- grants `contents: write` at workflow level;
- uses mutable `actions/checkout@v4`;
- mutable `actions/setup-node@v4`;
- mutable `dtolnay/rust-toolchain@stable`;
- mutable `tauri-apps/tauri-action@v0`;
- uses `npm install`, not `npm ci`;
- performs no test/security gates before release.

No trusted macOS signing/notarization, checksums, SBOM, provenance, or reproducible-build statement exists.

Bronze requirement: immutable action SHAs, least-privilege job tokens, lockfile-exact installs, tests before package job, signed/notarized macOS artifacts, checksums, SBOM, provenance, and separate release approval.

### 10.6 “Local-only” nuance

Daily update check is enabled by default and fetches GitHub Releases: [update.ts](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/src/update.ts). This is not telemetry, but app is not strictly offline. Documentation should state network behavior and allow disable-before-first-request if strict privacy mode exists.

### 10.7 Dependency advisories

Locked Vite 5.4.21 falls inside 2026 development-server advisories:

- [GHSA-4w7w-66w2-5vf9](https://github.com/vitejs/vite/security/advisories/GHSA-4w7w-66w2-5vf9): optimized-dependency source-map path traversal; requires explicitly network-exposed dev server and predictable sensitive `.map` file.
- [GHSA-fx2h-pf6j-xcff](https://github.com/vitejs/vite/security/advisories/GHSA-fx2h-pf6j-xcff): `server.fs.deny` bypass through NTFS alternate/8.3 paths; Windows-specific and requires network-exposed dev server plus sensitive file in allowed directory.

Neither establishes macOS packaged-Tauri runtime exposure. Cooper does not explicitly set Vite `host`, so normal development default is loopback. Version still demonstrates missing dependency-audit discipline; advisory applicability must retain platform and deployment preconditions.

## 11. Accessibility and internationalization audit

Observed Cooper WebView UI fails multiple WCAG 2.2 A/AA fundamentals; no public conformance evidence was found. This is scoped source/UI evidence, not whole-native-app certification.

### 11.1 Card semantics

[`ItemCard.tsx`](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/src/components/ItemCard.tsx#L164):

- clickable card is a `<div>` without role, tabindex, accessible name, or selected state;
- completion button forces `tabIndex={-1}`;
- branch controls force `tabIndex={-1}`;
- visual selected/done/armed states lack corresponding semantics;
- clickable images are not keyboard-operable;
- image alt text is generic “captured” or “attachment.”

Global custom key handling does not substitute for focusable semantic controls or screen-reader state.

### 11.2 Dialogs and overlays

Settings, setup, and shortcut sheets use nested `<div>` elements:

- no `role="dialog"`;
- no `aria-modal="true"`;
- no labelled heading relation;
- no focus trap;
- no intentional initial focus;
- no focus restoration to opener;
- click-outside dismissal can be accidental.

See [SettingsSheet.tsx](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/src/components/SettingsSheet.tsx), [MacSetupSheet.tsx](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/src/components/MacSetupSheet.tsx), and [ShortcutsEditor.tsx](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/src/components/ShortcutsEditor.tsx).

### 11.3 Menus

[`ContextMenu.tsx`](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/src/components/ContextMenu.tsx):

- no menu/menuitem roles;
- no initial focus;
- no roving tabindex;
- no arrow/Home/End navigation;
- no typeahead;
- no keyboard context-menu invocation;
- actions such as rename/retarget/export are primarily right-click discoverable.

### 11.4 Status and errors

- Toast has no `aria-live` or status role.
- Drop overlay has no announcement.
- Capture errors are not exposed in UI.
- Update banner does not manage announcement/focus.
- Busy and permission transitions are not announced.

### 11.5 Focus, motion, contrast, display preferences

[`styles.css`](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/src/styles.css#L111) removes `outline` from all inputs, textareas, and buttons. Only selected components receive custom focus styling.

Missing:

- general `:focus-visible` treatment;
- `prefers-reduced-motion` handling despite animations/transitions;
- forced-colors support;
- reduced-transparency handling;
- high-contrast theme;
- 200%/400% zoom and reflow verification;
- automated contrast verification.

Light-theme muted `#86868b` on white/light surfaces is likely below 4.5:1 for 13 px normal text and must be measured.

### 11.6 Internationalization

All strings are hard-coded English. Missing:

- message catalogs;
- plural rules;
- locale-aware dates/times;
- locale-aware case/search;
- RTL layout;
- translated permission/setup copy;
- localized key names;
- layout-independent shortcuts;
- pseudolocalization and expansion testing.

Bronze requirement: use owned shadcn React Aria primitives only as a starting point; verify actual semantics, keyboard behavior, VoiceOver, contrast, reduced motion/transparency, zoom, and localization. “Uses accessible library” is not conformance proof.

## 12. Testing and documentation health

### 12.1 Existing tests

There are 11 Rust unit tests, all in [`mac_tap.rs`](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/src-tauri/src/mac_tap.rs#L254). They cover:

- left and right pairs;
- single tap;
- timing-window rejection;
- hold rejection;
- chord rejection;
- captured live flag words;
- side mismatch;
- both-Shifts case;
- third-tap behavior.

No tests cover:

- AX selected text;
- permissions/TCC;
- actual event tap installation;
- enigo input;
- clipboard formats or timeouts;
- capture queue/busy state;
- SQLite commands or migrations;
- recursive deletion;
- Markdown/Obsidian export;
- frontend behavior;
- Tauri IPC permissions;
- multi-monitor/Spaces/focus;
- accessibility;
- localization;
- performance.

CI packages application but does not run these 11 tests.

### 12.2 Stale documentation

[`CLAUDE.md`](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/CLAUDE.md) says repository has no `#[cfg(test)]`; current tree does. `Panel.tsx` comment says rendering is capped at one child level while implementation now supports ten. Lockfile version is stale. README's single-file and clipboard-preservation claims contradict code.

Lesson: do not treat repository-authored agent guidance as verified truth. Bronze agentic loop must regenerate architecture references from code or enforce documentation checks in CI.

### 12.3 Required Bronze capture test matrix

Unit/property tests:

- arbitrary press/release/modifier sequences;
- strict boundary values;
- event loss and reordering;
- tap disable/re-enable reset;
- triple/quad taps;
- keys held before Shift;
- simultaneous modifiers;
- auto-repeat;
- configurable thresholds;
- queue saturation and cancellation.

macOS integration matrix:

- every macOS major/patch selected by ADR-002, including current release; unsupported older versions may be negative evidence but are not release claims;
- Apple silicon, plus Intel/universal2 only if ADR-002 selects Intel support;
- clean VM snapshots for every TCC flow;
- stable Developer ID identity; ad-hoc builds only as negative evidence for permission-identity drift, never supported release artifacts;
- built-in, external USB, Bluetooth, compact, and remapped keyboards;
- QWERTY, AZERTY, QWERTZ, Dvorak, Colemak, IME;
- sleep/wake, lock/unlock, user switch, keyboard reconnect;
- permission grant/revoke/stale record;
- Secure Input active/stuck;
- multiple displays, mixed scaling, all Dock positions, Spaces, full-screen apps.

Target applications:

- TextEdit;
- Safari;
- Chrome/Chromium;
- Firefox;
- VS Code and other Electron apps;
- Terminal and iTerm;
- Xcode;
- Finder filenames;
- Preview/PDF viewers;
- native tables/lists;
- secure/password fields;
- remote desktop/VM clients.

Clipboard corpus:

- plaintext;
- same plaintext already present;
- empty selection;
- whitespace-only and indentation-sensitive text;
- RTF;
- HTML;
- attributed string;
- image;
- file URLs;
- multiple items;
- custom UTIs;
- lazy/promised data;
- Universal Clipboard;
- simulated copy delays from 0 to 2 seconds.

UI/accessibility:

- keyboard-only operation;
- VoiceOver navigation and announcements;
- Full Keyboard Access;
- Switch Control where practical;
- 200% and 400% zoom;
- reduced motion/transparency;
- forced/high contrast;
- RTL and pseudolocale;
- axe checks plus manual WCAG audit.

## 13. Reddit post assessment

Source: [“I recreated shadcn's Copper as a free, open-source app…”](https://www.reddit.com/r/ClaudeAI/comments/1ve1dgb/i_recreated_shadcns_copper_as_a_free_opensource/)

Facts:

- First-person promotional post tagged “Built with Claude.”
- Repeats repository pitch: double left Shift capture, double right Shift panel, sections, inline Markdown, local SQLite, Tauri, unsigned binaries, Accessibility.
- Core workflow block appears twice.
- Only visible substantive comment asks whether app is essentially fast copy/paste history plus scratchpad.
- Post presents no independent macOS reliability evidence, benchmarks, security review, accessibility evidence, or test report.
- Repeats inaccurate “one SQLite file” claim after image storage moved outside DB.
- Says Copper costs $50; referenced product page/user reports another price. Price must be treated as time-, region-, and promotion-sensitive.

Assessment: marketing source, not validation. It confirms intended workflow and positioning but cannot support reliability, privacy, accessibility, or architecture claims.

## 14. License and implementation-provenance risk

### 14.1 Cooper code license

Repository is [Apache-2.0](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/LICENSE).

Apache-2.0 would permit Bronze to copy Cooper source if it:

- preserve required license/copyright notices;
- identify modifications where required;
- include Apache license with distributed work;
- audit third-party dependency licenses separately;
- understand Apache grants no trademark rights.

The Reddit phrase “fork it… no strings” is informal. Apache has conditions. Bronze v1 policy is stricter: copy no Cooper source code. Source inspection informs failure requirements and architecture provenance only.

### 14.2 Copper product material

Latest Cooper commit explicitly removed five accidentally committed Copper screenshots because they belonged to someone else's product UI: [commit 14ff307](https://github.com/TouchMyBar/cooper/commit/14ff307023b0e94d48a209db578a3e411bee7d10).

Bronze should avoid:

- Copper/Cooper name or confusingly similar branding;
- logos and icons;
- copied screenshots or video frames;
- copied product text;
- proprietary assets;
- pixel-identical trade dress;
- claims of affiliation or endorsement.

Functional workflow and failure analysis can inform an independent implementation. Use own brand, visual system, copy, icons, data model, and code. Because Cooper source was inspected, do not claim a formal clean-room process; record it as an analyzed reference. If future scope proposes source reuse, require a new ADR, exact file-level provenance, Apache notices/modification records, and dependency-license review.

Legal-risk inference, not legal advice: “Cooper” differs by one letter from “Copper,” and repository markets itself as a faithful recreation. Bronze branding materially reduces confusion/passing-off risk.

## 15. Reuse matrix

| Area | Reuse as concept | Copy code directly | Bronze decision |
| --- | --- | --- | --- |
| Tauri Rust/React split | Yes | No in v1 | Keep architecture concept, redesign contracts |
| SQLite local storage | Yes | No need | New versioned schema |
| macOS modifier-only event tap | Yes | No in v1 | New actor/state machine and permission flow |
| Device-dependent left/right bits | Yes | No in v1 | Independently validate keycodes/flags across hardware |
| Tap retry/health | Yes | No in v1 | Add state reset and diagnostics |
| Main-thread macOS operations | Yes | Pattern only | Enforce through native abstraction |
| App translocation diagnosis | Yes | No in v1 | Independently implement relocation-first onboarding |
| Clipboard sentinel | No | Never | AX first + generation-based fallback |
| Atomic busy guard | No | Never | Explicit queue with attempt IDs |
| Global shortcut scheme | No | Never | Unified configurable registry |
| Full-state refresh | MVP only | No | Granular typed events/queries |
| DB migrations | No | Never | Transactional versioned migrations |
| Absolute asset paths | No | Never | Relative asset IDs and manifest |
| Markdown export | No | Never | Lossless portable exporter/importer |
| Panel positioning | Partial | No | Work-area/current-display/Spaces model |
| Plain custom UI semantics | No | Never | Accessible component system and audit |
| CSP/IPC scope | No | Never | Default-deny capabilities and path validation |
| CI/release | No | Never | Pinned, gated, signed, notarized pipeline |
| Cooper/Copper branding/assets | No | Never | Independent Bronze identity |

## 16. Implementation lessons for Bronze

1. Treat global capture as a stateful native subsystem, not a shortcut callback.
2. Split permission, listener, selection, fallback, persistence, and UI health.
3. Use AX selected text first; clipboard simulation is compatibility fallback.
4. Never mutate clipboard to detect whether copy happened.
5. Never silently drop a trigger. Queue, coalesce with notice, or reject visibly.
6. Guarantee synthetic key cleanup after partial errors.
7. Fire standard fallback actions after physical shortcut release.
8. Make trigger thresholds and all shortcuts configurable and conflict-checked.
9. Sign with stable identity before judging TCC reliability.
10. Test TCC only on clean VM snapshots; cached developer-machine state misleads.
11. Start floating panel hidden and place it using current work context/work area.
12. Store assets by relative ID, not absolute path.
13. Make export a tested, lossless, round-trippable product feature.
14. Keep Tauri commands narrow: IDs in, validated owned paths internally.
15. Use CSP and capability scoping as if webview content will eventually be compromised.
16. Generate Rust/TypeScript contracts.
17. Avoid whole-state refetch for every mutation.
18. Design WCAG semantics before styling; automated checks supplement manual testing.
19. Internationalize from first user-facing string.
20. Agentic development requires executable acceptance criteria, adversarial tests, and documentation drift checks.

Final verdict: study Cooper as a behavioral reference and failure corpus, but copy no Cooper source in Bronze v1. Build an independently implemented macOS-first product with documented provenance rather than a fork-based cleanup.
