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
    locale: "system" | string
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
    motion: "system" | "reduce"
    transparency: "system" | "reduce"
    sounds: boolean
    haptics: boolean
  }
}
```

Internal safety ceilings (AX timeout, payload size) may be advanced-only or not user-facing. Never let configuration create unbounded waits/memory.

Accessibility overrides are one-way strengthening. Effective Reduce Motion, Reduce Transparency, Increase Contrast, and Differentiate Without Color equal active macOS preference OR stronger Bronze override; no app setting may weaken system request. Native bridge observes display-option changes and updates settings previews and every app window live.

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
| launch at login | off | explicit consent |
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

Global actions use native/Tauri registry. App-local resolver uses semantic contexts. An in-panel shortcut must never share same chord with global capture if both can fire.

## 5. Recorder interaction

1. Activate “Record shortcut” button.
2. Dialog explains Escape cancel, Delete clear, and reserved assistive/system combinations.
3. Live region/spoken text reports modifiers and key.
4. Recorder captures next complete chord on key release; modifier-only requires explicit gesture mode.
5. Normalize left/right only according to action schema.
6. Validate syntax and internal duplicate.
7. Attempt native registration without unregistering old binding.
8. If successful, offer user-ended/adjustable test mode with Stop/Skip and no motor-timing requirement; atomically swap registration + setting. Skipped test saves explicit untested status.
9. If failed, retain old value, announce reason and alternatives.

Recorder ignores composition, key repeat, lone character without modifier for global action, and VoiceOver/system-reserved chord where detectable. Provide typed/manual chooser for AT users who cannot use recorder.

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
- Launch at Login: optional service registration.
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
- Export Settings produces versioned JSON containing safe preferences, shortcuts, custom profiles, and built-in profile overrides. It always excludes app credentials, permission tokens, diagnostics, machine paths, install identity, and ephemeral state; preview highlights user-entered literals/app policies that may themselves be sensitive.
- Import validates version and previews changes/conflicts; shortcut registration is staged and can partially reject without corrupting other settings.
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
- exported file has no machine paths, diagnostics, app credentials, permission tokens, or internal secrets; preview covers user-entered sensitive literals/policies;
- reduced motion/transparency/contrast/differentiate-without-color system change applies live and cannot be weakened by app override;
- locale/RTL switch preserves focused control, updates WebView/native/portal/menu strings coherently, and announces completion once;
- permission revoke updates state without restart where platform allows.
- output-profile CRUD/duplicate/reset/default-delete protection and optimistic revision conflicts;
- formatter golden/property tests for every option, bounds, Unicode/newlines, prompt delimiting, and exact preview;
- UI-locale change leaves stored output literals unchanged while built-in presentation names relocalize.
