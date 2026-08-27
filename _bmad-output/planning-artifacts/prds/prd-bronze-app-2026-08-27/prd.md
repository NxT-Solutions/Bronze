---
title: Bronze
status: final
created: 2026-08-27
updated: 2026-08-27
---

# PRD: Bronze

Working name confirmed: **Bronze**. Platform: macOS only.

## 0. Document Purpose

This PRD is for PM, architecture, UX, and implementation owners of Bronze. It is a BMAD-shaped hydrate of the approved Bronze product requirements in `docs/03-prd.md` (authority also in `docs/00-executive-summary.md` and `docs/21-preimplementation-reconciliation.md`). Glossary-anchored vocabulary is taken from `research/glossary.md`. Features group P0 requirements with those IDs nested as functional requirements — IDs are not renumbered. UX, architecture, security, and ADRs already exist; this PRD points to them and does not duplicate them (`docs/05-ux-ui-interaction-spec.md`, `docs/06-system-architecture.md`, `docs/09-security-privacy-threat-model.md`, `docs/18-adrs.md`).

## 1. Vision

Knowledge workers using AI chats, browsers, editors, terminals, documents, and native apps repeatedly encounter text they need later in the same working session. Current choices interrupt flow: switch to a task manager, add a permanent note, rely on clipboard history, or remember it. Passive clipboard tools capture too much; automation tools need setup; scratchpads lack an ordered copy-back lifecycle.

Bronze is a deliberate local-only selection-to-action queue: selected text → capture → optional prompt item or note item → ordered queue → copy to target → explicit completion. Continuity is the product: park useful context and future prompts without switching into a full task manager or passively recording clipboard history.

Principles: deliberate over ambient; queue over archive; native reliability over portability abstraction; observable failure over silent magic; accessible alternatives over gesture dependence; local and portable over lock-in; original product over clone.

Positioning: offline, local-first, accessibility-first macOS selection-to-action queue for AI and human workflows.

## 2. Target User

### 2.1 Jobs To Be Done

- When I see useful selected text, capture it without losing focus so I can return later.
- When a future instruction comes to mind, queue it without sending it early.
- When ready, combine queued context and prompts in known order and paste into the active tool.
- When something fails, know which stage failed and how to recover.
- When the app handles private material, prove data remains under my control.
- When I use assistive technology, an alternate keyboard layout, IME, or larger text, access the full workflow without workaround.

### 2.2 Non-Users (v1)

- People who want passive unlimited clipboard history.
- People who want screen recording, keylogging, or complete input capture.
- People who want a full task manager, calendar, reminders, collaboration, or project planner.
- People who want a full note vault or PKM graph.
- People who want a hosted AI assistant or mandatory AI service.
- People who require account or cloud sync.
- Windows or Linux users in v1.

### 2.3 Key User Journeys

- **UJ-1. Developer parks compiler output without leaving the editor.**
  - **Persona + context:** AI-assisted developer gathering errors, code, constraints, and next prompts across editor, browser, and terminal.
  - **Entry state:** selected text in a supported macOS app; Bronze running local-only; a section is active capture destination.
  - **Path:** uses a trigger (standard global chord, or menu-bar command if the chord is unavailable); a capture request is queued and assigned an ID; AX retrieves selected text (synthetic fallback only if bounded and enabled); context item is saved on the active section; later adds a prompt item; copies through an output profile.
  - **Climax:** ordered context plus prompt lands on the clipboard for the destination tool; completion stays explicit.
  - **Resolution:** items remain in lifecycle states the developer controls; focus can return to the prior target.
  - **Edge case:** Secure Input or a terminal that does not expose selected text — capture request ends rejected/failed with a private, non-content reason; menu-bar command and manual composer remain available. Realizes CAP-001, CAP-003, CAP-004, CAP-005, CAP-006, QUE-004, QUE-005.

- **UJ-2. Researcher collects excerpts with source and copies an ordered bundle.**
  - **Persona + context:** Researcher/writer collecting excerpts, ordering arguments, copying a bundle without losing provenance.
  - **Entry state:** selected text in a browser or document; provenance enabled for that app unless excluded.
  - **Path:** capture into a named section; optional provenance records app identity, safe title, URL only when available and enabled, and timestamp; reorder context items; choose an output profile (plain text, Markdown list, numbered list, or prompt block) with exact preview; copy.
  - **Climax:** bundle copies in known order with provenance choices preserved.
  - **Resolution:** export/import can round-trip order, states, timestamps, per-item content language, and provenance choices.
  - **Edge case:** long text or an excluded app — capture request is rejected with reason; user can disable provenance per app. Realizes CAP-008, CAP-009, QUE-001, QUE-004, DAT-003.

- **UJ-3. Keyboard-first user captures, copies, and advances without a pointer.**
  - **Persona + context:** Keyboard-first power user who needs near-zero-friction triggers and programmable output.
  - **Entry state:** selected text; shortcuts configured; panel summonable.
  - **Path:** global chord trigger; optional left/right modifier double-tap if enabled; summon/hide panel; copy-and-advance via output profile; optionally return focus to prior target.
  - **Climax:** next queued item is ready; completion remains explicit by default. P0 never synthesizes paste.
  - **Resolution:** shortcut editor has already blocked internal duplicates, reserved/system conflicts when knowable, invalid modifier-only patterns, and layout ambiguity.
  - **Edge case:** shortcut collision or layout mismatch — SET-002 refuses save; CAP-003 menu-bar and composer remain. Realizes CAP-001, CAP-002, CAP-003, QUE-005, SET-002, WIN-001, A11Y-001.

- **UJ-4. Assistive-technology user completes the core workflow without a timing-dependent gesture.**
  - **Persona + context:** Assistive-technology user who needs a semantic panel, a non-timing route, and clear feedback.
  - **Entry state:** VoiceOver, Full Keyboard Access, Voice Control, or Switch Control active; global hook may be unavailable.
  - **Path:** menu-bar command or manual composer (CAP-003) instead of double-tap; keyboard-only capture and queue operations; reorder via button/menu, never required drag; errors name problem and recovery; capture results are visible and announced.
  - **Climax:** full capture-to-copy workflow without workaround widgets.
  - **Resolution:** focus is not trapped in the panel; Settings is a semantic resizable window.
  - **Edge case:** permission not healthy — SET-003/SET-004 explain why, deep-link/instructions, retest, and alternative workflow. Realizes CAP-003, A11Y-001 through A11Y-006, WIN-005, SET-003, SET-004.

- **UJ-5. Privacy-sensitive user captures only what they asked for and can prove it stayed local.**
  - **Persona + context:** Privacy-sensitive user who needs explicit capture, exclusions, and local backup — not hidden network traffic, clipboard corruption, or secret capture.
  - **Entry state:** local-only production-default build; exclusions configured; Secure Input contexts exist.
  - **Path:** deliberate trigger only; secure/password fields never captured; excluded apps skipped; diagnostics expose stages without storing captured content; backup/restore and deletion are local, explicit, and scoped.
  - **Climax:** About/diagnostics and a previewed support bundle show version, identity, data path, schema version, permission health, and redacted failures — no content.
  - **Resolution:** user can export, restore, or delete; zero runtime network in the production-default build.
  - **Edge case:** WebView must not reach arbitrary files or shell. Realizes CAP-006, CAP-009, CAP-010, DAT-002, DAT-004, SEC-001 through SEC-006, SUP-002.

## 3. Glossary

Downstream workflows and readers must use these terms exactly.

- **Bronze** — working product name; local macOS selection-to-action queue
- **capture** — deliberate acquisition of currently selected text; never screenshot, screen recording, or ambient keystroke logging
- **trigger** — user action requesting capture/show/new note, such as chord, modifier double tap, menu command
- **capture request** — queued, ID-bearing attempt with provider stages and terminal result
- **selected text** — source app’s current text selection exposed by AX or explicit clipboard path
- **queue** — ordered transient work items intended for later copy/use/completion
- **section** — named ordered queue container and active capture destination
- **context item** — captured source text useful later
- **prompt item** — user-authored instruction intended for later destination
- **note item** — user-authored temporary text without implied task/AI semantics
- **lifecycle** — queued, copied, active, done, skipped, trashed state model
- **output profile** — deterministic format and post-copy behavior for one/multiple items
- **provenance** — optional safe source app/title/URL/timestamp metadata
- **AX** — macOS Accessibility API used to query focused element and selection
- **Input Monitoring** — macOS privacy capability relevant to passive global event monitoring
- **Accessibility permission** — macOS trust needed for selected-text and synthetic input paths
- **Secure Input** — macOS mechanism suppressing keyboard observation in protected contexts; Bronze never bypasses it
- **manual clipboard capture** — user explicitly copies selection, then asks Bronze to create from clipboard
- **synthetic fallback** — experimental Bronze-generated Cmd-C path; bounded and off by default unless proven safe
- **local-only** — no runtime content/network service, account, telemetry, or hosted AI; data on user Mac
- **conformance evidence** — scoped criterion/build/platform/test record, not blanket “proof”

## 4. Features

### 4.1 Capture and input

**Description:** Deliberate capture of selected text from supported macOS apps. Every capture request is queued, identified, processed once, and terminated. AX is primary; synthetic fallback is bounded; manual clipboard capture and menu-bar/composer paths exist so no timing-dependent gesture is required. Realizes UJ-1, UJ-3, UJ-4, UJ-5.

**Functional Requirements:**

#### CAP-001: Capture selected text with a global chord

User can capture selected text from supported macOS apps using a configurable standard global chord. Realizes UJ-1, UJ-3.

**Consequences (testable):**
- A configured standard global chord issues a capture request.
- Supported-source captures in the release matrix meet G-01 when the provider path is matrix-tested.

#### CAP-002: Optional modifier double-tap trigger

User can optionally enable left/right modifier double-tap trigger with adjustable timing and side/action mapping. Realizes UJ-3.

**Consequences (testable):**
- Double-tap is optional, configurable, and never the only path (CAP-003, A11Y-001).

#### CAP-003: Menu-bar and manual composer alternatives

Menu-bar command and manual composer provide non-global-hook alternatives. Realizes UJ-1, UJ-3, UJ-4.

**Consequences (testable):**
- Capture, show, and new note remain possible when Input Monitoring or global hooks are unavailable.

#### CAP-004: Every capture request is identified and terminal

Every capture request is queued, assigned ID, processed once, and ends as saved/rejected/failed/cancelled with reason. Realizes UJ-1, UJ-5.

**Consequences (testable):**
- Terminal results are exactly `saved | rejected | failed | cancelled`.
- Zero silently dropped capture request (G-01).
- P0 commits a supported result before success feedback; quick editor is post-save revision. `[ASSUMPTION: pre-commit confirmation is not a P0 setting — docs/21 R-01.]`

#### CAP-005: AX primary; bounded fallback

AX selected-text retrieval is primary; synthetic fallback is bounded; manual clipboard capture remains an explicit path. Realizes UJ-1.

**Consequences (testable):**
- Provider order is AX first.
- synthetic fallback is experimental, bounded, and off by default unless proven safe.
- P0 never attempts automatic clipboard restoration.

#### CAP-006: Secure fields never captured

Secure/password fields are never captured; user receives a private, non-content error. Realizes UJ-1, UJ-5.

**Consequences (testable):**
- Secure Input and password fields yield rejected/failed without content in diagnostics.
- Bronze never bypasses Secure Input.

#### CAP-007: Exact Unicode and intentional whitespace

Captured text preserves exact Unicode and intentional leading/trailing whitespace by default. Realizes UJ-1.

**Consequences (testable):**
- Round-trip of leading/trailing whitespace and Unicode equals source selected text.

#### CAP-008: Optional provenance

Optional provenance records app bundle ID/name, safe window/document title, URL only when available and enabled, and timestamp. Realizes UJ-2.

**Consequences (testable):**
- URL is recorded only when available and enabled.
- Ingress does not retarget after the capture request snapshot. `[ASSUMPTION: snapshot fields follow docs/21 R-02.]`

#### CAP-009: Exclusions and per-app provenance

User can exclude applications and disable provenance per app. Realizes UJ-2, UJ-5.

**Consequences (testable):**
- Excluded apps do not yield saved content.
- Per-app provenance disable is honored.

#### CAP-010: Content-free capture diagnostics

Capture diagnostics expose permission, trigger, selection provider, fallback, persistence, and feedback stages without storing captured content. Realizes UJ-5.

**Consequences (testable):**
- Diagnostic records are content-free by construction (SEC-006).
- Persisted diagnostic columns match the store contract in `docs/06-system-architecture.md` and `docs/08-data-model-and-portability.md`.

### 4.2 Queue

**Description:** Ordered sections of context item, prompt item, and note item records with lifecycle, output profile, copy-and-advance, undo, local search, and recoverable empty/error states. Realizes UJ-1, UJ-2, UJ-3.

**Functional Requirements:**

#### QUE-001: Sections

User can create, rename, reorder, collapse, archive, and choose active section. Realizes UJ-2.

**Consequences (testable):**
- Active section is the capture destination.
- Section state is `active | archived | trashed`.

#### QUE-002: Item operations

User can add, edit, duplicate, reorder, move, select, and delete text items (context item, prompt item, note item). Realizes UJ-1, UJ-2.

**Consequences (testable):**
- `content_language` is canonical BCP 47 or `und`, default `und`, no silent detection, preserved across revision/export/import.

#### QUE-003: Lifecycle states

Item states are `queued`, `copied`, `active`, `done`, `skipped`, `trashed`; automatic transitions are configurable and undoable. Realizes UJ-1, UJ-3.

**Consequences (testable):**
- lifecycle values are exactly that set.
- Automatic transitions are undoable (QUE-006).

#### QUE-004: Output profiles

User can create, duplicate, edit, reset, and select named output profile records for plain text, Markdown list, numbered list, or prompt block, with exact preview and bounded format options. Realizes UJ-1, UJ-2, UJ-3.

**Consequences (testable):**
- output profile is the sole `postCopyAction` / `advancePolicy` authority.
- Built-in default is `copied` + `keep`.
- post-copy action ∈ `unchanged | copied | active | done`; advance policy ∈ `keep | nextQueued`.

#### QUE-005: Copy-and-advance; no synthesized paste

User can copy-and-advance and optionally return focus to prior target, but completion remains explicit by default. P0 never synthesizes paste. Realizes UJ-1, UJ-3.

**Consequences (testable):**
- `returnToPriorApp` is focus-only, never provenance, never synthetic paste.

#### QUE-006: Undo and trash

Undo restores last destructive or lifecycle action; trash retention protects accidental deletion. Realizes UJ-1, UJ-5.

**Consequences (testable):**
- Destructive and lifecycle actions undo.
- Trash retention matches DAT-004.

#### QUE-007: Local search

Search covers item content, sections, and safe provenance; search is local. Realizes UJ-2.

**Consequences (testable):**
- Search does not use a network service.
- `[NOTE FOR PM]: QUE-007 / G-06 incomplete until SEARCH-01 golden corpus exists (ADR-018 Proposed).`

#### QUE-008: Recoverable empty and failure UI

Empty, loading, capture failure, storage read-only/recovery, and migration states provide recoverable UI. Realizes UJ-4, UJ-5.

**Consequences (testable):**
- Each named state has a recoverable UI path; none silently empty the queue.

### 4.3 Panel and app shell

**Description:** Summonable panel on the current display/edge, respecting macOS work area, with configurable presence and a semantic Settings window that never traps focus. Realizes UJ-3, UJ-4.

**Functional Requirements:**

#### WIN-001: Summon and hide

User can summon/hide panel on current pointer or active-app display and chosen screen edge. Realizes UJ-3.

**Consequences (testable):**
- Persisted panel edge is physical `left | right | top` (or `lastPosition`); RTL never mirrors stored side.

#### WIN-002: Work area and displays

Panel respects work area, menu bar, Dock, notch, Spaces, full-screen apps, multiple displays, and scale factors. Realizes UJ-3.

**Consequences (testable):**
- Panel remains usable across those environments in the supported macOS range (ADR-002 remains Proposed; no marketing range).

#### WIN-003: Blur, pin, and focus restore

Panel can remain visible or hide on blur per setting and restores focus predictably. Realizes UJ-3, UJ-4.

**Consequences (testable):**
- Panel mode ∈ `summon | pinned | autoHide`.
- Focus restoration is deterministic after hide/show.

#### WIN-004: Shell chrome configuration

Menu-bar presence, Dock icon, launch at login, always-on-top, translucency, density, width, and edge are configurable where platform permits. Realizes UJ-3.

**Consequences (testable):**
- Unavailable platform options are not presented as if they work.

#### WIN-005: Settings window; panel does not trap focus

Settings is a semantic, resizable window; panel never traps focus. Realizes UJ-4.

**Consequences (testable):**
- Quick panel: active-section lifecycle. Library: paginated history, archive/trash, import/export, backup/restore. Settings: settings/shortcuts/permissions/diagnostics preview. Onboarding: manual note + permission preflight; no library/import/diagnostics authority.

### 4.4 Data

**Description:** Local SQLite, versioned migrations, automatic backups, deterministic export/import, scoped deletion. Realizes UJ-2, UJ-5.

**Functional Requirements:**

#### DAT-001: Local SQLite with migrations

Store uses local SQLite with transactional versioned migrations and integrity checks. Realizes UJ-5.

**Consequences (testable):**
- Migrations are transactional and versioned; integrity checks run.
- Duplicate command IDs reconstruct prior results; expired IDs never re-execute; receipts are content-free.

#### DAT-002: Automatic local backups and verified restore

App creates automatic local backups and supports verified restore. Realizes UJ-5.

**Consequences (testable):**
- `backupSchedule` is `daily | weekly` only; Back Up Now remains; pre-migration/pre-restore backups are mandatory; no manual-only mode.

#### DAT-003: Deterministic export/import

Deterministic JSON archive and Markdown+assets export/import preserve order, states, timestamps, per-item content language, and provenance choices. Realizes UJ-2.

**Consequences (testable):**
- Round trips preserve those fields.
- Queue export contains item bodies and enabled provenance; preview warns before write.

#### DAT-004: Explicit local deletion

Data deletion is local, explicit, scoped, and recoverable within configured trash period. Realizes UJ-5.

**Consequences (testable):**
- Deletion is not remote, not implicit, and is recoverable within trash retention.

### 4.5 Security and privacy

**Description:** local-only production default; least-privilege native boundary; no remote UI code; signed distribution; redacted diagnostics. Realizes UJ-5.

**Functional Requirements:**

#### SEC-001: No remote UI code; strict CSP

Production UI loads no remote code, fonts, scripts, or content; strict CSP applies. Realizes UJ-5.

**Consequences (testable):**
- Production-default packet capture shows no UI origin fetches.

#### SEC-002: Least-privilege capabilities; typed IPC

Tauri capabilities are least-privilege per window; IPC inputs/outputs are typed and validated in Rust. Realizes UJ-5.

**Consequences (testable):**
- Each window has a minimal capability set; invalid IPC is rejected in Rust.

#### SEC-003: WebView cannot reach arbitrary native powers

WebView cannot request arbitrary file reads, shell execution, recursive quarantine changes, or unrestricted paths. Realizes UJ-5.

**Consequences (testable):**
- Those requests fail closed.

#### SEC-004: No account, telemetry, or runtime network by default

App has no account, telemetry, analytics, hosted AI, or runtime network by default. Realizes UJ-5. local-only.

**Consequences (testable):**
- Zero runtime network requests in the production-default build (G-05), including no automatic update check.

#### SEC-005: Signed, notarized, reproducible-as-practical releases

Releases are reproducible as practical, signed/notarized, checksummed, SBOM-backed, dependency-audited, and preserve TCC identity. Realizes UJ-5.

**Consequences (testable):**
- Signed upgrade tests exist; TCC identity continuity is verified when macOS does not reset trust.

#### SEC-006: Redacted diagnostics; previewed support bundle

Diagnostics redact content and secrets by construction; support bundle is previewed before export. Realizes UJ-5.

**Consequences (testable):**
- Settings/support bundles exclude credentials, tokens, diagnostics secrets, and machine paths except where About explicitly shows the data path to the user.
- User previews the support bundle before export.

### 4.6 Accessibility

**Description:** Entire core workflow is keyboard-reachable, semantically exposed, resizable, and release-tested with platform assistive technologies. Realizes UJ-4. Claims require conformance evidence.

**Functional Requirements:**

#### A11Y-001: Keyboard without timing-dependent gesture

Entire core workflow works with keyboard without timing-dependent gesture. Realizes UJ-3, UJ-4.

**Consequences (testable):**
- CAP-003 paths complete capture, queue, copy, and settings without double-tap.

#### A11Y-002: Programmatic semantics

Semantic roles, names, descriptions, values, states, relationships, live status, focus order, and focus restoration are programmatically exposed. Realizes UJ-4.

**Consequences (testable):**
- Assistive APIs expose those properties on core controls.

#### A11Y-003: Resize, zoom, contrast, motion, color

UI supports 200% text resize and 400% zoom/reflow at 320 CSS px without lost content or functionality, plus Increase Contrast, Differentiate Without Color, Reduce Motion, Reduce Transparency, dark/light, and no color-only information. Realizes UJ-4.

**Consequences (testable):**
- Both 200% text resize and 400%/320 reflow are required.
- Native bridge reads Reduce Motion, Reduce Transparency, Increase Contrast, and Differentiate Without Color; app overrides may only strengthen.

#### A11Y-004: Reorder without drag

Reorder has button/menu alternatives; drag is never required. Realizes UJ-4.

**Consequences (testable):**
- Move up/down (or equivalent) works with keyboard and AT.

#### A11Y-005: Platform AT journeys release-tested

VoiceOver, Full Keyboard Access, Voice Control, and Switch Control core journeys are release-tested. Realizes UJ-4.

**Consequences (testable):**
- Release evidence includes those journeys. Human AT evidence uses `blocked-human-validation` until collected. Public AA/conformance claims stay blocked until every A/AA row plus independent audit exist.

#### A11Y-006: Errors and capture feedback

Errors identify problem and recovery; capture results use visible plus announced feedback. Realizes UJ-4, UJ-1.

**Consequences (testable):**
- Every capture request terminal result has visible and announced feedback.

### 4.7 Internationalization

**Description:** Catalogued UI strings, BCP 47 locales, RTL and IME, semantic shortcuts. Realizes UJ-4. Validates G-06.

**Functional Requirements:**

#### I18N-001: Localized catalog; no concatenated sentences

All UI strings use localized message catalog with plural/select formatting; no concatenated sentences. Realizes UJ-4.

**Consequences (testable):**
- No hard-coded UI sentences; pseudo-locale suite fails if concatenation appears.

#### I18N-002: BCP 47 locale selection and fallback

Locale uses canonical BCP 47, system default, per-app override, and script-preserving progressive fallback; dates/numbers/lists use `Intl`. Realizes UJ-4.

**Consequences (testable):**
- RFC 4647 progressive lookup preserves script/variant before language, then `en`.

#### I18N-003: RTL, IME, graphemes, content language

Layout supports RTL, long expansion, CJK/Indic input, emoji, grapheme-safe editing, and Unicode normalization policy; each item preserves user-overridable BCP 47 or `und` content-language metadata. Realizes UJ-2, UJ-4.

**Consequences (testable):**
- Stored panel edge is not mirrored by RTL (WIN-001).
- content language is never silently detected.

#### I18N-004: Localized shortcut display

Shortcut display uses localized modifier/key names while storage preserves semantic intent and physical-key details where needed. Realizes UJ-3.

**Consequences (testable):**
- Canonical store is `shortcuts` keyed by action ID; display is localized; storage is semantic.

### 4.8 Settings and support

**Description:** Searchable settings, shortcut validation, permission health, help, and About/diagnostics. Realizes UJ-3, UJ-4, UJ-5.

**Functional Requirements:**

#### SET-001: Searchable, resettable, exportable settings

Settings are searchable, grouped, keyboard accessible, resettable per field/group/all, and exportable without app credentials, permission tokens, diagnostics, or machine paths; preview identifies user-entered literals and app policies that may themselves be sensitive. Realizes UJ-5.

**Consequences (testable):**
- Settings export excludes those secret classes.
- Preview flags user-entered literals and sensitive policies.

#### SET-002: Shortcut editor conflict detection

Shortcut editor detects internal duplicates, reserved/system conflicts when knowable, invalid modifier-only patterns, and layout ambiguity before save. Realizes UJ-3.

**Consequences (testable):**
- Save is refused for those classes of conflict.

#### SET-003: Permission health

Permission health distinguishes Input Monitoring, Accessibility permission, Automation if introduced, and capture-pipeline self-test. Realizes UJ-4, UJ-5.

**Consequences (testable):**
- Permission enum is `unknown | not_requested | denied | granted_unverified | healthy | degraded | unavailable | requires_relaunch`.
- Granted ≠ healthy.

#### SET-004: Permission explanation and alternatives

App explains why each permission is needed and offers deep-link/instructions, retest, and alternative workflow. Realizes UJ-4, UJ-5.

**Consequences (testable):**
- Each required permission has why, how, retest, and a CAP-003 alternative.

#### SUP-001: Built-in help

Built-in help documents supported sources, limitations, privacy, backup/restore, shortcuts, and accessibility. Realizes UJ-4, UJ-5.

**Consequences (testable):**
- Help covers those topics in the locale catalog (I18N-001).

#### SUP-002: About and diagnostics

About/diagnostics shows version, bundle identity, data path, schema version, permission health, and redacted recent failures. Realizes UJ-5.

**Consequences (testable):**
- Failures are redacted; content is not shown.

## 5. Non-Goals (Explicit)

From `docs/03-prd.md` §9:

- Passive unlimited clipboard history.
- Screen recording, keylogging, or complete input capture.
- Full task management, calendar, reminders, collaboration, or project planning.
- Full note vault/PKM graph.
- Hosted AI assistant or mandatory AI service.
- Mandatory account/cloud sync.
- Pixel clone of Copper or fork of Cooper.
- Mac App Store distribution for v1.

## 6. MVP Scope

### 6.1 In Scope

P0: text capture, manual entry, ordered sections, edit/reorder/copy/complete/undo, search, configurable triggers, permission diagnostics, local persistence, backup/export, accessibility, i18n architecture, and signed distribution. All IDs in §4 plus G-01 through G-07.

v1 ships only when all P0 requirements have implemented tests or documented manual evidence, no open severity-0/1 defects, capture matrix meets target, signed upgrade preserves permissions, backup/restore drill succeeds, accessibility conformance matrix is reviewed, and network-capture test shows zero unexpected traffic.

### 6.2 Out of Scope for MVP

P1 (deferred, not v1):

- Context bundles and nested prompt blocks without arbitrary deep task hierarchy.
- Safe richer provenance and source reopen when supported.
- Merge/split items; reusable output templates.
- Services, Shortcuts/App Intents, URL scheme, CLI, PopClip, Raycast, Alfred integrations.
- Importers for Cooper and common Markdown/JSON structures.
- Local retention rules, archive analytics visible only to user, and secret-pattern warnings.
- Theme editor within validated contrast bounds.

P2 (deferred further):

- Images/files and on-device OCR.
- Reusable prompt-template library and variables.
- Optional end-to-end encrypted sync after separate threat model.
- Sandboxed extension/action system.
- Bring-your-own local model or explicit cloud actions.
- Windows/Linux after macOS quality targets sustain across two stable releases.

## 7. Success Metrics

Targets are quality gates, not telemetry requirements. Measure on test fixtures and opt-in local diagnostics; do not collect user behavior remotely.

**Primary**

- **G-01**: Reliable capture — ≥99.9% successful supported-source captures in release matrix; zero silently dropped capture request. Validates CAP-004, CAP-010.
- **G-02**: Fast feedback — local trigger acknowledgment ≤100 ms p95; AX capture-to-saved ≤300 ms p95; fallback ≤2 s p95. Validates CAP-001, CAP-005.
- **G-03**: Recoverable data — zero known destructive migrations; backup/restore and export/import round trips pass corpus. Validates DAT-001, DAT-002, DAT-003.
- **G-04**: Accessible core — P0 journeys pass keyboard, VoiceOver, Full Keyboard Access, Voice Control, and Switch Control; every WCAG 2.2 A/AA criterion accounted for; relevant EN 301 549 clauses 5, 11, and 12 matrix complete. Validates A11Y-001 through A11Y-006. Reported as conformance evidence, not blanket proof.
- **G-05**: Local privacy — zero runtime network requests in production-default build; no telemetry/account; exclusion and secure-field tests pass. Validates SEC-004, CAP-006, CAP-009. local-only.
- **G-07**: Low interruption — median capture user action ≤1 second after selection in usability study. Validates CAP-001, CAP-003.

**Secondary**

- **G-06**: Global readiness — no hard-coded UI strings; pseudo-locales, RTL, CJK IME, and long-string suite pass. Validates I18N-001 through I18N-004, QUE-007.

**Counter-metrics (do not optimize)**

- **SM-C1**: Remote user-behavior telemetry or always-on network checks to “prove” G-01 through G-07. Counterbalances G-05. Measurement stays on fixtures and opt-in local diagnostics.
- **SM-C2**: Capture volume / ambient clipboard coverage. Counterbalances G-01 and the deliberate-over-ambient principle. Bronze is not clipboard history.

## 8. Open Questions

1. ADR-002 Proposed — production macOS minimum and universal2 claim remain a decision gate; development may be arm64; no marketing range.
2. ADR-009 Proposed — storage locator is an interface; development may use Application Support.
3. ADR-018 Proposed — QUE-007 / G-06 incomplete until SEARCH-01 golden corpus exists.
4. Exact default global chords (`docs/05-ux-ui-interaction-spec.md` proposals) are compatibility-spike gated, not product defaults yet.

## 9. Assumptions Index

- `[ASSUMPTION]` §4.1 CAP-004 — pre-commit confirmation is not a P0 setting (docs/21 R-01).
- `[ASSUMPTION]` §4.1 CAP-008 — ingress snapshot/no-retarget follows docs/21 R-02.
- v1 targets a supported current macOS range chosen during Spike 0; minimum cannot be selected solely from a competitor claim.
- Distribution outside Mac App Store through Developer ID signed/notarized artifact.
- Input Monitoring likely needed for listen-only event tap; Accessibility permission needed for AX/synthetic input paths. Validate on every supported macOS version.
- Some apps do not expose selected text through AX or permit simulated copy. “Supported” means matrix-tested provider path with documented limitation.
- User content may be sensitive. No content logs, crash attachments, or support exports without explicit preview/consent.

## 10. Platform

v1 is macOS only. No Windows. No Linux. `[NON-GOAL for MVP]`

## 11. Constraints and Guardrails

- No network access by default, telemetry, account, remote fonts, CDN, hosted AI, or analytics. Any later network feature needs ADR, explicit opt-in, visible destination/payload class, and threat-model update.
- Never silently drop a capture request.
- Prefer AX; synthetic fallback is bounded, not primary.
- macOS capture code stays native and behind a narrow typed interface.
- No arbitrary filesystem paths from WebView.
- Every user-facing string belongs in the locale catalog.
- Independent implementation: Bronze v1 copies no Cooper source code. Do not claim a legal clean-room process.
