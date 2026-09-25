# Settings and shortcuts specification

## 1. Principles

- Safe defaults; permission-light start.
- Settings are typed, versioned, searchable, reversible, exportable, and validated in core.
- UI never saves a shortcut until native registration/test succeeds.
- Every global gesture has visible, keyboard, and menu alternative.
- Settings do not weaken security/accessibility silently.

## 2. Settings schema

Illustrative TypeScript contract mirrored/generated from Rust schema:

Canonical shortcut persistence is the `shortcuts` table keyed by `ShortcutActionId` (ADR-016). `SettingsV1.capture.standardChord` is the settings-window view of `capture.selection` only. Other registry actions are not omitted from the product; they are not duplicated inside the settings JSON blob. `modifierTap.action` may only map the optional double-tap gesture to `capture.selection` or `app.togglePanel`.

```ts
type ShortcutActionId =
  | "app.togglePanel"
  | "capture.selection"
  | "capture.newNote"
  | "queue.copy"
  | "queue.copyWithProfile"
  | "queue.copyAndAdvance"
  | "queue.complete"
  | "queue.edit"
  | "queue.moveUp"
  | "queue.moveDown"
  | "queue.search"
  | "queue.undo"
  | "window.settings"

type ShortcutBinding = {
  action: ShortcutActionId
  trigger: "accelerator" | "modifier_double_tap" | "disabled"
  modifiers?: Array<"Command" | "Option" | "Control" | "Shift" | "Fn">
  keyMode?: "physical" | "logical"
  physicalCode?: string
  logicalKey?: string
  modifierSide?: "either" | "left" | "right" | "same"
  gapMs?: number
  maxHoldMs?: number
  tapCount?: number
  enabled: boolean
  schemaVersion: number
  revision: number
  tested?: "tested" | "untested" | "skipped"
}

type SettingsV1 = {
  schemaVersion: 1
  general: {
    launchAtLogin: boolean
    showDockIcon: boolean
    locale: "system" | "en" | "nl" | "fr" | "de" | "es" | "it" | "ru" | "uk" | "hr" | "sl" | "da" | "sv" | "nb" | "fi" | "tr" | "en-XA" | "ar-XB"
    titleModel: "extractive" | "smol-135" | "smol-360" | "qwen-05" | "custom" | "ollama" | "hosted-openai" | "hosted-anthropic" | "hosted-openrouter"
    titleCustomId?: string
    titleCustomName?: string
    titleCustomBytes?: number
    titleOllamaModel?: string
    titleHostedBase?: string
    titleHostedConfirmed?: boolean
    reduceMotion: "system" | "on" | "off"
    startView: "last" | "activeSection" | "composer"
  }
  capture: {
    standardChord: ShortcutBinding
    modifierTap: {
      enabled: boolean
      modifier: "Shift"
      side: "either" | "left" | "right" | "same"
      action: "capture.selection" | "app.togglePanel"
      gapMs: number
      maxHoldMs: number
      firstTapFeedback: boolean
    }
    axTimeoutMs: number
    clipboardFallback: "manual" | "syntheticExperimental" | "off"
    preserveWhitespace: true
    activeSectionId: string
  }
  panel: {
    mode: "summon" | "pinned" | "autoHide"
    display: "pointer" | "frontmostApp" | "last"
    edge: "left" | "right" | "top" | "lastPosition"
    width: number
    density: "comfortable" | "compact"
    alwaysOnTop: boolean
    allSpaces: boolean
    translucency: "system" | "opaque" | "material"
  }
  copy: {
    defaultProfileId: string
    returnToPriorApp: boolean
  }
  privacy: {
    excludedBundleIds: string[]
    sourceMetadata: "none" | "app" | "appAndTitle" | "appTitleUrl"
    appPolicies: Record<string, {
      capture: "inherit" | "allow" | "deny"
      syntheticFallback: "inherit" | "allow" | "deny" | "ask"
      provenance: "inherit" | "none" | "app" | "appAndTitle" | "appTitleUrl"
    }>
    diagnosticsRetentionDays: number
  }
  data: {
    trashRetentionDays: number
    backupSchedule: "daily" | "weekly"
    backupRetention: number // closed to safe UI bounds; no off/manual-only schedule
  }
  accessibility: {
    appScale: number
    contrast: "system" | "more"
    differentiateWithoutColor: "system" | "always"
    motion: "system" | "on" | "off" // persist alias "reduce" maps to "on"; kept in sync with general.reduceMotion
    transparency: "system" | "reduce"
    sounds: boolean
    haptics: boolean
  }
}
```

Internal safety ceilings (AX timeout, payload size) may be advanced-only or not user-facing. Never let configuration create unbounded waits/memory.

Accessibility overrides for Increase Contrast, Reduce Transparency, and Differentiate Without Color are one-way strengthening: effective value equals the active macOS preference OR a stronger Bronze override. Native display-option bridge still only strengthens Reduce Motion. WebView motion is `SettingsV1.general.reduceMotion` (`system` | `on` | `off`, default `system`). `system` follows `matchMedia('(prefers-reduced-motion: reduce)')` plus a change listener. `on` always sets `document.documentElement.dataset.motion` to `reduce`. `off` always sets `full` so queue animations play even when macOS Reduce Motion is on (explicit Tools test override, not a silent weaken). Missing `general.reduceMotion` on load maps `accessibility.motion` (`system`, `reduce`, `on`, `off`). Persist copies `general.reduceMotion` onto `accessibility.motion`. Settings apply live; no restart. Persist broadcasts `ui-motion-changed`. Queue and Library first load call `load_settings_v1` (`allow-queue-live` / `allow-library-live`) plus the media query. `chrome.css` gates with `html[data-motion="reduce"]` (no anim) and `html[data-motion="full"]` (anim). `@media (prefers-reduced-motion: reduce)` is first-paint fallback only and must not win over `data-motion=full`. Existing JS still honors `data-reduce-motion`.

`SettingsV1.general.launchAtLogin` defaults to `false`. Settings → General `#launch-at-login` is a real labeled checkbox with circled-i catalog info. Save, reset field/group/all, and `LiveSession::open` apply the stored value through `SMAppService.mainApp` `register` / `unregister`. Persist does not flip the checkbox when status is `requires_approval` or `not_found`. Command `login_item_status` (`allow-settings-live`) returns `enabled` | `not_registered` | `requires_approval` | `unavailable`. Unbundled `tauri dev` and cargo-test binaries are not a `.app` and report `unavailable` without calling `register`. Cargo tests never invoke live `SMAppService`. Event-tap callbacks do not register, unregister, or read login items. `open_privacy_settings` `launchAtLogin` opens the Login Items pane. The permission-health row stays optional; the General checkbox is the user control.

## 3. Defaults

Shipped defaults:

| Setting | Default | Rationale |
| --- | --- | --- |
| standard capture chord | `capture.selection` Shift double-tap, either side, enabled | menu Capture remains if Input Monitoring is denied |
| modifier double tap | on (`capture.selection`) | same binding as `SettingsV1.capture.standardChord` |
| gap | 250 ms | responsive starting point; slider 150–900 ms |
| max hold | 400 ms | bounded 100–1,500 ms; test motor accessibility |
| clipboard fallback | manual | synthetic copy mutates shared clipboard; per-app opt-in required |
| built-in profile post-copy lifecycle | copied | do not claim task done from copy |
| provenance | none | workflow metadata is sensitive; onboarding may offer opt-in app identity |
| launch at login | off | explicit consent; Settings → General checkbox applies `SMAppService.mainApp` |
| UI locale | `system` (resolves to **en**) | Settings → General switcher persists en, nl, fr, de, es, it, ru, uk, hr, sl, da, sv, nb, fi, or tr; unknown tags reject on save; `system` and unknown effective tags map to en |
| title engine | empty auto-picks among bundled pins already on disk; otherwise last persisted `general.titleModel` | extractive uses no GGUF; `custom` stores `titleCustomId` (no path); `ollama` stores `titleOllamaModel`; hosted ids store `titleHostedBase` after confirm; a local change reloads the title worker; Settings shows load status (`title-engine-status` / `title_engine_status`); hosted keys stay in Keychain |
| reduce motion | `system` (follows this Mac) | play motion unless macOS Reduce Motion is on; Settings `on` always reduces; `off` always plays (Tools test) |
| Dock icon | off/accessory | menu-bar utility; user can enable |
| panel mode | summon | lowest intrusion |
| display | frontmost app or pointer, decided by spike | align source context |
| translucency | system with opaque accessibility fallback | style without sacrificing contrast |
| trash | 30 days | recovery |
| automatic backup | daily | DAT-002 safety baseline; Back Up Now remains available |
| diagnostics | 7 days, redacted | local troubleshooting |
| network | none | privacy promise |

## 4. Shortcut action registry

Every action has stable ID, scope, default proposal, allowed trigger forms, conflict class, text-editing behavior, and accessibility alternative.

```text
app.togglePanel
capture.selection
capture.newNote
queue.copy
queue.copyWithProfile
queue.copyAndAdvance
queue.complete
queue.edit
queue.moveUp
queue.moveDown
queue.search
queue.undo
window.settings
```

Global scope is only `app.togglePanel` and `capture.selection`. Every other action is app-local and must not grab an OS-wide chord. An in-panel shortcut must never share the same chord with global capture if both can fire.

Seeded defaults (all enabled):

| Action | Default | Scope |
| --- | --- | --- |
| `app.togglePanel` | Option+Space | global |
| `capture.selection` | Shift double-tap (`tapCount` 2), either side, gap 250 ms, max hold 400 ms | global |
| `capture.newNote` | Command+N | app-local |
| `queue.copy` | Command+C | app-local |
| `queue.copyWithProfile` | Command+Shift+C | app-local |
| `queue.copyAndAdvance` | Command+Shift+Enter | app-local |
| `queue.complete` | Space | app-local |
| `queue.edit` | Return | app-local |
| `queue.moveUp` | Option+Command+ArrowUp | app-local |
| `queue.moveDown` | Option+Command+ArrowDown | app-local |
| `queue.search` | Command+F | app-local |
| `queue.undo` | Command+Z | app-local |
| `window.settings` | Command+Comma | app-local |

There is no `queue.paste`. Factory-disabled seed rows (revision 1, trigger disabled, not enabled) load as these defaults. User customizations stay. `SettingsV1.capture.standardChord` remains the settings view of `capture.selection` (ADR-016).

Event-tap live fire is `capture.selection` when that binding is Shift N-tap (`tapCount` 2 through 8; absent means 2). The listen-only FSM triggers on the Nth Shift release, not earlier, using gap 250 ms, max hold 400 ms, either side, and refractory 500 ms. Non-Shift or non-multi-tap remaps of `capture.selection` disable the Shift engine. Other seeded globals persist through native registration without an OS grab. App-local chords and recorded Option/Command/Control/Fn multi-taps persist for the UI resolver; Quick Panel keydown still uses hardcoded composer chords. When Bronze is frontmost, that same Capture path persists a WebView highlight from Quick, Settings, Library, or Help (fixed Rust script; password fields excluded) before last-external AX.

## 5. Recorder interaction

Settings Shortcuts lists every action with its current chord. The assignment control (catalog Record shortcut) starts the recorder for that row. Restore Default is visible only when the row is not the seeded default.

1. Activate the assignment control on that row. Recording stays on the row; no extra field is inserted above the list. The assignment control shows Recording… until a chord is saved or Escape cancels.
2. The next complete accelerator is captured inline, including multi-modifier combinations. Option+letter uses the physical key (ø from Option+O binds as O). Repeated taps of the same modifier (Shift, Option, Command, Control, or Fn) record `modifier_double_tap` for that modifier. The first tap previews the glyph and does not persist. Each further tap within 500 ms increments `tapCount` (2 through 8) and resets the commit timer. The binding persists after 500 ms with no further tap, or immediately at 8 taps. A stored row without `tapCount` compares as 2. Timing defaults are gap 250 ms, max hold 400 ms, and either side. Escape leaves the old binding.
3. Live region reports recording, reject, or duplicate from the catalog. The painted chord is a catalog string for that modifier and count (double-tap, triple-tap, or `{modifier} {count}-tap`). No sentence concatenation.
4. Normalize left/right only according to action schema.
5. Validate syntax and internal duplicate. Two actions may not share the same assigned chord. Assigned-chord identity includes trigger, modifiers, logical key, gap, hold, side, and effective tap count. Option double-tap and Option triple-tap are distinct.
6. Attempt native registration without unregistering the old binding. App-local multi-tap must not OS-grab.
7. Persist only after native registration succeeds or the operator skips the test. A skipped test stores explicit untested status.
8. If registration fails, retain the old value and announce the catalog reason.

Recorder ignores composition, key repeat, a lone Backspace/Delete, a lone character without a modifier for a global action, and a VoiceOver/system-reserved chord where detectable. Provide a typed/manual chooser for AT users who cannot use the recorder.

## 6. Conflict model

Conflict classes:

- exact internal conflict in same/overlapping scope: hard reject;
- OS/native registration failure: hard reject and retain old;
- known macOS/reserved shortcut: hard reject or strong warning based on certainty;
- likely app/text-editing collision: warning;
- keyboard-layout ambiguity: warning with logical/physical choice;
- assistive-technology chord: strong warning and accessible explanation;
- modifier double tap likely conflicts with ordinary typing/Sticky/Slow Keys: test flow and alternative.

No API can enumerate every other app’s shortcut. UI says “registered successfully,” not “conflict-free everywhere.”

## 7. Modifier double-tap configuration

Controls:

- enabled;
- modifier (P0 Shift only; future Option/Control requires research);
- left/right/either/same side;
- gap slider 150–900 ms; keyboard numeric input; at least 500 ms allowed;
- max hold 100–1,500 ms bounded;
- first-tap feedback visual/sound/haptic;
- action mapping capture/toggle, avoiding ambiguous side schemes;
- test area showing accepted/rejected reason without saving content.

Show Input Monitoring state and Secure Input limitation. Reset FSM after setting change. Never listen to characters for display/logging.

## 8. Permission health settings

Each row includes status, why needed, last test, Retest, Open System Settings/help, and alternative.

- Input Monitoring: global modifier event tap.
- Accessibility: AX selection and synthetic fallback.
- Notifications: Notification Center banners. Status comes from `notification_authorization_status` (never auto-request from the queue WebView). Allow calls `request_notification_authorization` only while `not_requested`. Denied shows Open System Settings (`open_privacy_settings` notifications). Unbundled `tauri dev` reports `unavailable` and hides Allow; banners still go through the signed `BronzeNotice.app` helper.
- Launch at Login: optional `SMAppService.mainApp` registration. Status comes from `login_item_status`. The General checkbox persists `general.launchAtLogin` even when Login Items still requires approval. Unbundled debug reports `unavailable`. Open System Settings uses the Login Items pane (`launchAtLogin`).
- Automation: absent P0 unless a later feature requires it.
- Screen Recording: explicitly “Not used.”

Health statuses are not booleans: `unknown`, `not_requested`, `denied`, `granted_unverified`, `healthy`, `degraded`, `unavailable`, `requires_relaunch`. Self-test never captures unrelated content.

## 9. Privacy settings

`privacy.excludedBundleIds` is a list of stable bundle IDs. Settings Privacy is a searchable multi-select of apps installed on this Mac (CAP-009): the operator adds many apps from that list, not a free-text bundle dump. Catalog copy states that Bronze will not capture selections from those apps, and that people can search to add many or choose a missing app from Finder. The find control matches header-search craft; Reset stays ghost beside the field. `list_installed_apps` returns bundle ID and display name only. `pick_installed_app` opens a rust-owned `NSOpenPanel` restricted to `.app` packages (unbounded AppKit hop) and returns the same DTO, or `picker_cancelled` / `picker_unavailable` / `invalid_app`. WebView never receives filesystem paths (SEC-001). Official icons come from the local app via `app_icon_data_url`. Chip remove is an icon-only control whose accessible name is catalog `settings.field.excludedBundleIds.remove`. If the installed-app list is unavailable, search is disabled, Choose app… stays available, and already-saved IDs remain removable. Event-tap callbacks do not open the panel.

Live capture peeks the last-external bundle ID and returns `app_excluded` before any AX selection read when that bundle is on the list (F-CAP-01). That terminal delivers catalog `capture.announce.excluded` as a Notification Center banner and `#chrome-notice`, never captured text. Event-tap callbacks do not list apps, read icons, or post notices.

Application policy picker lists installed/running app metadata without scanning content. Per stable bundle ID it also controls synthetic fallback and provenance. Unknown/missing/changed bundle IDs inherit capture setting but deny synthetic fallback and provenance by default until user confirms. Built-in recommended exclusions: password managers, authentication tools, Terminal/iTerm secure-input contexts, banking/health apps where identifiable; user confirms rather than hidden defaults.

Provenance levels show exact sample output. P1 may add local advisory secret-pattern warning with false-positive explanation and no matched value in diagnostics; it is absent from P0 `SettingsV1`.

“Runtime network” panel should show P0 statement: no network features compiled/enabled. If updater later added, ledger shows purpose, host, last request, payload categories, toggle. Synthetic clipboard fallback disclosure explains that macOS Clipboard History, Universal Clipboard, and third-party managers may retain copied text; Bronze cannot retract those copies.

Output profile is sole authority for `postCopyAction` and `advancePolicy`; no global lifecycle setting may override it. `returnToPriorApp` means external application active immediately before Bronze panel activation, snapshotted when copy command starts. It never comes from item provenance and never synthesizes paste.

## 10. Copy and output profiles

Profiles are normalized domain entities, not embedded into `SettingsV1`; `copy.defaultProfileId` references one. Typed contract:

```ts
type OutputProfileV1 = {
  id: string
  builtinKey?: "plain" | "bullets" | "numbered" | "promptBlock"
  name: string
  format:
    | { kind: "plain"; separator: string }
    | { kind: "markdownBullets"; marker: "-" | "*" | "+"; separator: string }
    | { kind: "markdownNumbered"; startAt: number; separator: string }
    | {
        kind: "promptBlock"
        contextHeading: string
        instructionHeading: string
        itemDelimiter: string
      }
  sourcePolicy: "none" | "app" | "appAndTitle" | "appTitleUrl"
  postCopyAction: "unchanged" | "copied" | "active" | "done"
  advancePolicy: "keep" | "nextQueued"
  revision: number
}
```

- Create, duplicate, rename, edit, reset built-in, and delete custom profile. Default profile requires replacement before deletion.
- Profile editor preview uses fixed hostile Unicode/multiline sample, so Settings window needs no item-content authority. Exact current-item preview remains explicit Quick-panel action under its existing queue capability. Neither preview executes or fetches content.
- Separators/headings/delimiters are bounded literal text. Reject NUL, over-limit values, raw-HTML mode, and invalid numbered start. No template language exists; `${...}`, braces, and markup-looking text remain inert literals.
- `sourcePolicy` only filters provenance already stored under capture privacy policy; profile cannot retrieve missing title/URL or override per-app provenance denial.
- Built-in names localize at presentation. Prompt headings seeded when profile is created become stored output literals; later UI-locale change does not rewrite user output unexpectedly.
- Every field has label/instructions/error association. Preview remains reachable at 400% reflow; keyboard/VoiceOver can create and restore profile without drag.
- Import stages profiles, resolves ID/name/default conflicts, and never overwrites custom profile silently.

## 11. Reset and portability

- Per field Reset to Default.
- Per group Reset Group with preview.
- Reset All preserves content and backups unless separate Data action selected.
- Export Settings writes a separate versioned JSON file (`format: "bronze-settings"`, `version: 1`) containing safe `SettingsV1` preferences, the shortcut registry, and output-profile rows. It is not the Library `bronze-export` archive and must not reuse `export_library_archive` / `import_library_archive`. The file always excludes app credentials, permission tokens, diagnostics, machine paths, install identity, and ephemeral state (`tested` on bindings, install identity). Settings → Export preview lists included categories (General, Capture, Panel, Copy, Privacy, Data, Accessibility, Shortcuts, Profiles) and names leftover user-entered literals that may themselves be sensitive: excluded apps, app policies, capture shortcut, default profile id, custom shortcut chords, and profile literals. Rust commands are `preview_settings_export`, `export_settings_file`, and `import_settings_file`. The save panel is rust-owned (`NSSavePanel` via BronzeNative, default name `bronze-settings.json`). WebView may only pass `requestedPath: null` (SEC-003). The IPC result is a filename, never a machine path.
- Import uses a rust-owned `NSOpenPanel`, reads at most 1 MiB, requires JSON `format: "bronze-settings"` and known `version`, and rejects a Library archive, forbidden keys (credentials, tokens, diagnostics, paths), and any WebView-supplied path. Schema/version errors surface as named codes (`settings_wrong_format`, `settings_unknown_version`, `settings_forbidden`, `settings_invalid`). Shortcut registration is staged and can partially reject without rolling back already-applied settings or profiles. Import does not silently apply ADR-018 (locale-aware search stays Proposed). Event-tap callbacks still do not open panels or touch AX, DB, windows, or clipboard.
- Back Up Now is always available. Automatic daily/weekly backup cannot be replaced by manual-only schedule; pre-migration/pre-restore safety backups remain mandatory.
- Factory erase lists DB, backups, exports not owned by app, settings, and permissions. App cannot silently remove TCC entries or erase clipboard copies already retained by macOS history, nearby devices, or third-party managers; explain those separate steps.

## 12. Settings tests

- schema round trip and unknown-version rejection;
- every default produces valid safe state;
- migration from every settings schema;
- shortcut old binding survives failed new registration;
- duplicate/global/local overlap detection;
- QWERTY/AZERTY/QWERTZ/Dvorak logical/physical display;
- VoiceOver/keyboard recorder flow;
- search finds localized synonym and category;
- reset field/group/all scope correct;
- exported `bronze-settings` file has no machine paths, diagnostics, app credentials, permission tokens, or internal secrets; preview lists included categories plus user-entered sensitive literals/policies; import rejects WebView paths, `bronze-export` archives, unknown versions, and oversized files; rust-owned save/open panels are not invoked from cargo tests;
- `general.reduceMotion` defaults to `system` and round-trips `on` / `off` / legacy `reduce`; unknown values reject; reset field returns `system`; export/import include the field; `system` follows `prefers-reduced-motion` live; `on` sets `data-motion=reduce`; `off` sets `data-motion=full` even when the OS requests reduce; Increase Contrast, Reduce Transparency, and Differentiate Without Color still cannot be weakened by app override;
- `general.launchAtLogin` defaults to `false` and persists through save/reset/export; apply uses `SMAppService.mainApp`; `requires_approval` and `not_found` do not flip the stored checkbox; `login_item_status` maps enabled / not_registered / requires_approval / unavailable; unbundled debug is unavailable; cargo tests never call live `SMAppService`; event-tap files must not contain `SMAppService`;
- locale/RTL switch persists `general.locale`, reapplies the WebView catalog and `html lang`/`dir`, keeps item bodies `lang="und" dir="auto"`, and labels switcher options with endonyms plus option `lang`; native app/status menus follow the persisted locale at launch only;
- Title engine load status maps `switch scheduled` → loading, `weights resolved` / `hash ok` → hashing, `model loaded` → ready, extractive → idle, `missing_weights` → missing, and load fallbacks `bad_hash` / `timeout` / `unreadable` → failed; Settings shows spinner plus catalog Loading `{engine}` while loading or hashing, ready/loaded copy when ready, and vendor-command copy when missing; event `title-engine-status` and command `title_engine_status` stay Settings-only; capture never waits; focus stays on the select; Reduce Motion stops decorative spin; no WCAG or VoiceOver claim;
- permission revoke updates state without restart where platform allows.
- output-profile CRUD/duplicate/reset/default-delete protection and optimistic revision conflicts;
- formatter golden/property tests for every option, bounds, Unicode/newlines, prompt delimiting, and exact preview;
- UI-locale change leaves stored output literals unchanged while built-in presentation names relocalize.
