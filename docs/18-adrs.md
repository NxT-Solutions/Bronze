# Architecture decision records

Version: 1.0 planning baseline  
Decision date: 2026-08-27  
Authority: Accepted records bind implementation until superseded  
Related: [PRD](03-prd.md), [system architecture](06-system-architecture.md), [macOS capture reliability](07-macos-capture-reliability.md)

## 1. Status model

| Status | Meaning |
| --- | --- |
| Proposed | Decision gate remains. Agent may spike, measure, and document; must not silently make irreversible choice. |
| Accepted | Implementation contract. Change requires new ADR that supersedes this record. |
| Rejected | Evaluated and not selected. Reconsider only with materially new evidence. |
| Superseded | New ADR replaces decision; old record remains historical. |
| Deprecated | Still present during migration but prohibited for new work. |

An ADR change must name affected PRD IDs, migration impact, tests, distribution/TCC impact, and rollback.

## 2. Index

| ID | Decision | Status | Primary requirements |
| --- | --- | --- | --- |
| ADR-001 | Local selection-to-action queue; no recorder/account/sync | Accepted | G-05, SEC-004, product non-goals |
| ADR-002 | macOS minimum and architecture range | Proposed | G-01, SEC-005, WIN-002 |
| ADR-003 | Tauri/React/Rust monorepo stack | Accepted | all P0 |
| ADR-004 | In-process Swift native bridge | Accepted | CAP-001 through CAP-010, WIN-001 through WIN-004 |
| ADR-005 | Standard chord plus passive modifier event tap | Accepted | CAP-001 through CAP-004, A11Y-001, SET-002 |
| ADR-006 | AX-first selection with bounded clipboard fallback | Accepted | CAP-005 through CAP-009 |
| ADR-007 | Activating utility WebView window and native status menu | Accepted | WIN-001 through WIN-005, A11Y-001 through A11Y-005 |
| ADR-008 | Rust-owned SQLite, migrations, backup, deterministic export | Accepted | DAT-001 through DAT-004, QUE-001 through QUE-008 |
| ADR-009 | Protected App Group container for release data | Proposed | DAT-001 through DAT-004, SEC-005 |
| ADR-010 | Least-privilege Tauri IPC/capabilities and strict CSP | Accepted | SEC-001 through SEC-003 |
| ADR-011 | Direct Developer ID distribution without App Sandbox | Accepted | SEC-005, v1 constraints |
| ADR-012 | shadcn React Aria base | Accepted | A11Y-001 through A11Y-006 |
| ADR-013 | WCAG 2.2 AA plus EN 301 549 evidence model | Accepted | G-04, A11Y-001 through A11Y-006 |
| ADR-014 | BCP 47 and ICU catalog i18n architecture | Accepted | G-06, I18N-001 through I18N-004 |
| ADR-015 | Content-free diagnostics and user-previewed support bundle | Accepted | CAP-010, SEC-006, SUP-002 |
| ADR-016 | Semantic configurable-shortcut schema | Accepted | CAP-001, CAP-002, SET-001, SET-002, I18N-004 |
| ADR-017 | Packaged local UI with zero runtime network by default | Accepted | SEC-001, SEC-004, G-05 |
| ADR-018 | Locale-aware search/tokenizer semantics | Proposed | QUE-007, G-06, I18N-003 |
| ADR-019 | Portable local item titles (compact_title first, selectable offline GGUF refine) | Proposed | QUE-002, SEC-004, G-05 |

## ADR-001: Local selection-to-action queue

Status: Accepted

### Context

Bronze fills deliberate temporary workflow between source and destination. Expanding into recorder, passive clipboard manager, hosted AI, sync service, or task platform changes permission surface, privacy claim, data model, and product purpose.

### Decision

v1 implements:

    selected text/manual note
      → ordered local queue
      → deterministic copy output
      → explicit lifecycle

v1 has no:

- screen/audio/video recording;
- ScreenCaptureKit, AVFoundation, VideoToolbox;
- Screen Recording, microphone, or camera permission;
- passive clipboard history;
- account, cloud sync, hosted AI, analytics, telemetry;
- Windows/Linux build;
- Mac App Store distribution.

File/image attachments, OCR, sync, hosted actions, and other platforms remain P1/P2 only after separate threat model and ADR.

### Consequences

- Smaller TCC and entitlement surface.
- Core works offline.
- User moves text into AI tools by ordinary copy/paste; Bronze does not impersonate them.
- Competitor feature parity cannot justify scope creep.

### Verification

- Release entitlements contain no forbidden capture permission.
- Runtime network test sees zero unexpected connection.
- Dependency audit finds no recorder/cloud SDK.
- Product copy describes deliberate capture, not ambient monitoring.

## ADR-002: macOS minimum and architecture range

Status: Proposed  
Planning hypothesis: macOS 15 or later; Apple Silicon-first development

### Context

PRD requires Spike 0 choice based on evidence. Tauri uses OS WKWebView, so unsupported macOS stops receiving WebKit security fixes. macOS API behavior, TCC, AX compatibility, protected containers, Intel support, binary size, CI cost, and user reach differ by version/architecture. [Tauri WebView versions](https://v2.tauri.app/reference/webview-versions/)

### Options

1. macOS 15+, arm64 only.
2. macOS 15+, universal2.
3. Current supported macOS range, likely including macOS 14, universal2.

### Decision gate

Do not mark Accepted until Spike 0 records:

- Apple-supported OS set at release date.
- Tauri/WKWebView security status.
- Event tap, Input Monitoring, AX, Secure Input, pasteboard, Spaces/full-screen behavior per OS.
- Developer ID/TCC continuity per OS.
- Protected container availability and entitlements.
- Intel hardware/user need and CI access.
- Binary and Swift static-library universal build proof.

No production architecture default exists before gate. Select universal2 only if DG-01 commits to Intel support and test capacity; otherwise ship arm64 and state Apple Silicon-only support clearly. Development may begin arm64. Code must use availability checks and must not accidentally raise deployment target before gate.

### Consequences while Proposed

- CI/config uses one central minimum-version value.
- No marketing claim names supported range.
- Release is blocked until evidence matrix and ADR update exist.

### Reversal trigger

Apple security support change, Tauri minimum change, unavailable Intel CI, or measured native API incompatibility.

## ADR-003: Tauri, React, TypeScript, Rust, pnpm, and Turborepo

Status: Accepted

### Context

Product needs macOS-native privileged APIs, rich accessible text/queue UI, local storage, typed boundary, and later platform option without weakening macOS v1.

### Decision

- Tauri 2 shell using WKWebView.
- React and TypeScript UI.
- Rust workspace for domain, capture coordinator, storage, validation, diagnostics, and Tauri commands.
- pnpm workspace and Turborepo for JS/TS package graph.
- Biome for JS/TS/JSON/CSS format/lint.
- rustfmt and Clippy for Rust.
- SwiftPM and swift-format for native bridge.
- Vite-style packaged local frontend; no server-rendering/runtime web server.

Dependency versions lock at implementation start. Toolchain versions live in machine-readable root config. Signing/notarization/TCC tests never use Turbo cache.

### Consequences

- Three implementation languages, but clear ownership.
- Turbo does not replace Cargo or SwiftPM.
- Shared TypeScript contracts must derive from or be checked against Rust DTO definitions.
- Tauri OS WebView behavior remains release matrix concern.

### Rejected

- Electron: larger runtime and less direct alignment with requested stack.
- Pure web/PWA: cannot supply required AX/event-tap/menu-bar behavior.
- Next.js runtime: unnecessary server/runtime surface for local application.

### Verification

- Package boundary tests and dependency graph.
- Contract generation drift CI.
- No product behavior in packages/ui.
- No native framework imports outside macOS adapter.

## ADR-004: In-process Swift native bridge

Status: Accepted

### Context

Robust macOS implementation needs CGEventTap, Accessibility, AppKit activation/window/status behavior, NSWorkspace, and NSPasteboard. Pure Rust wrappers are possible but enlarge unsafe ownership/main-thread surface. Helper sidecar adds lifecycle, IPC, signing, and permission identity.

### Decision

Build BronzeNative Swift Package as static library linked into Tauri executable. Rust crate bronze-platform-macos exposes safe façade. Swift is not separate process and has no direct JavaScript surface.

Swift owns:

- event-tap run-loop and modifier FSM;
- AX query queue;
- AppKit main-thread operations;
- pasteboard and synthetic-copy cleanup;
- native permission probes;
- missing window/status-item platform adjustments.

Rust owns:

- request queue/state machine;
- domain and store;
- settings/provider policy;
- Tauri commands/capabilities;
- diagnostic redaction.

Bridge uses versioned C ABI, explicit buffer ownership, one-shot callbacks, fixed-width types, and no unwinding.

### Consequences

- One TCC/code-sign subject and lifecycle.
- Xcode/Swift required for macOS build.
- Universal2 requires both Swift and Rust architectures.
- ABI tests and thread annotations mandatory.
- Swift cannot become second domain implementation.

### Rejected

- Swift sidecar/helper.
- Raw native command surface exposed to JavaScript.
- Broad pure-Rust rewrite before reliability evidence.

### Verification

- ABI version check.
- Allocation/free and cancellation tests.
- Thread sanitizer/native stress run.
- Release bundle contains no unintended helper executable.
- Permission UI identifies one stable application.

## ADR-005: Standard chord plus passive modifier event tap

Status: Accepted

### Context

CAP-001 needs configurable standard chord. CAP-002 needs left/right modifier double tap, which ordinary accelerator registration cannot express. Double-tap timing is less accessible and Secure Input may suppress monitoring.

### Decision

- Registered global chord is standard route.
- Status menu/manual composer always exist.
- Optional double tap uses session CGEventTap with listenOnly.
- Observe only flagsChanged and, when robust cancellation enabled, keyDown reduced immediately to non-Shift-cancel signal.
- Never suppress, mutate, translate, persist, or log key stream.
- Trigger on second valid modifier release.
- Reset on invalid event, timeout, tap disable/re-enable, permission change, sleep/wake, session/input-device change.
- Double tap disabled by default; onboarding may offer.

Apple references: [event tap](https://developer.apple.com/documentation/coregraphics/cgevent/tapcreate%28tap%3Aplace%3Aoptions%3Aeventsofinterest%3Acallback%3Auserinfo%3A%29), [listenOnly](https://developer.apple.com/documentation/coregraphics/cgeventtapoptions/listenonly), [event timestamp](https://developer.apple.com/documentation/coregraphics/cgeventtimestamp), [Secure Keyboard Entry](https://support.apple.com/en-il/guide/terminal/trml109/mac).

### Consequences

- Input Monitoring permission applies only to modifier gesture.
- Secure Input may make gesture unavailable; no bypass.
- Callback latency/health becomes release gate.
- Timed gesture cannot be sole workflow under A11Y-001.

### Rejected

- Modifier polling.
- Keylogger-style broad capture.
- NSEvent global monitor as sole engine.
- IOHID interception.

### Verification

- FSM property suite.
- 1,000-pair physical test per mode.
- Zero false trigger corpus.
- Callback p99 below 1 ms.
- 24-hour soak without tap timeout/stuck state.

## ADR-006: AX-first selection and bounded clipboard fallback

Status: Accepted

### Context

Accessibility selected-text attributes provide deliberate selection without disturbing clipboard, but support varies across native, browser, PDF, terminal, and custom/Electron surfaces. Synthetic Cmd-C can broaden support but mutates global pasteboard and may fail under Secure Input.

### Decision

Provider chain:

1. Reject excluded app.
2. Resolve focused element and bounded, cycle-safe ancestor chain in snapshotted target.
3. Before content query at each relevant node, classify protection; reject known protected controls and fail closed on unknown protection.
4. Read kAXSelectedTextAttribute at each allowed node; empty/missing child is inconclusive.
5. Try selected range plus string-for-range parameterized attribute at each allowed node; only full-chain exhaustion is unsupported/no-selection.
6. Use bounded synthetic copy only when user enabled it for stable bundle ID after shared-clipboard disclosure.
7. Offer explicit Create from Clipboard.
8. Offer manual composer.

Synthetic copy:

- waits for physical modifiers up;
- records pasteboard generation and requires a stable post-copy read;
- guarantees Command/C key-up cleanup;
- uses bounded adaptive wait;
- writes no sentinel;
- performs no automatic restoration in P0 because `changeCount` is not atomic compare-and-swap;
- leaves captured selection on pasteboard and discloses possible Clipboard History, Universal Clipboard, and third-party retention.

References: [AX selected text](https://developer.apple.com/documentation/applicationservices/kaxselectedtextattribute), [AX selected range](https://developer.apple.com/documentation/applicationservices/kaxselectedtextrangeattribute), [secure text field](https://developer.apple.com/documentation/applicationservices/kaxsecuretextfieldsubrole), [NSPasteboard](https://developer.apple.com/documentation/appkit/nspasteboard), [changeCount](https://developer.apple.com/documentation/appkit/nspasteboard/changecount).

### Consequences

- Compatibility matrix defines “supported.”
- Clipboard fallback intentionally does not restore prior content in P0 and cannot retract history/sync/third-party copies.
- Default/manual path favors privacy and state safety over magical compatibility.
- Source-specific typed failures and local diagnostics required.

### Verification

- Mandatory source matrix.
- Secure/password/unknown-protection negative matrix across native, browser, Electron, and custom controls.
- Clipboard concurrency, stable-read, history/sync disclosure, and no-restoration tests.
- Exact Unicode/whitespace corpus.
- AX-to-committed capture p95 at or below 300 ms.

## ADR-007: Activating utility window and native status menu

Status: Accepted

### Context

Quick panel must receive keyboard, IME, VoiceOver, and focus reliably. Nonactivating floating panels reduce focus theft but complicate keyboard and accessibility. Capture target can be preserved by snapshotting before Bronze activation.

### Decision

- Warm hidden Tauri WebviewWindow capable of becoming key.
- Capture target and selection before showing it.
- Utility/floating level with tested Space/full-screen collection behavior.
- Accessory activation policy/Dockless default; configurable Dock icon.
- One native accessible status item/menu as fallback.
- Separate semantic resizable settings/library surface.
- Position using visible work area and selected display; clamp after display changes.
- Escape/hide restores prior focus where safe.

References: [NSStatusItem](https://developer.apple.com/documentation/appkit/nsstatusitem), [frontmost application](https://developer.apple.com/documentation/appkit/nsworkspace/frontmostapplication), [activation policy](https://developer.apple.com/documentation/appkit/nsapplication/activationpolicy-swift.enum).

### Consequences

- Panel activation is deliberate after capture, not during source query.
- Full-screen/Spaces/multi-display behavior needs physical matrix.
- Native menu strings and accessible labels require localization.
- Blur-hide cannot discard unsaved draft.

### Rejected

- Nonactivating NSPanel as primary editor.
- Transparent click-through overlay.
- Menu-only UI.

### Verification

- VoiceOver/Full Keyboard Access focus journey.
- IME composition test.
- Multiple display/scale/Dock/notch/full-screen/Spaces suite.
- Focus restoration tests.

## ADR-008: Rust-owned SQLite and deterministic portable export

Status: Accepted

### Context

QUE and DAT requirements need ordering, lifecycle, undo/trash, FTS, transactions, migrations, recovery, and portable archive. Browser storage or JSON rewrite lacks transactional/recovery strength. Exposing SQL to WebView breaks security boundary.

### Decision

- SQLite canonical store, WAL and foreign keys.
- One serialized writer; bounded reads.
- Rust repository is sole SQL owner.
- Versioned checksummed migrations.
- Integrity check and backup before migration/restore.
- FTS5 local search.
- Tombstone/trash and command/inverse undo journal.
- Deterministic versioned JSON archive and Markdown plus assets export.
- Atomic staged import/restore with preview and validation.
- Optimistic entity revision and command ID idempotency.

### Consequences

- Canonical storage may include database, WAL, SHM, and backups rather than one visible JSON file.
- FTS duplicates sensitive text locally and belongs in privacy/export/backup model.
- Guaranteed forensic erasure is impossible on SSD/WAL; DAT-004 is logical deletion/recovery contract.
- SQLite corruption/disk-full paths need first-class recovery UI.

### Verification

- Migration from every version.
- Crash injection, disk-full, locked, corrupt DB.
- Backup/restore drill.
- Export/import canonical corpus, fuzzing, checksum/path traversal tests.
- Deterministic ordering and timestamp output.

## ADR-009: Protected App Group container

Status: Proposed  
Fallback: user Application Support directory

### Context

Apple documents containers as stronger protection for local app data; macOS 15 adds relevant protection for non-sandboxed applications. Developer ID App Group entitlement, direct-distribution behavior, backup/restore, downgrade, and migration need signed validation. [Apple protected local containers](https://developer.apple.com/documentation/xcode/protecting-local-app-data-using-containers), [Application Support directory](https://developer.apple.com/documentation/foundation/url/applicationsupportdirectory)

### Proposed decision

Use signed App Group container for production database/backups if Spike DATA-CONTAINER-01 proves:

- Developer ID entitlement works without App Sandbox on supported OS range.
- Stable Team ID/group identifier.
- Upgrade and reinstall preserve access.
- Existing Application Support data migrates atomically and rolls back.
- Export/support can communicate path safely.
- Backup tools and user expectations are documented.

Otherwise use fixed Application Support subdirectory with mode restrictions.

### Consequences while Proposed

- Storage locator remains interface, not hard-coded path.
- No other crate/UI assumes concrete directory.
- Development builds may use Application Support.
- Release is blocked until decision accepted or fallback accepted explicitly.

### Verification

- Signed clean-install/update/reinstall tests.
- Ownership/mode inspection.
- Migration crash test.
- TCC/signing identity continuity.

## ADR-010: Least-privilege Tauri IPC and CSP

Status: Accepted

### Context

Captured/imported text is attacker-controlled. XSS or compromised WebView must not gain AX, event-tap, shell, filesystem, SQL, pasteboard-read, or arbitrary path capability. Tauri provides permissions, capabilities, runtime authority, and CSP.

### Decision

- WebView is untrusted presentation.
- Rust validates window label, capability, payload, length, enum, ID, revision, state.
- Separate local capabilities for quick, library, settings.
- No shell, raw fs, raw SQL, HTTP, WebSocket, upload, unrestricted opener, or remote-origin IPC.
- Captured content moves through scoped command/entity lookup, never global broadcast.
- Strict CSP: packaged self assets and exact required Tauri IPC only; no eval, frames, objects, remote assets.
- Release DevTools/navigation disabled.
- File dialogs and resolved paths remain native/Rust-owned.

References: [Tauri security](https://v2.tauri.app/security/), [permissions](https://v2.tauri.app/security/permissions/), [runtime authority](https://v2.tauri.app/security/runtime-authority/), [CSP](https://v2.tauri.app/security/csp/), [window capabilities](https://v2.tauri.app/learn/security/capabilities-for-windows-and-platforms/).

### Consequences

- More commands than broad plugins, but auditable.
- DTO generation/runtime validation required.
- New window/plugin needs threat review and capability update.
- Security patch upgrades are high priority.

### Verification

- Capability negative E2E tests from every window.
- Oversized/unknown/malicious payload tests.
- CSP/navigation test.
- Dependency/security advisory gate.
- Static check for forbidden plugins/features.

## ADR-011: Developer ID direct distribution without App Sandbox

Status: Accepted

### Context

v1 requires accessibility/assistive APIs and global input observation. App Sandbox constraints conflict with required behavior, and Mac App Store is explicit non-goal. Direct distribution still needs strong signing and runtime hardening.

### Decision

- No com.apple.security.app-sandbox entitlement in v1.
- Developer ID Application signing.
- Hardened Runtime.
- Minimal entitlements.
- Notarize with notarytool, staple, verify Gatekeeper.
- Stable bundle ID, Team ID, designated requirement.
- No get-task-allow, disabled library validation, JIT, unsigned executable memory in release.
- Sign nested content inside-out.
- Ad-hoc signatures development only.

References: [Apple App Sandbox guidance](https://developer.apple.com/documentation/security/protecting-user-data-with-app-sandbox), [notarization](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution), [Hardened Runtime](https://developer.apple.com/documentation/security/hardened-runtime), [Tauri macOS signing](https://v2.tauri.app/distribute/sign/macos/).

### Consequences

- No Mac App Store v1.
- Non-sandboxed process technically has broader user access; application code/IPC must stay narrow.
- Signed identity is functional dependency because TCC continuity can break on identity changes.
- Update mechanism, if later added, needs separate signed-network ADR.

### Verification

- Entitlement diff allowlist.
- codesign/Gatekeeper/notarization checks.
- Clean VM install.
- Signed upgrade TCC continuity matrix.
- Release contains no unintended helper/dylib.

## ADR-012: shadcn React Aria base

Status: Accepted

### Context

Earlier stack notes named Radix provisionally. Current shadcn supports React Aria as first-class base alongside Base UI and Radix. Bronze prioritizes keyboard, focus, international input, and assistive technology. [shadcn React Aria](https://ui.shadcn.com/docs/changelog/2026-07-react-aria)

### Decision

Initialize owned shadcn components with React Aria base. Use semantic native HTML where simpler. Generated source belongs to Bronze and is reviewed/tested; library choice is not conformance proof.

Avoid P0 contenteditable and virtualized core list. Composite widgets use documented keyboard patterns. Drag operations always have button/menu alternatives.

### Consequences

- Architecture supersedes provisional Radix wording in executive stack summary.
- Components copied from shadcn need local updates/audits.
- Mixed primitive bases prohibited without component-level ADR/reason.
- Portals, focus containment, direction, and live regions require Tauri WKWebView tests.

### Verification

- Component keyboard/focus/axe suite.
- Accessibility Inspector tree.
- VoiceOver, Voice Control, Switch Control.
- IME and RTL tests.

## ADR-013: Accessibility evidence model

Status: Accepted

### Context

“Fully WCAG proof” is not meaningful. WCAG covers web content; Bronze includes native menu/window, TCC permissions, global shortcuts, and OS interaction. Product needs scoped testable claim.

### Decision

- WebView target: WCAG 2.2 AA.
- Whole-app target: EN 301 549 V3.2.1 clauses 5, 11, 12 as applicable.
- Apple Accessibility Inspector/platform API evidence.
- VoiceOver, Full Keyboard Access, Voice Control, Switch Control release journeys.
- Publish ACR/VPAT-style criterion matrix and known limitations.
- Independent audit before broad public conformance claim.
- Double modifier gesture is enhancement; all core operations have non-timed route.

Sources: [WCAG 2.2](https://www.w3.org/TR/WCAG22/), [EN 301 549 V3.2.1](https://www.etsi.org/deliver/etsi_en/301500_301599/301549/03.02.01_60/en_301549v030201p.pdf), [Apple accessibility testing](https://developer.apple.com/documentation/accessibility/performing-accessibility-testing-for-your-app).

V3.2.1 is harmonized baseline at decision date. V4.1.0 approval draft is tracked, not silently adopted; implementation/release rechecks ETSI and EU harmonization status.

### Consequences

- Automated axe/Biome results are necessary, insufficient.
- Native and WebView semantics both release-gated.
- Conformance exceptions/known limitations remain visible.
- Accessibility work is feature acceptance, not final polish phase.

### Verification

- Criterion-to-test/evidence matrix complete.
- P0 journey passes keyboard and AT.
- 200/400 percent zoom, contrast, motion, transparency, target size, RTL.
- No open severity-0/1 accessibility defect.

## ADR-014: BCP 47 and ICU internationalization

Status: Accepted

### Context

P0 requires no hard-coded strings, plurals/select, locale formatting, per-app language, RTL, keyboard layouts, IME, long expansion, and grapheme-safe behavior.

### Decision

- Canonical BCP 47 locale identifiers and RFC 4647-style progressive fallback preserving script/variant subtags before language and English.
- react-i18next/i18next behind Bronze localization adapter.
- ICU MessageFormat through i18next ICU integration.
- Stable semantic message IDs; no sentence concatenation.
- Intl for dates, numbers, relative time, lists, collation, segmentation.
- UTC storage and locale-neutral enums.
- UI document lang/dir, CSS logical properties, shadcn RTL transform.
- Per-item canonical BCP 47 or `und` content language, default `und`, user override, and no silent language detection; direction-auto/bidi isolation for user content.
- Native menu and InfoPlist localization use same glossary/message ownership.
- Build fails on missing keys/placeholder mismatch.
- Pseudo-locales, Arabic RTL, CJK/Indic IME are CI/release gates.

Sources: [BCP 47/RFC 5646](https://www.rfc-editor.org/info/rfc5646/), [RFC 4647 lookup](https://www.rfc-editor.org/rfc/rfc4647.html), [ECMA-402](https://tc39.es/ecma402/), [Unicode MessageFormat](https://www.unicode.org/reports/tr35/tr35-messageFormat.html), [shadcn RTL](https://ui.shadcn.com/docs/rtl).

### Consequences

- User text remains untouched and untranslated.
- Native/React catalogs need extraction and parity tooling.
- Shortcut glyph and spoken name are separate localized representations.
- Search/collation behavior needs locale-aware tests without changing canonical content.

### Verification

- Missing-key/placeholder CI.
- 40 percent expansion and bidi pseudo-locales.
- Arabic, Japanese, Indic smoke suites.
- Portal direction/icon/animation audit.
- Grapheme/emoji/combining-mark corpus.

## ADR-015: Content-free diagnostics

Status: Accepted

### Context

Reliability target needs stage evidence, but selected text, clipboard, URLs, titles, paths, and keys may contain secrets. No telemetry/account means diagnostics stay local.

### Decision

Diagnostics may contain:

- request/diagnostic ID;
- timestamps and stage durations;
- trigger/provider/result enums;
- permission snapshot;
- build/schema version;
- source bundle ID only under policy;
- queue/tap health counters;
- store result code.

Diagnostics never contain:

- selected/item/clipboard text or excerpt;
- raw keycode/character stream;
- secure-field content length/hash;
- window/document title or URL;
- AX tree/description;
- user document path;
- SQL bound values.

Support bundle generates locally, presents full preview, and exports only after consent. No automatic upload.

Diagnostic persistence is independent from content transactions. Coordinator keeps bounded typed in-memory terminal status when diagnostic database writes fail; a diagnostic failure never rolls back a successfully saved item, and database failure can still be reported without writing that same failed database.

### Consequences

- Some defects require user-run local self-test instead of content trace.
- Redaction must happen at type/schema boundary, not log formatter.
- Framework errors become safe enums; any content-free causal chain is in-memory only and never persisted or exported.
- Exported `bronze-support.txt` is a sectioned English report (versions, machine, redacted data path, permission snapshot, login-item status, title-engine phase, queue counts, excluded bundle IDs, recent source apps, last-hour events and copy attempts). Help preview matches export. Unredacted `/Users/<name>` paths fail the secret scan.

### Verification

- Secret-fixture log scan.
- Type-level diagnostic schema excludes content types.
- Snapshot tests for every result code.
- Support preview/export parity.
- Network test confirms no upload.

## ADR-016: Semantic configurable-shortcut schema

Status: Accepted

### Context

String accelerators lose physical/logical intent, modifier side, layout ambiguity, localization, double-tap timing, and future migration. Registration conflicts can leave user without trigger.

### Decision

Store versioned semantic shortcut:

~~~text
action
trigger form
modifier set and optional side
physical or logical key mode
physical code or logical key
double-tap thresholds
enabled
schema version and revision
~~~

Registration is transactional:

- validate;
- try register;
- offer user-ended/adjustable test; mark skipped binding untested without blocking motor/Slow Keys users;
- persist;
- unregister old.

Failure retains old shortcut. Standard chord/status menu/manual route cannot all be disabled simultaneously. UI displays platform glyph plus localized spoken text.

### Consequences

- Platform adapter maps semantic model to Tauri/native registration.
- Layout/input-source changes recompute label and warning.
- Export/import migrates schema rather than opaque string.
- Some system conflicts remain unknowable before test.

### Verification

- Layout matrix.
- Registration rollback/conflict tests.
- Shortcut migration tests.
- VoiceOver announcement and recorder keyboard tests.
- At least one accessible route always enabled invariant.

## ADR-017: Packaged local UI and zero runtime network

Status: Accepted

### Context

SEC-001 and SEC-004 require no remote code/content and no runtime network by default. Captured content is sensitive; remote fonts/assets, update checks, crash reporting, or embedded pages weaken promise and CSP.

### Decision

- Bundle all scripts, styles, fonts, icons, locale data.
- No HTTP/WebSocket/upload client in production-default feature set.
- No remote images, embeds, iframes, analytics, crash upload, AI API, account endpoint.
- External help/source links open in default browser only after allowlist/confirmation; never render inside privileged WebView.
- Runtime network capture is release gate.
- Future updater/cloud/action requires separate opt-in design, signed protocol, threat model, capability, CSP, privacy UI, and superseding ADR.

### Consequences

- Help and documentation needed for core use ship locally.
- Security advisories/releases are user-initiated until updater ADR.
- Dependency features must be audited to prevent implicit network.
- CSP can remain narrow.

### Verification

- Offline end-to-end suite.
- Packet/DNS instrumentation on clean run and all P0 journeys.
- Static dependency/feature inspection.
- CSP and navigation negative tests.
- No remote URL in production asset graph.

## ADR-018: Locale-aware search/tokenizer semantics

Status: Proposed  
Planning gate: SEARCH-01

### Context

SQLite FTS5 configuration determines token boundaries, case folding, diacritics, prefixes, and substring behavior. Default tokenizer cannot be assumed correct across Latin, Arabic, Hebrew, CJK, Indic scripts, combining marks, emoji, and source code. QUE-007 and G-06 need defined query semantics, not merely a fast index.

### Decision gate

Before acceptance, build locale-tagged golden corpus mapping query to expected ordered results. Compare `unicode61`, trigram, application-side segmentation, locale-specific indexes, storage/size, privacy, and performance. Decide:

- normalization/case/diacritic policy for derived index only;
- word, prefix, and substring behavior;
- CJK and no-space-script segmentation;
- mixed code/identifier behavior;
- locale fallback and index-rebuild migration;
- deterministic ranking and 10k-item budget.

Raw item content remains unchanged. Search diagnostics never store query/body. Purge removes source row and rebuilds derived FTS state.

### Verification before Accepted

- Golden corpus for English, Dutch/German, Arabic/Hebrew, Japanese/Chinese/Korean, Indic, combining marks, emoji, and code identifiers.
- Query/result accessibility and locale sorting tests.
- 10k-item performance/storage benchmark.
- Migration/rebuild/purge and corruption recovery tests.
- Threat review for duplicated sensitive FTS data.

## ADR-019: Portable local item titles (compact_title first, selectable offline GGUF refine)

Status: Proposed  
Planning gate: TITLE-01

### Context

Captured bodies can be long. The inbox and status-menu overview need a title that is not the first sentence by policy. First-sentence-only is not the shipped title. Hosted AI, Private Cloud Compute, and a runtime Hugging Face Hub download violate ADR-017 and SEC-004. An OS-AI entitlement (`SystemLanguageModel`, NLEmbedding, Foundation Models) is not portable to Linux or Windows. T-10 allows only an offline SHA-256-pinned GGUF allow-list plus extractive `compact_title`; it does not allow hosted AI, PCC, or runtime fetch. ADR-017 remains Accepted. ADR-002, ADR-009, and ADR-018 remain Proposed.

### Decision gate

Do not mark Accepted until the operator formally accepts this title contract. The path below is a candidate. It must not silently become the Accepted contract.

Candidate path:

- Persist `compact_title` first: term-frequency best sentence over significant terms, 40-character word-boundary clamp, markup stripped, no ellipsis glyph. Not first-sentence-only. The clamp fits one title row in the 400px Quick Panel card at `--text-body` 0.9375rem. Composer add and body edit write the same function before any refine.
- After persist (and after composer add / body edit), a Rust `llama-cpp-2` 0.1.156 worker on thread `bronze-title-model` / `bronze-item-title` may refine the stored title. Inference is not on AppKit main and not in the event-tap. `TitleABI.swift` stays a compile-only stub (`BRONZE_STATUS_DEGRADED`). Linux and Windows call the same Rust function.
- Settings `general.titleModel` is a schema-valid, exportable, non-secret id: `extractive` | `smol-135` | `smol-360` | `qwen-05`. `extractive` uses `compact_title` only (no GGUF). A Settings change unloads the previous llama context and loads the chosen file on that worker for the next refine; capture never waits on the load. Reload failure stays on `compact_title` and surfaces the fallback reason. No app relaunch is required when load succeeds. Existing `bronze-title:` stages (`switch scheduled`, `weights resolved`, `hash ok`, `model loaded`, `missing_weights`, plus load fallbacks `bad_hash` / `timeout` / `unreadable`) map to a Settings-only snapshot. Command `title_engine_status` returns `{ tier, phase, reason }`. Event `title-engine-status` publishes the same DTO from the title worker, never from the event-tap. Phases: idle (extractive), loading, hashing, ready, missing, failed. Refine logs do not flip a ready engine to failed.
- Auto-pick runs only when `general.titleModel` is missing or empty (first install / setup). It does not overwrite a stored user value. Rust reads physical RAM locally with no telemetry. Bands: missing RAM, `< 8 GiB`, or no GGUF on disk → `extractive`; `8–16 GiB` → `smol-135` if present else extractive; `16–32 GiB` → `smol-360` if present else the next smaller present file; `≥ 32 GiB` → `qwen-05` if present else the next smaller present file. Most fit is the largest tier the RAM band can hold whose file is already on disk. Bronze never downloads to honor the recommendation. The resolved id is persisted so the Settings picker shows it.
- Allow-listed weights, each vendorable with `bronze-title-model/scripts/vendor-gguf.sh <id>` (or `all`). Load only a verified path (`set_weights_dir` / `set_weights_path`, else `BRONZE_TITLE_WEIGHTS`, else crate `vendor/`, else exe-relative `models/`). Never `-hf`. Never Hub at capture, first launch, or Settings switch. Never a WebView path (SEC-003). Weights are not user-writable via the app.
  - `smol-135`: HuggingFaceTB/SmolLM2-135M-Instruct, bartowski `SmolLM2-135M-Instruct-Q4_K_M.gguf`, revision `09816acd5d99df7be770d85ea30822623dab342c`, SHA-256 `2e8040ceae7815abe0dcb3540b9995eaa1fa0d2ca9e797d0a635ae4433c68c2d` (~105 MB, Apache-2.0).
  - `smol-360`: HuggingFaceTB/SmolLM2-360M-Instruct, bartowski `SmolLM2-360M-Instruct-Q4_K_M.gguf`, revision `ab928a97ee49f3a015f35194879f68211291d6ca`, SHA-256 `2fa3f013dcdd7b99f9b237717fa0b12d75bbb89984cc1274be1471a465bac9c2` (~271 MB, Apache-2.0).
  - `qwen-05`: official `Qwen/Qwen2.5-0.5B-Instruct-GGUF` `qwen2.5-0.5b-instruct-q4_k_m.gguf`, revision `9217f5db79a29953eb74d5343926648285ec7e67`, SHA-256 `74a4da8c9fdbcd15bd1f6d01d621410d31c6fc00986f5eb687824e7b93d7a9db` (~491 MB, Apache-2.0). Not Qwen3 thinking mode.
- Inference: greedy / low temperature; truncate input to 2048 characters; `max_new_tokens` 16; no tools; stop on newline; clamp with the same 40-character word-boundary function; English prompt v3 (short topic, no sentence copy, no `Title:` prefix). `clean_title` strips a leading `Title:` / `title:` (optional space) and surrounding quotes before clamp and groundedness. Skip refine when the body is already at most 40 characters, or when the candidate shares no 3+ character term with the body. A refine can still copy a source sentence; groundedness only rejects titles with no shared 3+ character term.
- Missing file, hash mismatch, first-load or generate timeout, empty output, load failure, or an ungrounded candidate keeps `compact_title`. Missing weights log `bronze-title: missing_weights`. Settings marks that tier unavailable and shows the vendor command; there is no download button. Capture ID and terminal result are never dropped.
- CPU path only (`n_gpu_layers=0`). No `allow-jit` or `allow-unsigned-executable-memory` entitlement. `llama-cpp-2` may still compile Metal on Apple Silicon; the worker does not offload.
- No Needle, Gemma, Llama 1B, Qwen3-thinking, hosted AI, Private Cloud Compute, `SystemLanguageModel`, NLEmbedding, URLSession, or OpenAI on the title path.

### Consequences while Proposed

- Persist, composer add, and edit always store `compact_title` before any refine.
- Refine is best-effort and local. Failure leaves the extractive title in place.
- Private Cloud Compute remains forbidden. Release copy must not treat this as an Accepted inference contract.
- Queue chrome does not show Apple Intelligence or model-status copy. Titles clip without an ellipsis glyph. A leaked `Title:` label is stripped before the card shows the heading.
- Settings → General Title engine shows a semantic spinner (`span.title-engine-spinner`) next to the select while phase is loading or hashing, and `#title-model-status` (`role=status`, `aria-live=polite`) shows catalog copy. Extractive stays idle with no spinner. Missing weights keep the vendor-command copy. Ready uses catalog `settings.field.titleModel.loaded`. Failed reasons are catalog strings (`bad_hash`, `timeout`, `unreadable`) with no huge paths. Focus stays on the select. Reduce Motion stops decorative spin; text still updates. This is not a WCAG or VoiceOver claim.

### Verification

- `compact_title` tests: a middle content-bearing sentence wins over a greeting; the clamp is 40 characters without an ellipsis glyph.
- Persist writes `compact_title` even when the GGUF is missing, the hash mismatches, or the worker fails.
- `clean_title` turns `Title: Landing Page Change Hasimproved` into `Landing Page Change Hasimproved` before clamp and groundedness.
- Auto-pick does not overwrite a stored `general.titleModel`. Missing `qwen-05` does not fetch.
- Title-worker sources have no runtime network. Capture ID and terminal result are never dropped.
- Swift source-scan of `TitleABI.swift`: no `PrivateCloudCompute`, `URLSession`, `openai`, `llama`, `gguf`, `SystemLanguageModel`, or `NLEmbedding`.
- Packaging asserts no `allow-jit` or `allow-unsigned-executable-memory`.
- Live binary does not link FoundationModels or NaturalLanguage.
- `apply_diag` maps `switch scheduled` → loading; `weights resolved` / `hash ok` → hashing; `model loaded` → ready; extractive → idle; `missing_weights` → missing; `bad_hash`, `timeout`, or `unreadable` during load → failed.
- Settings shows spinner plus catalog Loading `{engine}` while loading; no spinner and loaded copy when ready; no spinner and vendor-command copy when missing.
- Event `title-engine-status` and command `title_engine_status` stay Settings-capability only. Capture never waits on load.

## 3. Decision-change checklist

Before superseding Accepted ADR:

1. Name new user/product evidence.
2. List affected PRD IDs and update them if product behavior changes.
3. Describe data, IPC, permission, entitlement, accessibility, i18n, signing, TCC, and migration impact.
4. Provide alternative analysis.
5. Define rollback and backward compatibility.
6. Add acceptance tests before implementation.
7. Update [system architecture](06-system-architecture.md) and [capture reliability](07-macos-capture-reliability.md) when relevant.
8. Preserve historical record with Superseded link; never rewrite history silently.
