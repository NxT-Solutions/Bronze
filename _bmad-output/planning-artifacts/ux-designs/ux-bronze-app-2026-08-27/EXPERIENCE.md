---
name: Bronze
status: final
created: '2026-08-27'
updated: '2026-08-27'
sources:
  - docs/03-prd.md
  - docs/05-ux-ui-interaction-spec.md
  - docs/10-accessibility-conformance-plan.md
  - docs/11-i18n-localization.md
  - docs/12-settings-and-shortcuts.md
  - docs/18-adrs.md
---

# Bronze — Experience Spine

Visual identity lives in `DESIGN.md`. This spine owns IA, behavior, states, interactions, accessibility, and journeys. Spines win on conflict with mocks or the concept PNG.

## Foundation

macOS-only desktop utility. Tauri 2 native shell + WKWebView. UI is owned shadcn components on the React Aria base (ADR-012), TypeScript, Tailwind tokens from `DESIGN.md`. Semantic native HTML where simpler. Library choice is not conformance proof.

Surfaces: native status item/menu; activating utility quick panel; activating utility quick editor; resizable Library window; resizable Settings window; progressive onboarding. App defaults to accessory activation policy; Dock icon is opt-in (WIN-004). No account, telemetry, remote fonts, CDN, hosted AI, or runtime network by default (SEC-004).

`DESIGN.md` is the visual identity reference. Unlisted shadcn visuals inherit until they fail Bronze contrast, focus, or motion rules.

## Information Architecture

| Surface | Reached from | Purpose |
|---|---|---|
| Status menu | Always-visible status item | Capture Selection, New Note, Show Bronze, recent capture result, Permission Health, Settings, Help, Quit — available even when global monitoring is unavailable (CAP-003, WIN-004, SET-003) |
| Quick panel | Status menu Show Bronze / `app.togglePanel` | Active-queue lifecycle: section chooser, search, health indicator, item cards, composer, copy/profile, footer status (QUE-001, QUE-002, WIN-001) |
| Library | Status menu / Settings Data / panel overflow | P0 resizable history, section management, archive/trash, search, import/export, backup, recovery (QUE-001, DAT-002, DAT-003) |
| Settings | Status menu / `window.settings` | Sidebar: General, Capture, Shortcuts, Panel, Copy, Privacy, Data, Accessibility, Language, Advanced, About. Search indexes localized labels, descriptions, synonyms (SET-001, WIN-005) |
| Quick editor | Post-save Edit on an item | Ordinary activating utility window. Save writes a revision; Cancel keeps the already captured item (CAP-004, QUE-002) |
| Onboarding | First launch | Permission-late: explain local queue, create a manual note, then optional capture route and TCC (CAP-003, SET-004) |
| Help | Status menu Help | Accessible HTML/Markdown: sources, limitations, privacy, backup, shortcuts, accessibility (SUP-001) |

Quick panel owns active-queue reorder/lifecycle. Library owns broad data-management. Settings owns preferences, shortcuts, permission health, diagnostics preview. Onboarding has no library/import/diagnostics authority.

Modal stacks one level. Panel never traps focus (WIN-005). Escape hides/closes and restores prior focus where the platform permits.

## Voice and Tone

Microcopy. Brand posture lives in `DESIGN.md`. Catalog strings only; no sentence concatenation (I18N-001). “Capture” never means screenshot, recording, or keylogging.

| Do | Don't |
|---|---|
| “Captured to Research.” | “Copied from your screen.” / surveillance verbs |
| “No selectable text found. Copy manually or open a note.” | Silent no-op |
| “Protected field not captured.” | Source app name or content in the error |
| “Accessibility permission required.” plus Open Settings and Use Manual Capture | “Permission denied.” with no alternative |
| “Queued.” / “Copied.” / “Done.” with icon + name | Color-only status |
| “Not specified” for unknown content language | Guessed locale on captured text |
| Spoken shortcut: “Option plus Shift plus Space.” | Glyph-only instruction |
| Stable diagnostic ID + Retry | Raw stack traces in the panel |

VoiceOver intent examples (localized wording belongs in catalogs): “Research, section, 4 queued items, expanded.” “Item: Compare configuration formats. Queued. Position 2 of 4.” “Complete Compare configuration formats, checkbox, unchecked.” Avoid repeating full item bodies on every status update.

## Component Patterns

Behavioral. Visual specs live in `DESIGN.md.Components`.

| Component | Use | Behavioral rules |
|---|---|---|
| Status item / menu | Global | Localized name, role, enabled/state/action. Menu remains usable without the panel. Recent result is persistent, not toast-only (A11Y-006, SET-003). |
| Item card | Quick panel, Library | Not a clickable `div`. List/article + real buttons/checkbox/menu. Anatomy: lifecycle control with item-specific name; kind icon + visible label when kind matters; wrapping preview; optional `contentLanguage` (default `und`); optional provenance; status text; context actions; drag handle only if pointer reorder enabled, with Move Up/Down equivalents (QUE-002, A11Y-002, A11Y-004). Ordinary tab order for MVP. |
| Section chooser | Panel header | Active section receives capture/manual adds (QUE-001). Collapse/expand announces count. |
| Queue toolbar | Panel | Result count, selected count, output profile, Copy. At 320 CSS px wraps into labeled overflow; Copy and lifecycle remain available (QUE-004, QUE-005, A11Y-003). |
| Composer | Panel footer-adjacent | Multiline textarea. Explicit Add. `⌘Enter` never fires during IME composition (I18N-003). Do not trim/normalize on blur. |
| Output profile picker | Toolbar, Copy settings | Named profiles: plain, bullets, numbered, prompt block. Preview uses fixed hostile sample in Settings; current-item preview is an explicit panel action. Profile owns `postCopyAction` and `advancePolicy` (QUE-004, QUE-005). |
| Permission health | Menu, panel indicator, Settings | Closed enum: `unknown`, `not_requested`, `denied`, `granted_unverified`, `healthy`, `degraded`, `unavailable`, `requires_relaunch`. Each row: why, last test, Retest, Open System Settings, alternative. Screen Recording: “Not used.” (SET-003, SET-004). |
| Shortcut recorder | Settings Shortcuts | Record on key release; Escape cancel; Delete clear; spoken modifiers; typed chooser for AT. Save only after native registration succeeds; failure keeps old binding (SET-002, I18N-004). |
| Dialog | Confirmations, recorder | Name, description, initial focus, modal/inert background, Escape, restore invoker (A11Y-002). |
| Toast / live status | Panel, native | `role=status` polite for routine results; alert only for blocking errors. Critical recovery stays in persistent status center. Hidden WKWebView live regions are not trusted (A11Y-006). |
| Empty state | Panel, Library, search | Recoverable, with one primary alternative (manual note, clear search, grant permission) (QUE-008). |

Tooltip never sole label. Accessible name contains exact visible text (WCAG 2.5.3). Icon-only controls have localized names.

## State Patterns

| State | Surface | Treatment |
|---|---|---|
| Idle / local-only | Panel footer | Quiet “local-only” plus contextual shortcut hints. No ambient capture animation. |
| Capture trigger acknowledged | Source app + status | Optional brief haptic/sound/visual, each independently configurable. ≤100 ms local ack target. Does not steal source focus (CAP-004, WIN-003). |
| Capture success | Status menu + native announcement | Persistent recent result. Native `NSAccessibility` `.announcementRequested` (“Captured to {section}”) while Bronze is inactive. WebView live region only if Bronze window is active. |
| No selection | Status + panel | Visible message and manual/note recovery. Never silent drop. |
| Secure field | Status + panel | “Protected field not captured.” No source or content details (CAP-006). |
| Permission needed | Menu, panel, Settings | Reason + Open Settings + Use Manual Capture. Denial leaves the app usable (SET-004). |
| Busy / queued request | Status | Position or progress; every request has ID and terminal result (CAP-004). |
| Failure | Status + diagnostics | Stable diagnostic ID, Retry; raw details stay redacted (CAP-010, SEC-006). |
| Empty queue | Panel | Manual composer remains the primary path. |
| Search empty | Panel / Library | Announced count, clear action, no chatter (QUE-007). |
| IME composing | Composer / editor | Enter/shortcuts do not submit; composition preserved (I18N-003). |
| Storage read-only / recovery | Library / panel | Recoverable UI; no destructive surprise (QUE-008, DAT-002). |
| Reduce Transparency | All WebView | `{colors.background}` opaque + `{colors.border}`; no vibrancy (A11Y-003). |
| Differentiate Without Color | All | Shape/icon/label for every state; app override may only strengthen the OS setting (A11Y-003). |
| Locale switch | All | Reacquire focused control by stable semantic ID; `lang`/`dir`, native strings, menus, portals update; one completion announcement after stale-language surfaces are gone (I18N-002). |

## Interaction Primitives

**Keyboard.** Every operation has a non-timing path (A11Y-001). Resolver order: text editing → open modal/menu → focused component → app-local → global. Do not intercept VoiceOver, system, text-editing, or IME chords.

| Action | Default (proposals; spike-confirmed) | Notes |
|---|---|---|
| Summon/hide | `⌥Space` | `app.togglePanel`; avoid Spotlight conflict (WIN-001) |
| Capture selection | `⌥⇧Space` | Global; register/test before commit (CAP-001) |
| Add composer | `⌘Enter` | Never during IME |
| Copy selected/focused | `⌘C` | Native text selection wins in editors |
| Copy as profile | `⇧⌘C` | Configurable; skip if global conflict |
| Edit focused item | `Return` | Focused-row context |
| Complete | `Space` | Row focus only, never while editing |
| Search | `⌘F` | Moves focus; announces results (QUE-007) |
| Select all items | `⌘A` | Editor selection wins while editing |
| Move item | Move menu + optional chord | Chord not required for conformance (A11Y-004) |
| Close/hide | `Escape` | Preserves draft; restores prior focus |

Modifier double-tap is optional, off by default, adjustable to ≥500 ms, and never the sole route (CAP-002, EN 301 549 5.8/5.9).

**Focus.** Opening panel by shortcut returns last meaningful control, or composer on New Note. Capture-only success does not steal source focus. Quick editor: title/textarea documented initial focus; VoiceOver hears window title and status. Removing a focused item: next, previous, then section heading/composer. Reorder keeps focus on the moved item and announces index. Error summary takes focus only for blocking submit errors.

**Pointer.** Real buttons. Drag reorder optional and never required. No action only from hover, right-click, double-tap, color, or toast.

**Banned:** silent dropped capture; `contenteditable` in P0; virtualized core list in P0; synthesized paste; WebView-arbitrary filesystem paths; clickable non-semantic rows; timing-only capture.

## Accessibility Floor

Behavioral. Contrast and focus paint live in `DESIGN.md`. Do not claim “WCAG proof” or “A11y proof.”

- WebView target: WCAG 2.2 AA on a specific build/platform/settings/content. Whole app: EN 301 549 V3.2.1 clauses 5, 11, 12 plus Apple VoiceOver evaluation. Public AA/conformance claims stay blocked until every A/AA row and independent audit exist (ADR-013, A11Y-001–A11Y-006).
- Keyboard-only, VoiceOver, Full Keyboard Access, Voice Control, Switch Control on core journeys (A11Y-005).
- Visible focus (2.4.7); not entirely hidden (2.4.11); non-text contrast (1.4.11). Enhanced ≥2 CSS px / ≥3:1 focus appearance is a Bronze target, not part of the AA claim (2.4.13 is AAA).
- 200% text resize and 400% zoom/reflow at 320 CSS px without lost content or functionality (A11Y-003).
- Native bridge publishes Reduce Motion, Reduce Transparency, Increase Contrast, and Differentiate Without Color; overrides only strengthen (WIN-004).
- Capture-only announcements use AppKit while the WebView is inactive; do not double-speak with a live region.
- Help and support are accessible, not image-only (EN 301 549 clause 12, SUP-001).

## Key Flows

Protagonists are the PRD personas, named for journey narration.

### Flow 1 — Capture without leaving the editor (Alex, AI-assisted developer)

1. Alex selects a compiler error in the editor and invokes Capture Selection (`capture.selection` or status menu).
2. Bronze snapshots the request, assigns an ID, and reads AX selected text. Source focus stays in the editor.
3. Status menu shows success; macOS announces “Captured to Research.” No Bronze window steals key focus.
4. Later Alex summons the panel, reviews the queued item (whitespace preserved), chooses Numbered list, and copies.
5. **Climax:** The item becomes `copied` (built-in profile), the formatted block is on the pasteboard, and Alex pastes into the chat themselves. Completion stays explicit; Bronze does not synthesize paste (QUE-005, CAP-007).

Failure: no selection → “No selectable text found. Copy manually or open a note.” Secure Input → “Protected field not captured.” Permission missing → reason + Manual Capture. Busy → queued, never dropped (CAP-004, CAP-006, A11Y-006).

### Flow 2 — Ordered excerpts (Jordan, researcher/writer)

1. Jordan captures three excerpts from a browser and a local document with provenance opt-in at app identity only.
2. Panel lists them in capture order under the active section. Jordan moves one via Move Down (no drag required).
3. Jordan opens Library to search, archive the spent section, and export a Markdown bundle.
4. **Climax:** Export preview shows order, states, timestamps, per-item `contentLanguage` (`und` unless Jordan set it), and enabled provenance. Jordan confirms a local write. UI locale does not rewrite item bodies (DAT-003, I18N-003, QUE-007).

Failure: provenance denied for that bundle ID → item still captures; source row omitted. Export warns that item bodies are user content.

### Flow 3 — Keyboard-only queue (Riley, keyboard-first power user)

1. Riley records a conflict-tested summon chord in Settings. Native registration succeeds before save; test mode has Stop/Skip and no motor-timing requirement (SET-002).
2. Riley summons the panel on the frontmost-app display against the stored physical right edge. RTL UI does not flip that edge (WIN-001, I18N-003).
3. `⌘F` focuses search and announces counts. Riley tabs the card, `Space` completes one item, `⌘Z` undoes.
4. **Climax:** Riley copies with the prompt-block profile and returns to the prior app. Focus restoration is predictable; the panel hides per summon mode (WIN-003, QUE-005).

Failure: registration conflict → old binding kept, reason announced, menu/manual routes still work.

### Flow 4 — Assistive-technology capture (Sam, assistive-technology user)

1. Sam skips modifier double-tap. Onboarding creates a manual note first, proving value without TCC.
2. Sam enables VoiceOver, opens the status menu, and chooses Capture Selection after Accessibility is granted.
3. Sam selects non-sensitive text in TextEdit (not Bronze’s own WebView), then captures. Announcement is native, medium priority, localized.
4. In the panel, VoiceOver hears section, position, and “Complete {summary}, checkbox, unchecked.” Card actions are buttons; reorder is a menu.
5. **Climax:** Sam completes the workflow with keyboard and VoiceOver only. No timing gesture was required. Quick editor, if used, is an activating window with a titled focus target (A11Y-001, A11Y-005, WIN-005).

Failure: VoiceOver chord not swallowed by the recorder. Switch Control / Voice Control use visible names that match on-screen text.

### Flow 5 — Permission-late privacy (Priya, privacy-sensitive user)

1. Priya launches Bronze. Dock icon and launch-at-login stay off. No network, no account.
2. Onboarding explains the local queue with one static diagram. Priya adds a manual note.
3. Priya denies Input Monitoring. Menu Capture Selection and composer still work. Health shows `denied` with why, Retest, and alternative (SET-003, SET-004).
4. Priya excludes a password manager and leaves provenance at `none`.
5. **Climax:** Priya captures from a supported app via the menu, sees a content-free success status, and knows data stayed on this Mac. Factory-erase copy later lists what Bronze can and cannot remove (clipboard history is outside Bronze) (CAP-009, SEC-004, DAT-004).

Failure: settings export preview flags user-entered literals and app policies; credentials, tokens, diagnostics, and machine paths are absent (SET-001).

## Responsive & Platform

| Condition | Behavior |
|---|---|
| Default panel | One column; `{spacing.3}` card padding; composer + status reachable |
| 200% text resize | Layout wraps; no lost controls; OS text size maps to app scale presets (A11Y-003) |
| 400% zoom / 320 CSS px | One-column; toolbar → labeled overflow; no horizontal scroll for chrome; content vertical scroll; code may scroll inside a bounded region |
| RTL | Logical CSS for chrome; stored physical edge unchanged; user content `dir="auto"` with isolation (I18N-003) |
| Reduce Motion | No reorder animation; state still understandable |
| Reduce Transparency | Opaque `{colors.background}` + `{colors.border}` |
| Increase Contrast / Differentiate Without Color | Stronger text, boundaries, focus; icon+label on every status |

**Platform:** Quick panel and quick editor are activating utility windows (key-capable), not nonactivating panels, so keyboard and VoiceOver focus are reliable. Native `NSStatusItem` carries the menu. Position uses visible work area (menu bar, Dock, notch, Spaces, displays, scale). Persist physical `left|right|top` and width. Capture-only flows must not steal source focus. Hidden WKWebView is not an announcement channel.

## Inspiration & Anti-patterns

- **Kept from the Bronze UX spec:** deliberate queue, observable failure, permission-late onboarding, persistent status over toast-only recovery.
- **Rejected — Copper/Cooper trade dress:** no copied screenshots, icons, name, or pixel-identical chrome. Cooper is a failure corpus, not a visual source.
- **Rejected — ambient clipboard / surveillance chrome:** no always-on history reel, no keylogging aesthetic, no secret capture.
- **Rejected — gesture-only power-user myth:** double-tap is optional; menu, chord, and composer are first-class.
- **Rejected — WCAG-proof marketing:** evidence is scoped; this spine does not authorize a conformance badge.
