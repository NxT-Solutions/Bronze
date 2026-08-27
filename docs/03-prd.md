# Product requirements document

Version: 1.0 planning baseline  
Status: ready for architecture spikes  
Platform: macOS only  
Working name: Bronze

## 1. Problem

Knowledge workers using AI chats, browsers, editors, terminals, documents, and native apps repeatedly encounter text they need later in same working session. Current choices interrupt flow: switch to task manager, add permanent note, rely on clipboard history, or remember it. Passive clipboard tools capture too much; automation tools need setup; scratchpads lack ordered copy-back lifecycle.

Bronze creates fast, deliberate, local holding queue between source and destination.

## 2. Jobs to be done

- When I see useful selected text, capture it without losing focus so I can return later.
- When future instruction comes to mind, queue it without sending it early.
- When ready, combine queued context and prompts in known order and paste into active tool.
- When something fails, know which stage failed and how to recover.
- When app handles private material, prove data remains under my control.
- When I use assistive technology, alternate keyboard layout, IME, or larger text, access full workflow without workaround.

## 3. Personas

| Persona | Need | Risk |
| --- | --- | --- |
| AI-assisted developer | gather errors, code, constraints, next prompts across editor/browser/terminal | code whitespace loss, terminal/Secure Input limitations |
| Researcher/writer | collect excerpts with source, order arguments, copy bundle | provenance loss, long text, rights/privacy |
| Keyboard-first power user | near-zero-friction triggers and programmable output | shortcut collisions, layout mismatch |
| Assistive-technology user | semantic panel, non-timing route, clear feedback | gesture-only workflow, inaccessible custom widgets |
| Privacy-sensitive user | explicit capture, exclusions, local backup | hidden network traffic, clipboard corruption, secret capture |

## 4. Principles

1. Deliberate over ambient.
2. Queue over archive.
3. Native reliability over portability abstraction.
4. Observable failure over silent magic.
5. Accessible alternatives over gesture dependence.
6. Local and portable over lock-in.
7. Original product over clone.

## 5. Goals and success metrics

| ID | Goal | Release target |
| --- | --- | --- |
| G-01 | Reliable capture | ≥99.9% successful supported-source captures in release matrix; zero silently dropped requests |
| G-02 | Fast feedback | local trigger acknowledgment ≤100 ms p95; AX capture-to-saved ≤300 ms p95; fallback ≤2 s p95 |
| G-03 | Recoverable data | zero known destructive migrations; backup/restore and export/import round trips pass corpus |
| G-04 | Accessible core | P0 journeys pass keyboard, VoiceOver, Full Keyboard Access, Voice Control, and Switch Control; every WCAG 2.2 A/AA criterion accounted for; relevant EN 301 549 clauses 5, 11, and 12 matrix complete |
| G-05 | Local privacy | zero runtime network requests in production-default build; no telemetry/account; exclusion and secure-field tests pass |
| G-06 | Global readiness | no hard-coded UI strings; pseudo-locales, RTL, CJK IME, and long-string suite pass |
| G-07 | Low interruption | median capture user action ≤1 second after selection in usability study |

Targets are quality gates, not telemetry requirements. Measure on test fixtures and opt-in local diagnostics; do not collect user behavior remotely.

## 6. P0 requirements

### Capture and input

- **CAP-001** User can capture selected plain text from supported macOS apps using configurable standard global chord.
- **CAP-002** User can optionally enable left/right modifier double-tap trigger with adjustable timing and side/action mapping.
- **CAP-003** Menu-bar command and manual composer provide non-global-hook alternatives.
- **CAP-004** Every capture request is queued, assigned ID, processed once, and ends as saved/rejected/failed/cancelled with reason.
- **CAP-005** AX selected-text retrieval is primary; clipboard simulation is bounded fallback.
- **CAP-006** Secure/password fields are never captured; user receives private, non-content error.
- **CAP-007** Captured text preserves exact Unicode and intentional leading/trailing whitespace by default.
- **CAP-008** Optional provenance records app bundle ID/name, safe window/document title, URL only when available and enabled, and timestamp.
- **CAP-009** User can exclude applications and disable provenance per app.
- **CAP-010** Capture diagnostics expose permission, trigger, selection provider, fallback, persistence, and feedback stages without storing captured content.

### Queue

- **QUE-001** User can create, rename, reorder, collapse, archive, and choose active section.
- **QUE-002** User can add, edit, duplicate, reorder, move, select, and delete text items.
- **QUE-003** Item states are `queued`, `copied`, `active`, `done`, `skipped`, `trashed`; automatic transitions are configurable and undoable.
- **QUE-004** User can create, duplicate, edit, reset, and select named output profiles for plain text, Markdown list, numbered list, or prompt block, with exact preview and bounded format options.
- **QUE-005** User can copy-and-advance and optionally return focus to prior target, but completion remains explicit by default. P0 never synthesizes paste.
- **QUE-006** Undo restores last destructive or lifecycle action; trash retention protects accidental deletion.
- **QUE-007** Search covers item content, sections, and safe provenance; search is local.
- **QUE-008** Empty, loading, capture failure, storage read-only/recovery, and migration states provide recoverable UI.

### Panel and app shell

- **WIN-001** User can summon/hide panel on current pointer or active-app display and chosen screen edge.
- **WIN-002** Panel respects work area, menu bar, Dock, notch, Spaces, full-screen apps, multiple displays, and scale factors.
- **WIN-003** Panel can remain visible or hide on blur per setting and restores focus predictably.
- **WIN-004** Menu-bar presence, Dock icon, launch at login, always-on-top, translucency, density, width, and edge are configurable where platform permits.
- **WIN-005** Settings is a semantic, resizable window; panel never traps focus.

### Data, privacy, security

- **DAT-001** Store uses local SQLite with transactional versioned migrations and integrity checks.
- **DAT-002** App creates automatic local backups and supports verified restore.
- **DAT-003** Deterministic JSON archive and Markdown+assets export/import preserve order, states, timestamps, per-item content language, and provenance choices.
- **DAT-004** Data deletion is local, explicit, scoped, and recoverable within configured trash period.
- **SEC-001** Production UI loads no remote code, fonts, scripts, or content; strict CSP applies.
- **SEC-002** Tauri capabilities are least-privilege per window; IPC inputs/outputs are typed and validated in Rust.
- **SEC-003** WebView cannot request arbitrary file reads, shell execution, recursive quarantine changes, or unrestricted paths.
- **SEC-004** App has no account, telemetry, analytics, hosted AI, or runtime network by default.
- **SEC-005** Releases are reproducible as practical, signed/notarized, checksummed, SBOM-backed, dependency-audited, and preserve TCC identity.
- **SEC-006** Diagnostics redact content and secrets by construction; support bundle is previewed before export.

### Accessibility and i18n

- **A11Y-001** Entire core workflow works with keyboard without timing-dependent gesture.
- **A11Y-002** Semantic roles, names, descriptions, values, states, relationships, live status, focus order, and focus restoration are programmatically exposed.
- **A11Y-003** UI supports 200% text resize and 400% zoom/reflow at 320 CSS px without lost content or functionality, plus Increase Contrast, Differentiate Without Color, Reduce Motion, Reduce Transparency, dark/light, and no color-only information.
- **A11Y-004** Reorder has button/menu alternatives; drag is never required.
- **A11Y-005** VoiceOver, Full Keyboard Access, Voice Control, and Switch Control core journeys are release-tested.
- **A11Y-006** Errors identify problem and recovery; capture results use visible plus announced feedback.
- **I18N-001** All UI strings use localized message catalog with plural/select formatting; no concatenated sentences.
- **I18N-002** Locale uses canonical BCP 47, system default, per-app override, and script-preserving progressive fallback; dates/numbers/lists use `Intl`.
- **I18N-003** Layout supports RTL, long expansion, CJK/Indic input, emoji, grapheme-safe editing, and Unicode normalization policy; each item preserves user-overridable BCP 47 or `und` content-language metadata.
- **I18N-004** Shortcut display uses localized modifier/key names while storage preserves semantic intent and physical-key details where needed.

### Settings and support

- **SET-001** Settings are searchable, grouped, keyboard accessible, resettable per field/group/all, and exportable without app credentials, permission tokens, diagnostics, or machine paths; preview identifies user-entered literals and app policies that may themselves be sensitive.
- **SET-002** Shortcut editor detects internal duplicates, reserved/system conflicts when knowable, invalid modifier-only patterns, and layout ambiguity before save.
- **SET-003** Permission health distinguishes Input Monitoring, Accessibility, Automation if introduced, and capture-pipeline self-test.
- **SET-004** App explains why each permission is needed and offers deep-link/instructions, retest, and alternative workflow.
- **SUP-001** Built-in help documents supported sources, limitations, privacy, backup/restore, shortcuts, and accessibility.
- **SUP-002** About/diagnostics shows version, bundle identity, data path, schema version, permission health, and redacted recent failures.

## 7. P1

- Context bundles and nested prompt blocks without arbitrary deep task hierarchy.
- Safe richer provenance and source reopen when supported.
- Merge/split items; reusable output templates.
- Services, Shortcuts/App Intents, URL scheme, CLI, PopClip, Raycast, Alfred integrations.
- Importers for Cooper and common Markdown/JSON structures.
- Local retention rules, archive analytics visible only to user, and secret-pattern warnings.
- Theme editor within validated contrast bounds.

## 8. P2

- Images/files and on-device OCR.
- Reusable prompt-template library and variables.
- Optional end-to-end encrypted sync after separate threat model.
- Sandboxed extension/action system.
- Bring-your-own local model or explicit cloud actions.
- Windows/Linux after macOS quality targets sustain across two stable releases.

## 9. Non-goals

- Passive unlimited clipboard history.
- Screen recording, keylogging, or complete input capture.
- Full task management, calendar, reminders, collaboration, or project planning.
- Full note vault/PKM graph.
- Hosted AI assistant or mandatory AI service.
- Mandatory account/cloud sync.
- Pixel clone of Copper or fork of Cooper.
- Mac App Store distribution for v1.

## 10. Assumptions and constraints

- v1 targets supported current macOS range chosen during Spike 0; minimum cannot be selected solely from Copper claim.
- Distribution outside Mac App Store through Developer ID signed/notarized artifact.
- Input Monitoring likely needed for listen-only event tap; Accessibility needed for AX/synthetic input paths. Validate on every supported macOS version.
- Some apps do not expose selection through AX or permit simulated copy. “Supported” means matrix-tested provider path with documented limitation.
- User content may be sensitive. No content logs, crash attachments, or support exports without explicit preview/consent.

## 11. Release definition

v1 ships only when all P0 requirements have implemented tests or documented manual evidence, no open severity-0/1 defects, capture matrix meets target, signed upgrade preserves permissions, backup/restore drill succeeds, accessibility conformance matrix is reviewed, and network-capture test shows zero unexpected traffic.
