# UX, UI, and interaction specification

## 1. Experience promise

Bronze should feel like temporary working memory attached to current task: instant to summon, quiet when idle, explicit about state, forgiving when platform capture fails. It must not look or behave like surveillance software.

Warm bronze tokens from DESIGN.md are inspiration, not a pixel spec (A11Y-003). The running desktop chrome uses a zinc/neutral hand-test set (`#fafafa` surface, `#18181b` text, `#e4e4e7` hairline) so the product is not a brown/metal theme. Automated sampling requires normal text ≥4.5:1. Grouping hairlines follow HIG; Increase Contrast is the 3:1 non-text path. That sampling is not a public WCAG AA or AAA claim. Draft WebView rows live in `docs/evidence/WCAG-22-INVENTORY.md`. No Copper trade dress.

## 2. Information architecture

### Menu-bar menu

Always available, even when global monitoring is unavailable. Status-item left-click is Show. The status menu lists the latest five overview items (click copies), then Capture, Help, and Quit. New Note, Settings, Library, and permission health stay on the Bronze app menu and Settings window.

### Quick panel

- Header: active section chooser, search, permission/capture health indicator, overflow.
- Queue toolbar: result count, selected count, output profile, Copy.
- Section list: heading and ordered item cards.
- Composer: multiline note/prompt field and explicit Add.
- Footer/status: local-only state, contextual shortcut hints, diagnostic retry when needed.

### Library window

P0 resizable window for full history, section management, archive/trash, search, import/export, backup, and recovery. Quick panel retains active-queue lifecycle/reorder actions; Library owns broad data-management journeys.

### Settings window

Sidebar categories: General, Capture, Shortcuts, Panel, Copy, Privacy, Data, Accessibility, Language, Advanced, About. Search indexes localized labels, descriptions, and synonyms.

### Quick editor

Optional post-save edit surface after capture. P0 always commits a supported result before success feedback; opening editor creates ordinary revision on Save, and Cancel keeps already captured item unchanged. It is a normal activating utility window, not nonactivating panel, because reliable keyboard and VoiceOver focus outrank visual cleverness. A future pre-commit confirmation mode would require explicit pending-content state, timeout, cancellation, queue, latency, privacy, and recovery design; it is not a P0 setting.

## 3. Visual system

Original concept: [Bronze UI concept](../assets/bronze-ui-concept.png).

### Tokens

- Surfaces: opaque semantic background by default; material/transparency only with tested fallback.
- Accent: warm bronze for focus/selection/active state; text/icons retain sufficient independent contrast.
- Status: neutral, success, warning, error tokens each include icon, label, and accessible description.
- Radius: modest 8–12 px; avoid excessively pill-shaped controls.
- Spacing: 4 px base; minimum card padding 12 px; dense mode remains ≥24×24 CSS px pointer target or documented spacing exception.
- Typography: system UI stack; user content system mono optional. Base equivalent ≥14 px and resizable; never encode hierarchy by size alone.
- Shadow: optional and removed/increased-border under Reduce Transparency/Increase Contrast.

Token pairs must be measured in all themes. Normal text target 4.5:1; large text 3:1; UI components and focus indicators 3:1 adjacent contrast. Prefer margin above minimum.

## 4. Item card anatomy

Each item exposes:

- heading from stored `items.title` (`h3`); hidden when the title is absent or blank; the UI does not invent a title; persist writes `compact_title` first (40-character word-boundary clamp, no ellipsis glyph) so the heading is a short descriptive line shown in full; a local refine may replace that stored string after persist; card CSS uses `text-overflow: clip` on the title, never ellipsis; Show more / Show less apply to the body only; the source row is unchanged;
- lifecycle control with item-specific accessible label;
- kind icon plus visible/announced label when kind matters;
- content preview: constrained markdown rendered as `strong`/`em`/text plus line-start `ul`/`ol`/`li` (createElement only); body uses `white-space: pre-wrap` and collapses with `max-height` (not `-webkit-box`); catalog `queue.item.showMore` / `queue.item.showLess` expands the full body on a real `button` (`aria-expanded`);
- optional user-editable content-language metadata, defaulting to unknown rather than guessed;
- optional provenance row: catalog `capture.source` (`From {appName}`) when Capture stored a focused-process name; a 16px official app icon from the list DTO `sourceAppIcon` (`data:image/png;base64,…` only) sits beside the name when Rust can resolve the `.app` via NSWorkspace; the name stays if the icon is unavailable; no URL or window title in this build; composer rows omit the row;
- status text (`Queued`, `Copied`, `Done`, etc.);
- context action button (filled **Copy**);
- compact icon row for Complete, Skip, Edit…, Move up, Move down, and Trash (catalog `aria-label` plus hover/focus-visible tip; Trash uses the destructive token). Move up / Move down replace drag.

The row is not a clickable `div`. Use list/article semantics with actual buttons and checkbox/menu primitives. Focused row can support roving keyboard navigation only if semantics remain clear; ordinary tab order is preferred for MVP.

## 5. Keyboard model

Core panel defaults, user-remappable where marked:

| Action | Default | Notes |
| --- | --- | --- |
| Summon/hide | `⌥Space` | `app.togglePanel`; global; persist/register before commit |
| Capture selection | Shift double-tap (either side; 250/400 ms; `tapCount` 2) | `capture.selection`; live event-tap is Shift N-tap from the saved binding; non-Shift remaps disable that engine; menu Capture remains |
| New note | `⌘N` | `capture.newNote`; app-local |
| Add composer | `⌘Enter` | never fires during IME composition |
| Copy selected/focused | `⌘C` | `queue.copy`; native text selection wins when editor focused |
| Copy as profile | `⇧⌘C` | `queue.copyWithProfile` |
| Copy and advance | `⇧⌘Return` | `queue.copyAndAdvance` |
| Edit focused item | `Return` | `queue.edit`; explicit focused-row context |
| Complete | `Space` | `queue.complete`; only when row focus, never while editing |
| Move item up | `⌥⌘↑` | `queue.moveUp` |
| Move item down | `⌥⌘↓` | `queue.moveDown` |
| Search | `⌘F` | `queue.search`; moves focus and announces results |
| Undo | `⌘Z` | `queue.undo` |
| Settings | `⌘,` | `window.settings` |
| Select all items | `⌘A` | editor selection wins while editing |
| Close/hide | `Escape` | preserves draft and restores prior focus where possible |

Do not intercept VoiceOver, system, text editing, or IME chords. Shortcut resolver orders contexts: text editing → open modal/menu → focused component → app-local → global.

## 6. Focus rules

- Opening panel by shortcut: initial focus returns to last meaningful panel control, or composer on New Note.
- Capture-only success does not steal source focus.
- Opening quick editor: title/textarea receives documented initial focus; VoiceOver hears window title and status.
- Opening menu/dialog: focus enters controlled surface; Escape closes; focus returns to invoker.
- Removing focused item: focus moves to next item, previous item, then section heading/composer in that order.
- Reordering preserves focus on moved item and announces new index.
- Error summary receives focus only for blocking submit error; live status handles nonblocking results.
- Never put focus under hidden/inert panel or behind modal.

## 7. Capture feedback

Stage feedback must be immediate but not noisy:

- Trigger acknowledged: optional brief haptic/sound/visual, each independently configurable.
- Success and every other capture terminal: rust delivers a Notification Center banner via `UNUserNotificationCenter` when Bronze is a `.app`, plus `NSAccessibility.post(..., notification: .announcementRequested)` using catalog `app.name` / `panel.quick.title` and `capture.announce.saved|rejected|denied|protected|failed|excluded`. Unbundled `tauri dev` / cargo-run cannot call that API in-process; a signed sibling `BronzeNotice.app` (Bronze icon, catalog title/body only) posts instead. Title and body are catalog strings only — never the captured selection, a path, or diagnostics. Event-tap does not post notices. [Apple announcement API](https://developer.apple.com/documentation/appkit/nsaccessibility-swift.struct/notification/announcementrequested)
- `#capture-status` is a visually hidden live region. When the queue WebView is open, `#chrome-notice` (`position: fixed`) also shows that catalog line so a scrolled inbox is not the only clue. Composer add failure uses the same notice (`composer.add.error`) and stays out of the scroll flow. This is not a composer dump.
- Permission denial still offers Retest and Open System Settings; the composer stays available. Notification Center access is its own Permission health row: Allow only while `not_requested`, Open System Settings after deny, Unavailable (no Allow) in unbundled `tauri dev`. Startup requests only when undetermined.
- Secure field and excluded app: catalog `capture.announce.protected` / `capture.announce.excluded`. No source/content details in the banner.
- Busy: queued position or progress, never silent drop.
- Copy and other queue actions set `aria-busy` with a visible spinner on the control, pointer/hover/active states on every clickable control, a tip on the control, and `#action-status` catalog `copy.announce.copied` / `copy.announce.failed` after pasteboard write. Copy does not post a Notification Center banner.

Toasts do not contain sole copy of critical recovery action; Notification Center plus `#chrome-notice` keep the last no-anchor result visible when the queue is scrolled or Bronze is in the background.

## 8. First-run onboarding

Progressive, permission-late flow:

1. Explain deliberate local queue with one static diagram.
2. Create manual note; proves app value without permissions.
3. Choose capture route: standard chord recommended, optional double Shift.
4. Explain Input Monitoring before system request if gesture enabled.
5. Explain Accessibility before AX capture test.
6. Guide user to select known non-sensitive text in an external supported app such as TextEdit, then invoke capture. Testing Bronze's own WebView does not prove cross-process Accessibility or target preservation.
7. Show stage result and alternative routes.
8. Offer launch-at-login and Dock behavior; default off unless clearly beneficial.

No dark patterns. Denial leaves app usable and can be revisited.

## 9. Settings UX

Every control has label, short description, current/default value, and reset action. High-impact settings preview consequences:

- enabling synthetic clipboard fallback explains pasteboard limitation;
- changing capture shortcut tests registration before save;
- shortening retention previews deletion schedule;
- clearing data identifies exact counts and recovery window;
- changing UI language persists `SettingsV1.general.locale` (en, nl, fr, de, es, it) and applies the catalog live in open WebViews, including `html lang`/`dir`. Language options use endonyms with `lang` on each option. Item bodies stay `lang="und" dir="auto"` and are never relabeled. Native app and status menus follow the persisted locale at launch; a live switch does not rebuild the app menu. User content, focus, and the active task stay put.
- changing Title engine persists `SettingsV1.general.titleModel` and reloads the title worker without relaunch. `#title-model-status` (`role=status`, `aria-live=polite`) shows catalog idle, loading, hashing, ready, missing, or failed copy. A `span.title-engine-spinner` next to the select is visible only while loading or hashing (`aria-busy` on the status, not the select). Focus stays on the select. Reduce Motion (`prefers-reduced-motion` / `data-reduce-motion`) stops the decorative spin; the status text still updates. Missing weights keep the vendor-command copy. Failed reasons are catalog strings (`bad_hash`, `timeout`, `unreadable`) with no huge paths. Queue chrome does not show this status. This is Settings markup, not a WCAG or VoiceOver claim.

Exported settings always exclude app credentials, permission tokens, diagnostic events, and machine paths. Diagnostics use separate previewed support-bundle flow. The Settings Export preview card is a live preview of the pending file: it lists included categories, then names leftover user-entered profile literals, excluded-app policies, and custom shortcuts that may themselves be sensitive. Export is the one filled primary on Settings; Import is ghost; Help stays ghost trailing. Both actions open a rust-owned save/open panel. The WebView never supplies a filesystem path. Status after write/read is Exported or Imported (or a catalog failure), not a machine path. This file is not the Library archive.

## 10. Responsive behavior

Panel must remain usable under 200% text resize and 400% WebView zoom/reflow at 320 CSS px:

- one-column content; Copy and the icon row stay on one line; composer stays in document flow;
- no horizontal scroll for controls at 320 CSS px viewport width;
- content may scroll vertically while composer and status remain reachable;
- card actions stay as Copy plus the icon row beside it; primary Copy remains available;
- user content wraps by default; code can horizontally scroll inside bounded region;
- OS text-size preference maps to application scale presets without layout break.

## 11. Motion and materials

- Transitions ≤200 ms and not required to understand state.
- Reorder movement disables under Reduce Motion.
- Title engine spinner uses `bronze-spin`; Reduce Motion sets `animation: none`. Status text still updates.
- No parallax, flashing, or auto-moving content.
- Reduce Transparency uses opaque surface and border.
- Increase Contrast strengthens text, boundaries, focus, selection.
- Focus indicator never relies on shadow alone.

## 12. VoiceOver phrases

Examples are semantic intent; localization controls wording:

- “Capture Selection, menu item. Accessibility permission required.”
- “Research, section, 4 queued items, expanded.”
- “Item: Compare configuration formats. Queued. Position 2 of 4.”
- “Complete Compare configuration formats, checkbox, unchecked.”
- “Moved to position 3 of 4.”
- “Captured to Bronze.”
- “Copy format, pop-up button, Numbered list.”

Avoid repeating full item content on every status update. User can request details.

## 13. Design acceptance checklist

- Core flow works mouse, keyboard, VoiceOver, Voice Control, Switch Control.
- Hit targets, focus, contrast, zoom/reflow, motion/transparency variants measured.
- RTL screenshots and keyboard navigation reviewed.
- Long localized strings fit or wrap.
- No action accessible only from hover, drag, right-click, double tap, color, or toast.
- Permission denial and source incompatibility are first-class screens, not edge-case alerts.
- Visual design differs clearly from Copper while keeping workflow clarity.
