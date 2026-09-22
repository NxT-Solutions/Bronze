# System architecture

Version: 1.0 planning baseline  
Status: implementation contract  
Platform: macOS v1  
Related: [PRD](03-prd.md), [functional specification](04-functional-spec.md), [macOS capture reliability](07-macos-capture-reliability.md), [architecture decisions](18-adrs.md)

## 1. Purpose and authority

This document defines Bronze runtime boundaries, package ownership, privileged interfaces, persistence model, performance gates, and release architecture. Agents may refine internal implementation, but must not cross a boundary or change an accepted decision without updating [ADRs](18-adrs.md).

PRD traceability:

| Area | Requirements |
| --- | --- |
| Capture | CAP-001 through CAP-010 |
| Queue/domain | QUE-001 through QUE-008 |
| Window shell | WIN-001 through WIN-005 |
| Data | DAT-001 through DAT-004 |
| Security/privacy | SEC-001 through SEC-006 |
| Accessibility | A11Y-001 through A11Y-006 |
| Internationalization | I18N-001 through I18N-004 |
| Settings/support | SET-001 through SET-004, SUP-001, SUP-002 |

## 2. Scope boundary

Bronze is local selection-to-action queue:

    selected content
      → captured item
      → optional prompt or note
      → ordered section
      → copy profile
      → explicit lifecycle transition

P0 includes selected-text capture, manual composition, sections, ordered items, copy profiles, completion/undo/trash, local search, settings, permission diagnostics, backup, import/export, accessibility, localization, and signed distribution.

P0 does not include screen recording, screenshots, microphone, camera, OCR, passive clipboard history, account, sync, hosted AI, telemetry, collaboration, Windows, or Linux. No ScreenCaptureKit, AVFoundation, VideoToolbox, or Screen Recording permission belongs in v1. Future file/image attachments require separate threat model and ADR.

## 3. Architecture decisions summary

| Concern | Decision |
| --- | --- |
| Desktop shell | Tauri 2 using system WKWebView |
| UI | React, TypeScript, shadcn React Aria base, Tailwind design tokens |
| Native macOS | In-process Swift static library behind narrow Rust façade |
| Core | Rust domain, capture coordinator, settings, storage, validation |
| Storage | SQLite WAL, FTS5, versioned transactional migrations, backups |
| JS workspace | pnpm and Turborepo |
| Quality tools | Biome for JS/TS; rustfmt/Clippy; swift-format |
| Trigger | Registered standard chord plus optional passive CGEventTap modifier gesture |
| Selection | Accessibility API first; bounded explicit clipboard fallback |
| Distribution | Direct Developer ID, Hardened Runtime, notarized; no App Sandbox v1 |
| Runtime network | None in production-default build |
| Conformance target | WCAG 2.2 AA WebView plus relevant EN 301 549 clauses 5, 11, and 12 whole-app evidence |
| Locale model | BCP 47, ICU messages, Intl formatting, full RTL |

macOS minimum remains Spike 0 decision. Planning hypothesis is macOS 15 or later; architecture must not silently encode this until ADR-002 becomes Accepted.

## 4. System context

~~~mermaid
flowchart LR
  U[User]
  S[Source application]
  B[Bronze app]
  T[Destination application]
  P[macOS TCC and platform APIs]
  D[(Local Bronze data)]

  U -->|selects text and invokes trigger| S
  S -->|AX selection or explicit copy| B
  U -->|edits, orders, copies, completes| B
  B -->|plain text or selected output profile| T
  B <-->|Input Monitoring, Accessibility, AppKit, pasteboard| P
  B <-->|transactions, search, backup, export| D
~~~

Text equivalent: Bronze receives deliberate user triggers, reads source selection through authorized macOS APIs, stores local queue state, and writes user-requested output to system pasteboard. No service or remote host participates.

## 5. Container and thread model

~~~mermaid
flowchart TB
  subgraph WebView["Tauri WebView: untrusted presentation"]
    QW[Quick panel]
    LW[Library]
    SW[Settings]
    UI[React UI packages]
  end

  subgraph Rust["Rust: privileged application core"]
    IPC[Typed command boundary]
    CC[Serial capture coordinator]
    DM[Domain model and command handlers]
    ST[SQLite repository]
    DG[Redacted diagnostics]
    PM[Permission and shortcut services]
  end

  subgraph Swift["Swift static library: platform adapter"]
    ET[CGEventTap run-loop thread]
    FSM[Double-tap FSM]
    AX[AX serial queue]
    AK[AppKit main-thread adapter]
    PB[NSPasteboard adapter]
  end

  DB[(SQLite, WAL, backups)]

  QW --> IPC
  LW --> IPC
  SW --> IPC
  IPC --> DM
  IPC --> CC
  IPC --> PM
  CC --> AX
  ET --> FSM
  FSM --> CC
  PM --> AK
  DM --> ST
  ST --> DB
  DM --> DG
  CC --> DG
  PB --> CC
~~~

Thread rules:

1. CGEventTap owns dedicated CFRunLoop thread. Callback performs no AX, UI, IPC, database, allocation-heavy, or logging work.
2. Modifier FSM consumes compact native events on same thread or bounded native queue.
3. One serial capture coordinator owns CAP-004 request ordering and terminal-state transition.
4. AX queries run on dedicated serial queue with deadlines. Late responses are discarded by request generation.
5. AppKit, NSStatusItem, activation policy, and NSWindow mutation run on main thread.
6. SQLite has one serialized writer. Read pool may serve bounded queries under WAL.
7. React never calls Swift. React invokes typed Rust commands; Rust alone crosses native ABI.
8. No callback holds UI, store, or domain locks.

## 6. Repository and package layout

Target tree:

~~~text
apps/
  desktop/
    src/
      app/
      features/
        capture/
        composer/
        queue/
        search/
        settings/
        support/
      routes/
      main.tsx
    src-tauri/
      capabilities/
        quick.json
        library.json
        settings.json
        onboarding.json
      src/
        commands/
        events/
        setup/
        main.rs
      tauri.conf.json
packages/
  ui/
    src/components/
    src/tokens/
  contracts/
    src/generated/
    src/index.ts
  i18n/
    src/
    locales/{en,nl,fr,de,es,it,en-XA,ar-XB}/app.json
    scripts/
  test-support/
bronze-domain/
bronze-capture/
bronze-storage/
bronze-settings/
bronze-diagnostics/
bronze-platform/
bronze-platform-macos/
native/
  macos/
    BronzeNative/
      Package.swift
      Sources/
      Tests/
~~~

Ownership:

| Package | Owns | Must not own |
| --- | --- | --- |
| apps/desktop/src | Rendering, focus, local draft, accessible interaction | SQL, filesystem paths, AX, pasteboard mutation, permission decisions |
| packages/ui | Owned shadcn components and tokens | Product state, IPC calls |
| packages/contracts | Generated DTOs and error codes | Domain behavior |
| packages/i18n | Message catalogs, locale resolution, formatters | User content transformation |
| bronze-domain | Entities, lifecycle, order, undo invariants | Tauri, AppKit, SQL |
| bronze-capture | Request state machine and provider policy | Direct WebView access |
| bronze-storage | Schema, repositories, migration, backup/import/export | UI messages |
| bronze-settings | Typed settings schema, shortcut registry and policy (SET-*) | Rendering, Tauri/AppKit, capture decisions |
| bronze-diagnostics | Redacted event schema and support bundle | Captured or clipboard payload |
| bronze-platform | Platform-neutral traits | macOS framework imports |
| bronze-platform-macos | Safe Rust façade and ABI ownership | Business rules |
| BronzeNative | CGEventTap, AX, AppKit, NSPasteboard | Queue state, SQL, localization policy |

Turborepo tasks:

| Task | Cache |
| --- | --- |
| format, lint, typecheck, unit test, frontend build | enabled with declared inputs |
| Rust compile/test | enabled only with toolchain and Cargo inputs included |
| native Swift test | enabled only with Xcode/SDK identity included |
| accessibility E2E, TCC tests, signing, notarization, release packaging | disabled |

Pin Node/package manager, Rust, Xcode minimum, Swift tools version, Tauri, and dependency lockfiles. Generated contracts must be reproducible and CI must fail on uncommitted generation drift.

## 7. Native bridge

### 7.1 Decision

Use in-process Swift static library, not helper/sidecar. One executable retains one lifecycle, code-signing identity, and TCC subject. Rust owns public Tauri surface. Swift is implementation detail.

### 7.2 ABI contract

Bridge must expose:

- ABI version and feature bitset.
- Explicit initialization/shutdown.
- Permission snapshot and request operations.
- Start/stop/reconfigure trigger monitor.
- Capture-selection request with request ID, target PID, options, and completion callback.
- Pasteboard read/write/fallback transaction.
- Frontmost-app snapshot and activation.
- Window/status-item platform tweaks.

Rules:

- C-compatible tagged enums and fixed-width integers only.
- UTF-8 buffers always carry pointer plus length; never rely on NUL termination.
- Allocation owner provides matching free function.
- Rust panic and Swift exception/error never cross ABI.
- Completion occurs exactly once or is cancelled during shutdown.
- Callbacks document queue. Rust immediately copies borrowed buffers.
- Every async request carries opaque ID and generation.
- Sensitive buffer lifetime ends after handoff; no debug description may include content.
- ABI conformance tests cover empty, embedded NUL, invalid UTF-8 rejection, large payload, cancellation, shutdown race, and double completion.

### 7.3 Why not alternatives

| Alternative | Reason rejected for v1 |
| --- | --- |
| Swift helper sidecar | Extra IPC, process lifecycle, nested signing, permission identity ambiguity, larger attack surface |
| Pure Rust/macOS FFI | Viable, but larger unsafe/CoreFoundation/AppKit ownership surface for first macOS reliability target |
| JavaScript native plugin | Privileged capture exposed too close to WebView trust boundary |
| NSEvent global monitor alone | Cannot implement complete own-app/global modifier semantics as precisely; key monitoring permission behavior differs |
| IOHID interception | More invasive than required; wrong privacy and portability trade-off |

## 8. Runtime flows

### 8.1 Startup

1. Enforce single instance.
2. Resolve local container and open database read-only enough to inspect version.
3. Verify integrity; create backup before pending migration.
4. Apply transactional migrations.
5. Load settings and locale (`SettingsV1.general.locale`; `system` and unknown map to **en**; native menus use that catalog at launch).
6. Create accessible status item and warm quick panel (debug `tauri dev` shows it on launch; capture-only still does not reveal it).
7. Register last-known-good standard chord.
8. Request Accessibility and Input Monitoring when not already granted; never Screen Recording; never loop-prompt.
9. Start passive event tap only when configured and authorized.
10. Publish local health snapshot to settings/support UI.

If storage cannot enter writable healthy state, app enters read-only recovery. It must not accept captures that appear saved.

### 8.2 Capture

~~~mermaid
sequenceDiagram
  participant M as Trigger monitor
  participant C as Capture coordinator
  participant N as Native AX adapter
  participant R as Repository
  participant W as Quick panel

  M->>C: trigger(id, time, target/destination/policy generation snapshot)
  C->>C: queue immutable ingress snapshot
  C->>N: revalidate target; acquire focus identity; read selection
  N-->>C: captured or typed failure
  C->>R: transaction: item/source/undo
  R-->>C: committed revision
  C->>C: publish terminal result; append diagnostic independently/best effort
  C-->>W: scoped item-created event
  W->>W: visible and announced feedback
~~~

Menu-bar Capture, Bronze-menu Capture, and the saved `capture.selection` Shift N-tap snapshot `NSWorkspace` frontmost PID before any first-capture permission prompt. When Bronze is frontmost, persist first reads the current highlight from the Quick, Settings, Library, and Help WebViews with a fixed Rust JavaScript snippet (`getSelection` and non-password field selection; `LIVE_AX_MAX_BYTES`). WKWebView typically does not expose `AXSelectedText`. If that read is empty, persist uses last-external PID (`AXSelectedText`, or `AXSelectedTextRange` plus `AXAttributedStringForRange` / `AXStringForRange`; focused element, then that app’s windows and a bounded child search). Allowed text roles include `AXWebArea`, `AXBrowser`, and `AXDocument`. Neutral containers are queried for selected text when the focused Electron/Chromium control exposes it there. Last-external memory still skips Bronze. System-wide focused AX is used when no last-external PID exists and may read own PID. Event-tap callbacks do not evaluate JavaScript or touch windows. Persist writes `compact_title` before emit. Title refine, if scheduled, runs after persist on a dedicated Rust thread and never in the event-tap. Every persist outcome emits `capture-result` and delivers a Notification Center banner (`UNUserNotificationCenter` in a `.app`, or the signed `BronzeNotice.app` helper in unbundled `tauri dev`, plus `announcementRequested`, catalog title/body only); Saved also emits `queue-changed`. Every `ShortcutActionId` ships an enabled default (`docs/12` §4). Event-tap live fire is `capture.selection` Shift N-tap from the shortcuts table (`tapCount` 2 through 8; absent means 2); non-Shift remaps disable that engine; other globals persist without an OS grab; app-local chords persist for the UI resolver. ADR-018 stays Proposed. The inbox renders catalog `capture.source` plus a list-time 16px official app icon (`sourceAppIcon` data PNG from NSWorkspace / the installed `.app`) when the name is present, including Bronze. `#capture-status` stays a visually hidden live region for `capture.announce.*`; `#chrome-notice` is the viewport-fixed in-app line. Composer rows stay unlabeled. URL provenance is off. File and image attachments stay out of P0 until a separate threat model and ADR (ADR-001).

Capture-specific algorithms live in [macOS capture reliability](07-macos-capture-reliability.md).

### 8.3 Queue mutation

All mutations use command objects and optimistic revision:

    command ID + actor window + entity ID + expected revision + validated payload

Repository transaction writes entity change, undo record, and content-free command receipt/entity references. Domain event emits only after commit. Duplicate command ID inside retry window returns reconstructed prior result. Timestamped command ID older than retry window returns `idempotency_expired` and is never executed as new. Stale revision returns typed conflict; UI retains draft and offers reload/reapply.

### 8.4 Copy

1. UI sends ordered item IDs plus output-profile ID.
2. Rust reloads items, verifies section/order/state, and builds immutable output.
3. Rust calls native pasteboard write with plain text plus sanitized constrained-markdown HTML (`ul`/`ol`/`li`/`strong`/`em`/`br` only). Plain succeeds even when the HTML type cannot be written.
4. Only successful write applies profile lifecycle: unchanged, copied, active, or done.
5. Failure leaves lifecycle unchanged.
6. UI announces localized confirmation once.

Pasteboard and SQLite cannot form one atomic transaction. Required order is deterministic output → successful pasteboard write → DB lifecycle/receipt transaction. Crash in gap may leave output on clipboard while lifecycle remains unchanged; retrying same command rewrites identical output then commits receipt/state. Never mark lifecycle before pasteboard success.

WebView must preserve normal Cmd-C inside editable controls or non-collapsed text selection. Whole-item copy applies only to item focus with no active text selection. Visible Copy action always exists.

## 9. Domain and persistence

### 9.1 Aggregate model

- Workspace: one local v1 root.
- Section: title, state, color token, stable order, timestamps.
- Item: identity, section, content, optional title, canonical BCP 47/`und` content language, lifecycle, stable order, source, revision, timestamps.
- Source: optional provenance according to policy; captured/authored text always lives in `items.body`; optional `items.title` is a derived heading, not a second body.
- CaptureRequest: transient request plus redacted terminal diagnostic.
- OutputProfile: deterministic formatter, post-copy lifecycle action, and focus-advance policy; sole post-copy authority.
- Shortcut: semantic action plus trigger representation.
- UndoEntry: command and reversible inverse.
- Settings: versioned typed values.

Item lifecycle is exactly:

    queued | copied | active | done | skipped | trashed

No UI-only state may silently change lifecycle.

### 9.2 Logical schema

~~~text
workspaces(id, name, created_at_ms, updated_at_ms)
sections(id, workspace_id, title, state, color_token, rank, revision,
         created_at_ms, updated_at_ms, deleted_at_ms)
items(id, section_id, kind, body, title, content_language, status, rank, source_id, revision,
      created_at_ms, updated_at_ms, completed_at_ms, deleted_at_ms)
sources(id, bundle_id, app_name, safe_title, url, captured_at_ms, policy_version)
item_revisions(item_id, revision, body, content_language, status, changed_at_ms, change_kind)
output_profiles(id, builtin_key, name, format, format_options_json, source_policy,
                post_copy_action, advance_policy, revision)
shortcuts(action, trigger_json, enabled, revision, updated_at_ms)
settings(key, schema_version, json_value, updated_at_ms)
undo_log(id, command_type, inverse_json, created_at_ms, expires_at_ms)
command_receipts(id, command_type, result_code, created_at_ms, expires_at_ms)
command_receipt_entities(command_id, ordinal, entity_id, result_revision)
diagnostic_events(id, occurred_at_ms, request_id, stage, result_code,
                  duration_ms, trigger_kind, provider_kind, permission_state,
                  source_bundle_id, app_schema_version, queue_depth,
                  overflow_count, tap_health, store_result_code, build_id)
schema_migrations(version, name, checksum, applied_at_ms)
items_fts(body, section_title, safe_provenance)
~~~

Content fields never enter capture diagnostics. FTS is local but still sensitive persisted data; backup/export/privacy documentation must state it exists.

### 9.3 Storage rules

- SQLite WAL, foreign keys, busy timeout, integrity check.
- One writer; bounded read pool.
- Every migration versioned, checksummed, idempotence-tested, transactional where SQLite permits.
- Backup before migration and before restore.
- No silent destructive reset.
- Atomic export/import staging.
- Directory mode 0700 and files 0600 where platform permits.
- Database and WAL never stored in iCloud/Dropbox by default.
- FileVault may protect disk at rest; Bronze must not claim application-level encryption.
- SSD/WAL behavior prevents guaranteed forensic secure deletion. DAT-004 promises logical scoped deletion and configured trash recovery, not impossible physical erasure.

Export manifest is deterministic and locale-neutral. It includes format/version, UTC creation time, checksums, sections, items, safe settings subset, provenance choices, and reserved assets directory. Import validates schema, size, checksums, UTF-8, paths, IDs, enums, timestamps, duplication policy, and transaction outcome before swap.

## 10. Tauri IPC and capability model

WKWebView is untrusted boundary. Tauri security model explicitly separates privileged Rust core from WebView. See [Tauri security](https://v2.tauri.app/security/), [permissions](https://v2.tauri.app/security/permissions/), and [runtime authority](https://v2.tauri.app/security/runtime-authority/).

Window capabilities:

| Window | Allowed capability |
| --- | --- |
| quick | read/search active section, create/edit/reorder/transition active items, copy, capture status, hide |
| library | paginated queue/search, section/item lifecycle, reorder, import/export, backup/restore request |
| settings | typed settings, output-profile management, shortcut registration/test, permission preflight/request, diagnostics preview; no raw global-shortcut plugin capability |
| onboarding | manual note, typed permission preflight/request, external TextEdit self-test, locale/help; no library, import/export, raw shortcut plugin, or diagnostics authority |

Forbidden everywhere unless new ADR:

- Arbitrary filesystem/path API.
- Shell/process execution.
- Raw SQL.
- Raw AX/event-tap/pasteboard API.
- HTTP, WebSocket, upload, remote asset loading.
- Unrestricted opener.
- Remote-origin IPC.

Command surface should resemble:

~~~text
capture_request(trigger_context) -> CaptureReceipt
capture_cancel(request_id) -> CommandResult
item_create(input, command_id) -> Item
item_update(id, expected_revision, patch, command_id) -> Item
item_move(ids, destination, rank, command_id) -> MoveResult
item_transition(ids, expected_revisions, transition, command_id) -> TransitionResult
queue_query(cursor, filter, limit) -> Page
copy_items(ids, output_profile_id, command_id) -> CopyResult
output_profile_list() -> OutputProfile[]
output_profile_upsert(input, expected_revision, command_id) -> OutputProfile
output_profile_delete(id, replacement_default_id, expected_revision, command_id) -> DeleteResult
settings_get(group) -> SettingsGroup
settings_update(patch, expected_revision) -> SettingsGroup
shortcut_test(candidate) -> ShortcutTest
permission_snapshot() -> PermissionSnapshot
permission_request(kind) -> PermissionRequestResult
export_begin(options) -> ExportResult
import_preview() -> ImportPreview
import_commit(preview_token, strategy) -> ImportResult
backup_create() -> BackupResult
restore_preview() -> RestorePreview
restore_commit(preview_token) -> RestoreResult
diagnostics_preview() -> RedactedSupportBundle
~~~

Rust validates window label, capability, payload, enum, length, ID, revision, and state transition. Renderer-provided paths are never trusted. File dialog and subsequent operation remain native/Rust-owned.

Events are scoped and content-minimal:

- capture-stage-changed: request ID, stage, safe status.
- item-created/updated: ID and revision; window refetches authorized entity.
- permission-changed: capability and status.
- settings-changed: group and revision.

Captured content must not be broadcast globally. CSP permits self-hosted build assets and required Tauri IPC only; no inline evaluation, frames, objects, or remote origins. Release disables DevTools and navigation away from packaged origin. See [Tauri CSP](https://v2.tauri.app/security/csp/) and [capabilities by window](https://v2.tauri.app/learn/security/capabilities-for-windows-and-platforms/).

Rust owns Tauri global-shortcut plugin registration/listening. No WebView receives raw register, unregister, or listen permission; settings invokes a typed Bronze command that validates action, conflict, scope, and rollback.

## 11. Window and menu architecture

- One accessible status item and native menu.
- One warm hidden quick panel.
- One resizable P0 library window for section lifecycle, archive/trash, search, import/export, backup, and recovery.
- One resizable settings/support window or route with settings-only capability.
- Quick panel is activating/key-capable normal window, not nonactivating panel.
- App defaults to accessory activation policy; Dock icon setting may change policy.
- Position from visible work area and selected pointer/frontmost-app display.
- Persist explicit physical left/right/top edge and width, not RTL-relative edge or unvalidated absolute frame.
- Clamp after display removal, resolution/scale change, Dock/menu bar/notch change.
- Full-screen auxiliary/all-Spaces behavior stays opt-in until matrix-tested.
- Escape/hide restores prior focus when safe; modal/dialog focus restores invoking control.
- Status item/menu carries localized accessible name and ordinary menu keyboard semantics.

Apple references: [frontmost application](https://developer.apple.com/documentation/appkit/nsworkspace/frontmostapplication), [NSStatusItem](https://developer.apple.com/documentation/appkit/nsstatusitem), [activation policy](https://developer.apple.com/documentation/appkit/nsapplication/activationpolicy-swift.enum).

## 12. Accessibility architecture

Conformance language:

- WebView target: WCAG 2.2 AA.
- Whole-app target: EN 301 549 V3.2.1 software requirements, especially clauses 5, 11, and 12.
- Platform evidence: Accessibility Inspector plus VoiceOver, Full Keyboard Access, Voice Control, Switch Control.
- “WCAG proof” or “fully accessible” cannot ship without scoped evidence and independent audit.

UI base is shadcn React Aria. React Aria became first-class shadcn base in July 2026; initialize with aria base. Generated components remain owned source and require audit. [shadcn React Aria](https://ui.shadcn.com/docs/changelog/2026-07-react-aria)

Implementation rules:

- Semantic HTML before ARIA.
- Avoid contenteditable and virtualized core list in P0.
- No timing gesture is sole route: chord, status menu, manual composer, buttons.
- Visible focus and focus restoration.
- Modal behavior follows WAI-ARIA dialog pattern.
- Minimum 24 by 24 CSS pixel targets unless WCAG exception applies.
- No color-only state.
- Drag always has Move Up/Down/menu alternative.
- Active-window live regions announce stable outcomes, not every transient stage. Capture-only feedback while another app owns focus uses a Notification Center banner (`UNUserNotificationCenter` when Bronze is a `.app`; unbundled `tauri dev` uses signed `BronzeNotice.app` with the Bronze icon; catalog title and body only) plus AppKit `NSAccessibility.Notification.announcementRequested`; hidden WKWebView live regions are not relied upon. In-app `#chrome-notice` is viewport-fixed so a scrolled queue does not hide the result. Event-tap does not post notices. Announcement text is localized and priority is intentional. [Apple announcement API](https://developer.apple.com/documentation/appkit/nsaccessibility-swift.struct/notification/announcementrequested)
- Errors link field/control to recovery text and move focus only when necessary.
- Respect reduced motion, Increase Contrast, Differentiate Without Color, Reduce Transparency, dark/light, zoom/reflow.
- Native bridge reads `NSWorkspace.shared.accessibilityDisplayShouldReduceMotion`, `accessibilityDisplayShouldReduceTransparency`, `accessibilityDisplayShouldIncreaseContrast`, and `accessibilityDisplayShouldDifferentiateWithoutColor`. It observes `NSWorkspace.shared.notificationCenter` for [`accessibilityDisplayOptionsDidChangeNotification`](https://developer.apple.com/documentation/appkit/nsworkspace/accessibilitydisplayoptionsdidchangenotification), publishes one typed preference snapshot to every WebView, and reapplies tokens live. App overrides may strengthen active OS preference, never weaken it.
- Shortcut recorder exposes label, current value, Reset, Cancel, test result, and spoken key names.
- Biome a11y rules and axe catch regressions; manual AT matrix remains release gate.

Primary standards: [WCAG 2.2](https://www.w3.org/TR/WCAG22/), [EN 301 549 V3.2.1](https://www.etsi.org/deliver/etsi_en/301500_301599/301549/03.02.01_60/en_301549v030201p.pdf), [Apple accessibility testing](https://developer.apple.com/documentation/accessibility/performing-accessibility-testing-for-your-app), [WAI-ARIA dialog pattern](https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/).

## 13. Internationalization architecture

- Canonicalize stored/exchanged BCP 47 locale tags with `Intl.getCanonicalLocales`; reject invalid overrides.
- Resolve catalogs with RFC 4647-style progressive lookup that preserves script and variants before language fallback, e.g. `zh-Hant-HK` → `zh-Hant` → `zh` → `en`.
- Use react-i18next/i18next behind Bronze adapter with ICU integration, as accepted by ADR-014.
- No concatenated sentences or embedded UI prose.
- Stable semantic message IDs; build fails on missing messages or placeholder mismatch.
- Use Intl DateTimeFormat, NumberFormat, RelativeTimeFormat, ListFormat, Collator, and Segmenter.
- Store UTC timestamps and locale-neutral enums.
- Set UI document language and direction at runtime from the persisted locale catalog. Open WebViews reapply on `ui-locale-changed`. Native menus resolve the same glossary at launch and do not rebuild on a live switch.
- Use CSS logical properties and shadcn RTL transform.
- Portal direction, directional icons, animation direction, and third-party components need explicit tests.
- Store `content_language` per item as canonical BCP 47 or `und`, default to `und`, and never silently detect it. Render each user-content container with its own `lang`, direction auto, and bidi isolation; never translate or normalize it silently. Export/import and revisions preserve field.
- Localize native menu, permission explanation, InfoPlist strings, diagnostics labels, shortcut spoken names.
- Shortcut storage preserves semantic action plus physical/logical-key choice; displayed glyph/name recomputes for current keyboard layout.
- Input handlers honor composition events. Enter never submits while IME composition is active.

Test locales: English, Arabic RTL, Japanese, one Indic locale, 40 percent expansion pseudo-locale, bidi pseudo-locale. Corpus includes combining marks, emoji ZWJ sequences, embedded NUL rejection, mixed-direction code, and intentional leading/trailing whitespace.

References: [BCP 47](https://www.rfc-editor.org/info/rfc5646/), [ECMA-402 Intl](https://tc39.es/ecma402/), [Unicode MessageFormat](https://www.unicode.org/reports/tr35/tr35-messageFormat.html), [shadcn RTL](https://ui.shadcn.com/docs/rtl).

## 14. Security and privacy

Threats:

- Malicious captured/imported content attempts WebView script/navigation.
- Compromised WebView invokes privileged command.
- Hostile target app returns oversized/malformed AX value.
- Clipboard changes concurrently.
- Local database corruption or same-user process access.
- Supply-chain or signed-update compromise.
- Support diagnostics reveal content.

Controls:

- Plain-text rendering; no raw HTML or remote resources.
- Strict CSP and no remote navigation.
- Narrow per-window capabilities and typed Rust validation.
- Maximum payloads and pagination.
- Secure-field rejection before value query.
- Per-app exclusion before AX content query.
- No automatic clipboard restoration in P0; `changeCount` detects stable copied generation only.
- Local-only runtime; network instrumentation release gate.
- Stable signed identity, Hardened Runtime, notarization, SBOM, dependency audit.
- Content-free diagnostics and user-previewed support bundle.
- Database backups and transactional recovery.

App Sandbox is not enabled for v1 because required assistive/accessibility APIs conflict with sandbox constraints and PRD excludes Mac App Store distribution. Direct build still uses Hardened Runtime and minimal entitlements. [Apple App Sandbox guidance](https://developer.apple.com/documentation/security/protecting-user-data-with-app-sandbox)

Production must not carry get-task-allow, JIT, unsigned executable memory, disabled library validation, Screen Recording, microphone, or camera entitlement. Ad-hoc signatures are development-only. The title worker uses CPU inference (`n_gpu_layers=0`) and does not add JIT or unsigned-executable-memory entitlements. `llama-cpp-2` may still compile Metal on Apple Silicon.

## 15. Performance and reliability gates

| Measure | Gate | Maps to |
| --- | ---: | --- |
| Trigger callback work | p99 below 1 ms | CAP-002 |
| Local trigger acknowledgment | p95 at or below 100 ms | G-02 |
| AX capture through committed item | p95 at or below 300 ms | G-02 |
| Enabled clipboard fallback | p95 at or below 2 s | G-02 |
| Copy pasteboard write | p95 below 50 ms | QUE-004 |
| Ordinary DB mutation | p95 below 50 ms | DAT-001 |
| 10k-item filtered query | p95 below 100 ms on reference Mac | QUE-007 |
| Idle CPU | below 0.5 percent after settling | low interruption |
| Warm app steady RSS | target below 200 MB | low interruption |
| Supported-source capture success | at least 99.9 percent, zero silent drops | G-01 |
| 24-hour tap soak | zero timeout disable, duplicate trigger, leak trend | CAP-002 |
| Runtime network | zero unexpected requests | G-05, SEC-004 |

Reference Mac model, OS build, power mode, dataset, and measurement harness must accompany results. Budgets are release gates, not marketing claims.

## 16. Test architecture

| Layer | Tests |
| --- | --- |
| Swift unit | modifier FSM sequence/property tests, tap disable/re-enable, ABI ownership, permission adapters, AX provider fakes |
| Rust unit | domain transitions, order invariants, command idempotency, output serialization, redaction |
| Storage integration | every migration path, crash/disk-full/locked/corrupt DB, backup/restore, export/import corpus and fuzzing |
| React component | keyboard/focus, IME, i18n, RTL, pseudo-locales, axe |
| Tauri E2E | WebView journeys, IPC authorization/mocks, multiwindow capabilities |
| Signed native matrix | TCC grant/revoke, AX source apps, Secure Input, Spaces/full-screen/displays, sleep/wake, layouts, assistive technology |
| Release | ADR-002-selected architecture artifact (universal2 only if Intel supported), signature, notarization, Gatekeeper, SBOM, zero-network capture |

Current Tauri WebdriverIO service supports macOS through embedded driver; use it for WebView and IPC flows. It cannot establish honest TCC/AX/Secure Input evidence. [Tauri WebDriver](https://v2.tauri.app/develop/tests/webdriver/)

## 17. Release architecture

Distribution: direct Developer ID download, not Mac App Store.

Release pipeline:

1. Locked clean build environment.
2. Biome/typecheck/unit/integration/a11y gates.
3. Rust and Swift tests.
4. Build selected target architectures.
5. Generate checksums and SBOM.
6. Sign nested native content inside-out.
7. Sign app with stable Developer ID and Hardened Runtime.
8. Package/sign DMG or chosen artifact.
9. Submit with notarytool and staple.
10. Verify code signature, Gatekeeper assessment, staple, entitlements, bundle identity.
11. Install on clean VM; complete permission and upgrade-continuity smoke tests.
12. Verify zero unexpected network and no forbidden permission request.

References: [Apple notarization](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution), [Hardened Runtime](https://developer.apple.com/documentation/security/hardened-runtime), [Tauri macOS signing](https://v2.tauri.app/distribute/sign/macos/).

Stable bundle ID, Team ID, designated requirement, application path behavior, and signing order are release invariants because TCC continuity depends on identity. Identity is necessary, not a guarantee; signed upgrade tests and recovery UX remain mandatory. macOS range and universal2 policy remain ADR-002 gate.

## 18. Implementation invariants

Agents must preserve:

1. No capture request disappears or remains nonterminal.
2. No secure-field value is queried, stored, logged, or copied.
3. No normal Shift/key event is consumed or modified.
4. No WebView receives raw native capability.
5. No content enters diagnostics.
6. No lifecycle changes before corresponding side effect succeeds.
7. No user content is silently trimmed, normalized, translated, or sent.
8. P0 performs no automatic clipboard restoration; fallback never writes after its synthetic copy.
9. No migration destroys last known-good database.
10. No timing-dependent gesture is sole route.
11. No release changes bundle/signing identity without explicit migration decision.
12. No screen/audio/video capture dependency enters v1.

Violation blocks release and requires ADR if product intent changed.
