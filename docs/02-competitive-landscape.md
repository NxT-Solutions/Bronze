# Bronze competitive landscape

Status: decision-ready research  
Research date: 2026-08-27  
Scope: macOS-first selection capture, transient work queues, clipboard workflows, scratchpads, quick entry, prompt libraries, and local-first productivity tools  
Price convention: public USD price where available, before tax. Prices, promotions, App Store tiers, and subscription terms can change. Recheck before publication or purchasing.

## Executive decision

Bronze should not compete as another clipboard-history utility, general notes app, task manager, or AI client.

Recommended category:

> Bronze is an offline, local-first, accessibility-first selection-to-action queue for AI-assisted and human workflows on macOS.

Core loop:

1. Select text in any supported application.
2. Invoke a configurable capture gesture or shortcut.
3. Preserve selected text and safe source context in a local queue.
4. Add follow-up prompts, notes, or checklist items.
5. Copy one item or a context bundle back to ChatGPT, Claude, Cursor, an editor, or another target.
6. Mark work done, skip it, restore it, or advance to the next item.

No reviewed product combines deliberate selection capture across a documented app matrix, transient ordered prompt/task semantics, one-step copy-back, local-only default storage, published accessibility evidence, and portable open data formats.

macOS 26 now includes Spotlight Clipboard History. Commodity clipboard-history positioning is therefore weak. Bronze must own semantic workflow: selected context becomes explicit work, then moves through an observable lifecycle.

## Scope correction

Copper is not a screen recorder. It combines a local to-do list, clipboard-like capture, and scratchpad for AI-assisted work. Double-Shift captures selected text; users add follow-up prompts, copy material back to ChatGPT, Claude, or Cursor, then check work off.

Screen-recording products such as Screen Studio, Tella, Focusee, Loom, Cap, and CleanShot are outside this competitive set. They should not influence Bronze requirements except where generic macOS permissions, media capture, or distribution lessons later become relevant.

## Category map

| Category | Representative products | Market strength | Missing versus Bronze |
| --- | --- | --- | --- |
| Selection action tools | ActionClip, PopClip, OnText, Marker, Pluks | Mature selected-text acquisition, contextual actions, automation | No explicit ordered work/completion queue |
| Clipboard queues/managers | Pastebot, PastePal, PasteBar, Paste, CleanClip, Stacked, Clipbara, Batch Clipboard, FlowClip, Maccy | History, search, collections, sequential paste | Usually passive recording; weak task state and prompt semantics |
| Transient scratchpads | Antinote, SideNotes, Drafts, Tot, Scratchpad, Jot | Fast capture, editing, checklists, export | No universal selection-to-copy-back lifecycle |
| Task quick entry | Things, Todoist, TickTick, Superlist | Global entry, completion, organization | Heavy task model; poor clipboard/context feedback loop |
| Prompt libraries | Promta, Prompt Box, PromptBox.ai, Snippety | Reusable prompts, variables, search, expansion | Optimized for permanent assets, not ephemeral context work |
| Automation platforms | Raycast, Alfred, BetterTouchTool, Keyboard Maestro, Espanso | Can approximate full workflow | High setup cost; no opinionated portable queue |
| Knowledge bases | Obsidian, Joplin, Pieces | Durable local knowledge and broad integrations | Too permanent or passive for short-lived work |
| OS baseline | macOS Spotlight Clipboard, Apple Quick Note | Free, built in, native | No semantic selection-to-action queue |

## Competitive capability matrix

Legend: Yes = native product behavior; Partial = achievable with adjacent feature, extension, or manual step; No = not meaningfully supported in reviewed material.

| Product | Selected-text capture | Transient ordered queue | Completion state | Sequential copy/paste | Local-first | Open source | Extensible |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Copper | Yes | Yes | Yes | Partial | Yes | No | No |
| Cooper | Yes | Yes | Yes | Partial | Yes | Yes | Partial |
| Antinote | Partial | Partial | Yes | Partial | Yes | No | Yes |
| SideNotes | Partial | Partial | Yes | Partial | Yes | No | Yes |
| Steecky | No | Partial | Yes | No | Yes | No | No |
| PasteBar | No | Partial | No | Yes | Yes | No (source-available) | Yes |
| Pastebot | No | Yes | No | Yes | Yes | No | Partial |
| PastePal | No | Yes | No | Yes | Yes | No | Yes |
| Paste | No | Yes | No | Yes | Partial | No | Partial |
| Drafts | Partial | Partial | Partial | No | Yes | No | Yes |
| Things | Partial | Yes | Yes | No | Yes | No | Yes |
| ActionClip | Yes | No | No | No | Yes | No | Yes |
| PopClip | Yes | No | No | No | Partial | No | Yes |
| OnText | Yes | No | No | No | Partial | No | Yes |
| Marker | Yes | Partial | No | No | Yes | Yes | Yes |
| Pluks | Yes | Partial | No | No | Partial | No | No |
| CleanClip | No | Yes | No | Yes | Partial | No | No |
| Stacked | No | Yes | No | Yes | Yes | Yes | No |
| Clipbara | No | Partial | No | No | Yes | Yes | No |
| Promta | No | No | No | Partial | Partial | No | Yes |
| Prompt Box | Partial | No | No | Partial | Yes on free tier | No | Partial |
| Scratchpad | Partial | No | No | No | Yes | No | Yes |
| Tot | Partial | Partial | Yes | No | Partial | No | Yes |
| Raycast | Partial | Partial | Partial | Partial | Partial | No | Yes |
| BetterTouchTool | Yes | Partial | Partial | Yes | Yes | No | Yes |
| Apple Spotlight Clipboard | No | No | No | No | Yes | No | Partial |

## Direct products

### Copper

- Source: [shadcn Copper](https://shadcn.com/copper)
- Price: $39 one-time.
- Platform: macOS 14 or later.
- Workflow: double-Shift captures selected text; users add notes/prompts, copy them back to AI or editor applications, and check them off.
- Privacy: no account, sync, or collection; data stored in a local file.
- Permission: macOS Accessibility access required for selected-text and shortcut behavior.
- Published evidence gap: no formal accessibility conformance, VoiceOver test statement, or localization inventory found.

Decision: behavior reference, not implementation reference. Bronze should preserve low-friction capture and task closure while making reliability, configurability, accessibility, and data portability explicit.

### Cooper

- Source: [TouchMyBar/cooper](https://github.com/TouchMyBar/cooper)
- Price/license: free; Apache-2.0.
- Platforms: macOS, Windows, Linux.
- Stack: Tauri 2, Rust, web UI, SQLite.
- Workflow: double left Shift captures selection; double right Shift shows the panel; fallback chords support capture and panel display.
- Features: sections, Markdown formatting, images/files, multi-select copy as Markdown, themes, login launch, tray, Markdown export.
- Privacy: local SQLite; no account, telemetry, or sync.
- Maturity: small, new project with limited issue history.

Historical evidence: [issue #3](https://github.com/TouchMyBar/cooper/issues/3) documents the `v0.2.1` Apple Silicon crash after keyboard input even with Accessibility and Input Monitoring permissions. That release's raw `rdev` listener reached HIToolbox from the wrong dispatch context and terminated with SIGTRAP. macOS `v0.3.0` removed `rdev`; audited `v0.3.5` uses a custom CGEventTap and has different reliability risks documented in the [Cooper audit](16-cooper-oss-audit.md). Registered fallback shortcuts remain viable.

Decision: study behavior and failure patterns, but copy no Cooper source into Bronze v1. Native macOS event handling and signed release identity are P0. Any future source reuse requires separate ADR, file-level provenance, Apache notices, and dependency-license review.

### Antinote

- Source: [Antinote features](https://antinote.io/features)
- Price: $5 one-time with lifetime updates.
- Platform: macOS 14 or later.
- Workflow: global hotkey, temporary notes, checklists, AutoPaste collection mode, on-device OCR, one-click export.
- Storage: local SQLite; fully usable offline.
- Privacy: no account or analytics; optional encrypted iCloud sync; per-feature network controls.
- Extensibility: JavaScript extensions, URL export, Alfred and Raycast integrations, optional BYO LLM/API calls.

Decision: strongest value and scratchpad benchmark. Bronze needs selection capture plus explicit queue state to remain distinct.

### SideNotes

- Sources: [official site](https://www.apptorium.com/sidenotes), [Mac App Store](https://apps.apple.com/us/app/sidenotes-screen-edge-notes/id1441958036?mt=12)
- Price: $19.99 one-time for current major version; 30-day website trial.
- Platform: macOS 13 or later.
- Workflow: screen-edge drawer, hot side, menu-bar access, global shortcut, quick notes/tasks/links/snippets, checklists, quick copy, folders, search.
- Integrations: Apple Shortcuts, AppleScript, URL API, Alfred, Raycast, PopClip, Hookmark, Dropzone.
- Storage/privacy: local storage; optional private iCloud sync; automatic backups; no analytics or telemetry stated.
- Localization: English plus eight additional languages.

Decision: strongest polished panel, information architecture, and integration benchmark. Bronze should stay more transient and queue-oriented.

### Steecky

- Source: [Steecky](https://steecky.app/)
- Price: free tier; Pro $3.99/month or $39.99/year.
- Platform: macOS 14 or later, Intel and Apple Silicon.
- Workflow: menu-bar notes, tasks, clipboard, code snippets, links, and trash.
- Privacy: local-only, offline, sandboxed; no account; application exclusions for sensitive clipboard sources.
- Free limits: unlimited notes, last ten clipboard items, two tasks, five snippets, five links.

Decision: demonstrates demand for same component nouns, but components remain separate silos. Bronze advantage is an integrated selection-to-work lifecycle.

### PasteBar

- Sources: [PasteBar](https://www.pastebar.app/), [GitHub repository](https://github.com/PasteBar/PasteBarApp)
- Price: free; source-available, not OSI open source.
- Platforms: macOS and Windows.
- Stack: Tauri, Rust, React, Vite, Tailwind.
- Workflow: unlimited clipboard history, custom clips, tabs/boards, notes, Markdown/code, instant paste, forms/templates, web/API utilities.
- Privacy: local data, custom data location, passcode lock, backups, no account.
- Localization: nine languages documented.
- License: [CC BY-NC plus custom limited-commercial exception at reviewed revision](https://github.com/PasteBar/PasteBarApp/blob/c89d054d29a0b51fdd3a031757a8dd21a09bc241/CC-LICENSE). Noncommercial restriction fails [Open Source Definition](https://opensource.org/osd). Legal review required before any code reuse, especially if Bronze distribution could be commercial.

Decision: best broad Tauri architecture benchmark. Reimplement behavior unless license counsel approves reuse.

### Pastebot

- Sources: [Pastebot](https://tapbots.com/pastebot), [purchase page](https://tapbots.com/pastebot/buy/)
- Price: Pastebot 3 direct license $39 including one year of updates; another update year $19. App Store option $2.99/month or $24.99/year. $12.99 is the Pastebot 2 upgrade discount, not current base price.
- Platform: macOS 26 or later.
- Workflow: clipboard history, Pastebins, Stacks, filters, Quick Paste, shortcuts, blacklist, and optional iCloud. Earlier Pastebot 2 material used scratchpad/sequential-paste terminology; treat that wording as historical.
- Privacy: local by default; optional iCloud; application blacklist.

Decision: primary benchmark for FIFO queue clarity, next-item visibility, and sequential paste recovery.

### PastePal

- Sources: [PastePal](https://indiegoodies.com/pastepal), [Mac App Store](https://apps.apple.com/us/app/clipboard-manager-pastepal/id1503446680?platform=mac)
- Price: $14.99 one-time universal purchase.
- Platforms: macOS 13+, iPhone, iPad, Apple Vision.
- Workflow: collections, Quick Mode, screen-edge bar, Paste Stack, snippets, more than 75 transforms, keyboard/share/Shortcuts integrations.
- Privacy: local storage, no tracking or third-party server; optional iCloud and local peer sharing; application/content exclusions.
- Localization: English plus 35 languages.
- macOS accessibility declaration: VoiceOver, Voice Control, and dark interface. The same listing declares 36 UI languages. Larger Text at 200%+ appears on its iPad declaration, not the Mac declaration, so it is not used as macOS evidence.

Decision: strongest public accessibility and internationalization benchmark. Claims remain developer declarations, not independent conformance proof.

### Drafts

- Sources: [getting started](https://docs.getdrafts.com/gettingstarted/), [macOS capture window](https://docs.getdrafts.com/docs/extensions/capture-window), [Drafts Pro](https://docs.getdrafts.com/draftspro)
- Price: core free; Pro $1.99/month or $19.99/year.
- Platforms: macOS, iPhone, iPad, Apple Watch.
- Workflow: global capture window, new/append/prepend capture, inbox-first text, programmable actions.
- Privacy: offline-capable; Apple ecosystem sync.

Decision: benchmark “capture now, route later.” Bronze should add source selection and queue lifecycle without inheriting full writing-workspace scope.

### Things

- Sources: [Things](https://culturedcode.com/things/), [Quick Entry](https://culturedcode.com/things/support/articles/2249437/), [pricing FAQ](https://culturedcode.com/things/support/articles/2803552/)
- Price: Mac $49.99 one-time; mobile applications separate.
- Workflow: global Quick Entry; Quick Entry with Autofill captures current context/link from supported applications; clipboard lines can become separate to-dos.
- Privacy: local operation plus optional free Things Cloud.

Decision: benchmark quick-entry semantics, completion feedback, keyboard behavior, and context autofill. Avoid full project/date/recurrence scope.

### ActionClip

- Sources: [ActionClip](https://actionclip.app/), [built-in actions](https://actionclip.app/actions), [privacy policy](https://actionclip.app/privacy)
- Price on research date: free for 30 actions/day; $8 early-bird individual lifetime against advertised $19 list price. Promotion is highly volatile.
- Platform: native macOS 15+.
- Workflow: selected-text widget, smart action suggestions, broad built-in action inventory, custom workflows, copy/paste/format/translate/note/AI operations.
- AI/privacy: local Apple Intelligence and Ollama options; external providers are opt-in/BYO key; local actions stay on device.
- Account: none required; paid activation uses license code.

Decision: leading current benchmark for selected-text acquisition, contextual suggestions, permission explanation, and local-versus-online disclosure.

### PopClip

- Sources: [PopClip](https://www.popclip.app/), [developer documentation](https://www.popclip.app/dev/), [purchase page](https://www.popclip.app/buy)
- Price: paid Standard and Lifetime licenses; dynamic storefront did not expose a stable public amount during research.
- Workflow: automatic action bar after text selection.
- Extensibility: URL, key press, macOS Service, Shortcut, JavaScript/TypeScript, shell, and AppleScript actions; filters by application, content type, and regex.
- Security: signed extension directory; warnings for unsigned executable extensions.

Decision: primary external integration target. A Bronze URL scheme plus official PopClip extension can provide selected-text capture without making automatic popups mandatory.

### OnText

- Sources: [OnText](https://gityeop.github.io/OnText/), [PopClip comparison](https://gityeop.github.io/OnText/docs/guides/popclip-alternative)
- Price: name-a-fair-price distribution.
- Platform: native macOS, Intel and Apple Silicon.
- Workflow: select text, invoke configurable hotkey, then run an action; default trigger documented as F2.
- Extensibility: URL, shell, AppleScript, macOS Shortcuts, regex/context rules, and cloud/local AI.

Decision: useful keyboard-first alternative to automatic selection popups. Bronze should support both explicit chord and optional gesture.

### Marker

- Sources: [official site](https://getmarkerapp.net/), [MIT repository](https://github.com/Mazide/marker).
- Platform/price: free, macOS 14+, native Swift.
- Workflow: AX observes selected text into separate searchable history; explicit Option-V/middle-click paste, menu action, CLI, App Intents, and URL scheme. App exclusions and local SQLite are documented.
- Caution: unsupported-app and paste paths synthesize copy or temporarily swap/restore shared clipboard. Bronze must not inherit restoration guarantees because pasteboard has no compare-and-swap.

Decision: closest open AX-first selected-text/history benchmark and strongest external-interface benchmark. Bronze remains deliberate-triggered, transient, and no-restore.

### Pluks

- Source: [official site](https://pluks.app/); public GitHub repository contains release artifacts, not reviewed source.
- Price/platform: free; macOS 11+ universal build, Windows beta; Tauri/Rust claimed by vendor.
- Workflow: automatically detects selection gestures, synthesizes copy, keeps up to 200 clips, and offers searchable overlay.
- Privacy: content stays local, but anonymous usage statistics and crash reports are enabled with opt-out; secure-field/concealed-clipboard controls are vendor claims.

Decision: useful permission/onboarding and Tauri-size comparison, but ambient selection capture and default telemetry conflict with Bronze scope.

### Prompt libraries

#### Promta

- Source: [Promta](https://promta.app/)
- Price: free for 25 prompts and five AI improvements/month; Pro $4.99/month, $24.99/year, or $49.99 lifetime.
- Platform: Apple Silicon Mac on macOS 14+, iPhone/iPad.
- Workflow: menu bar, hotkey, prompt variables, search/tags, iCloud sync, MCP access.

Decision: copy MCP and variable-template ideas later. Do not turn P0 into a durable prompt asset manager.

#### Prompt Box

- Source: [Prompt Box](https://promptboxapp.com/)
- Price: unlimited local prompts free; Pro cloud sync $2.99/month, $19/year, or $39 founding lifetime.
- Platform: browser extension.
- Workflow: prompt library, browser text expansion, selected-text context-menu save, CSV import/export.
- Privacy: local free tier requires no account/server/tracking.

Decision: browser-only competitor validates local prompt storage and quick retrieval. Bronze focuses native transient work.

#### PromptBox.ai

- Source: [PromptBox.ai](https://app.promptbox.ai/)
- Price observed: $5.99/month or $79 lifetime.
- Platforms: macOS, iOS, Windows.
- Workflow: system-wide text expansion, nested folders, team sharing.

Decision: separate product from Prompt Box. Relevant to reusable prompt expansion, not selected-context queueing.

### Minimal scratchpads

#### Scratchpad

- Source: [Scratchpad](https://sindresorhus.com/scratchpad)
- Price: $8 universal purchase.
- Workflow: one persistent plain-text note, custom global hotkey, Services, Share, Shortcuts, URL scheme, and published PopClip integration.
- Privacy: optional iCloud, no tracking, local rolling backups.
- Limitation: developer explicitly does not plan localization.

Decision: benchmark safe URL API design, backups, and simple native behavior.

#### Tot

- Source: [Tot](https://tot.rocks/)
- Price: Mac application free; mobile unlock $19.99.
- Workflow: seven finite pads, rich/plain/Markdown, checklists, menu bar, Shortcuts.
- Privacy: iCloud sync.
- Localization: English only.
- Accessibility history: VoiceOver improvements, non-color differentiation, reduced-transparency work, and RTL fixes appear in release history.

Decision: finite workspace encourages cleanup. Bronze should not use color as sole section identity.

#### Jot

- Source: [Jot](https://jot.arunbrahma.com/)
- Price: $4.99 one-time.
- Platform: macOS 14+, Intel and Apple Silicon.
- Workflow: Option-Space quick capture into searchable local log; Markdown export.
- Privacy: local, no account, no sync.

Decision: establishes low price floor for capture-only utilities.

## Platform baseline and adjacent products

### macOS 26 Spotlight Clipboard

- Source: [Apple support](https://support.apple.com/en-euro/guide/mac-help/mchl40d5b86b/26/mac/26)
- Access: Command-Space, then Command-4.
- Supports searching and re-copying recent text, images, links, and files.
- User must enable feature; Apple warns sensitive data may appear.

Decision: do not spend Bronze scope reproducing generic history. Support macOS 14+ if desired, but frame history as deliberate Bronze captures, not a replacement clipboard recorder.

### Apple Quick Note

- Source: [Apple support](https://support.apple.com/guide/notes/create-a-quick-note-apdf028f7034/mac)
- Access: Globe/Fn-Q or hot corner.
- Safari can add selected text/highlight with page context.

Decision: native baseline for quick capture. Bronze must work across applications and add queue lifecycle.

### Automation platforms

| Product | Current public price | Relevant capability | Bronze implication |
| --- | ---: | --- | --- |
| [Raycast](https://www.raycast.com/pricing) | Free; Pro $10/month or $8/month annual | Clipboard, notes, snippets, Quicklinks, extensions, local encrypted history | Offer extension/API; preserve focused standalone experience |
| [Alfred](https://www.alfredapp.com/powerpack/) | £34 current major; £59 lifetime upgrades | Clipboard, snippets, workflows, selection/clipboard automation | Publish workflow and URL/CLI integration |
| [BetterTouchTool](https://folivora.ai/buy/) | $15 standard; $25 lifetime | Global/selection triggers, local clipboard, scripts, menus, webviews | Benchmark configurability; avoid exposing comparable complexity |
| [Keyboard Maestro](https://www.keyboardmaestro.com/main/) | $36 | Macros, named clipboards, history, scripts | Support macro-friendly URL/CLI commands |
| [Espanso](https://espanso.org/) | Free, GPL-3.0 | Local Rust text expansion, forms, scripts, file configuration | Consider import/export for prompt templates later |

### Clipboard products

- [Maccy](https://maccy.app/): free MIT open source, native, local, keyboard-first. Use only the official maccy.app domain.
- [Paste](https://pasteapp.io/pricing): $2.49/month or $29.99/year personal; polished synced visual history and pinboards. [Paste Stack](https://pasteapp.io/help/using-paste-stack) is a temporary ordered queue: Shift-Command-C starts collection, Command-V consumes items top-to-bottom, and direction can reverse.
- [Unclutter](https://unclutterapp.com/): $19.99 once; clipboard, file drop zone, and notes in a top-screen drawer; 11 UI languages.
- [CopyQ](https://copyq.readthedocs.io/en/stable/): free GPL open source; programmable cross-platform history, tabs, editing, tagging, scripts, and exclusions.
- [Batch Clipboard](https://github.com/jpmhouston/Batch-Clipboard): free MIT open source; explicit FIFO batch copy/paste using Control-Command-C/V; currently English-only.
- [FlowClip](https://github.com/gityeop/FlowClip): free Maccy fork; visual queue, FIFO paste, paste-all, custom separators.
- [Copy](https://github.com/tarikbc/Copy): free open source; visual shelf, pinboards, local editor, and numbered paste stack; no analytics.
- [CleanClip](https://cleanclip.cc/): caret-adjacent quick menu plus Paste Stack; Command-V consumes next ordered item, Split Copy turns lines into stack entries, and workflow targets form filling. Passive clipboard history remains broader than Bronze.
- [Stacked](https://github.com/sharnobyl/stacked): free MIT native Swift session-only stack; Shift-Command-C collects copies and Command-V consumes them in order; manual mode works without Accessibility. Current documented binary is not notarized, making it behavior—not release—benchmark.
- [Clipbara](https://github.com/mobrava/Clipbara): free GPL-3.0 native Swift/AppKit/SwiftData manager; local storage, exclusions, configurable shortcuts, pinboards, keyboard flow, signed/notarized releases, and Sparkle-only network. Useful OSS release/privacy benchmark, not code-reuse source without GPL review.

Decision: borrow queue visibility, recovery, and exclusion patterns. Avoid persistent passive capture by default.

### Task products

- [Todoist](https://www.todoist.com/pricing/): free; Pro $5/user/month when billed annually. Global Quick Add and natural language. Its [shortcut documentation](https://www.todoist.com/help/articles/use-keyboard-shortcuts-in-todoist-Wyovn2) documents QWERTY/QWERTZ limitations, demonstrating keyboard-layout risk.
- [TickTick](https://www.ticktick.com/upgrade): free; Premium $49.99/year. Global add and clipboard recognition.
- [Superlist](https://www.superlist.com/pricing): free; Basic $5/person/month annual. Notes/tasks/collaboration, cloud-first.

Decision: use simple completion semantics only. No due dates, recurrence, calendar, teams, or full project management in v1.

### Knowledge and OCR products

- [Obsidian](https://obsidian.md/pricing): free core, local Markdown, no signup or telemetry; optional paid Sync. Official Web Clipper supports templates and Interpreter prompts.
- [Joplin](https://joplinapp.org/): free open source, local notes/to-dos and browser clipper, optional paid/self-hosted sync.
- [Pieces](https://pieces.app/pricing): paid local-first AI workflow memory across applications; Pro is $18.99/user/month monthly after card-required seven-day trial, with no free plan. Passive long-term observation contrasts with Bronze deliberate capture.
- [TextSniper](https://textsniper.app/): $7.99+ one-time; on-device OCR for non-selectable text.
- [ScreenFloat](https://www.eternalstorms.at/ScreenFloat/): $17.99 one-time; screenshots, OCR, floating text/images, Services, Shortcuts, AppleScript.
- [Shottr](https://shottr.cc/purchase.html): free with reminders or $12 Basic; local screenshot/OCR workflow.

Decision: OCR and attachments are P2. Text selection remains P0.

## Pricing and value signal

The market sets several reference bands:

| Band | Examples | Signal |
| --- | --- | --- |
| Free, OSS, or source-available | Cooper, PasteBar (source-available), Maccy, CopyQ, Batch Clipboard, FlowClip, Espanso, macOS built-ins | Users expect basic clipboard/capture infrastructure at no cost |
| $5–$13 one-time | Antinote $5, Jot $4.99, Scratchpad $8, TextSniper $7.99+ | Focused native utilities face strong low-price alternatives |
| $15–$25 one-time | PastePal $14.99, BetterTouchTool $15/$25, SideNotes $19.99, Unclutter $19.99 | Polished power utility range |
| $39–$50 one-time | Copper $39, Pastebot 3 direct $39, Promta $49.99 lifetime, Things Mac $49.99 | Requires differentiated workflow, brand, or depth |
| Subscription | Pastebot App Store, Raycast Pro, Drafts Pro, Paste, Steecky, Todoist, TickTick | Usually justified by updates, AI, sync, collaboration, or continuing service |

Bronze is intended as self-owned, local-first software with portable open formats. Distribution license remains a separate decision. Product quality must still meet paid-tool expectations. Avoid using “free” to excuse unreliable capture, unsigned builds, inaccessible controls, or weak recovery.

## Accessibility and internationalization gap

No reviewed product publishes a VPAT/ACR or an independent WCAG 2.2 AA/VoiceOver conformance report.

This is an absence in reviewed public evidence, not proof a product is inaccessible.

Best public benchmark:

- PastePal's Mac declaration lists VoiceOver, Voice Control, dark interface, and 36 languages. Its iPad declaration also lists Larger Text at 200%+, but that cannot be transferred to macOS evidence.

Other signals:

- SideNotes: nine languages.
- Unclutter: 11 languages.
- Tot: English-only despite accessibility fixes.
- Scratchpad: explicitly no localization plan.
- Cooper and Copper: no meaningful public accessibility or i18n documentation found.
- Todoist: global-shortcut behavior constrained to QWERTY/QWERTZ in published documentation.

Bronze differentiation requires evidence:

1. VoiceOver journeys for onboarding, permission recovery, capture, edit, reorder, copy, complete, skip, undo, archive, and settings.
2. Full Keyboard Access, Switch Control, and Voice Control verification.
3. Correct AX roles, names, values, states, relationships, focus restoration, and live announcements.
4. Text resize at 200%, zoom/reflow at 400%/320 CSS px, Increase Contrast, Differentiate Without Color, Reduce Motion, Reduce Transparency, and system accent testing.
5. No color-only state and minimum contrast in every theme/state.
6. RTL mirroring plus Arabic and Hebrew content.
7. CJK IME composition and candidate-window safety.
8. AZERTY, QWERTZ, Dvorak, Colemak, dead-key, external-keyboard, and remapped-modifier tests.
9. Localized shortcut display and semantic-versus-physical key modeling.
10. Automated webview checks plus native Accessibility Inspector and manual assistive-technology testing.

## Privacy gap

Clipboard managers normalize passive recording. Bronze should make deliberate capture the default and treat clipboard access as a temporary transport mechanism.

Required controls:

- Per-application capture exclusions.
- Concealed/password-field rejection.
- Configurable secret detection for API keys, tokens, recovery codes, and private keys.
- Local-only default with no account.
- Optional network-free build/runtime mode.
- Network activity ledger showing exact feature, destination, and purpose.
- AX-first capture and manual clipboard fallback; experimental synthetic copy uses stable generation checks and no automatic restoration.
- Configurable retention, explicit app-controlled purge, trash, and undo, with backup-survival disclosure.
- Restricted local database permissions. App-level encryption/keychain lock needs a separate threat model and is not P0.
- Export before deletion or destructive migration.
- No telemetry by default.

## Reliability lessons

Double-Shift is a gesture, not a dependable sole interface. It may conflict with normal typing, Sticky Keys, Slow Keys, input methods, remappers, games, remote desktops, or application-specific event handling.

Bronze capture fallback chain:

1. Configurable conventional global chord registered through native macOS APIs.
2. Optional double-tap modifier gesture.
3. Menu-bar command that captures current selection.
4. Explicit copy-current-clipboard/manual composer route when selection cannot be read.
5. Future P1 macOS Service/Quick Action operating on selected text.
6. Future P1 URL/CLI integration for external tools.

Health diagnostics must verify actual behavior:

- Accessibility authorization.
- Input Monitoring when genuinely needed.
- Bundle path, signing identity, notarization, and update identity.
- Global shortcut registration and collisions.
- Event-tap liveness and timeout recovery.
- Selection acquisition strategy used.
- Clipboard generation/stable-read outcome; P0 has no automatic restoration.
- Current application exclusion.
- Secure Input or unsupported-context detection.

## Market whitespace

### Semantic queue

Clipboard history answers “what did I copy?” Bronze answers “what work remains from this context?”

Recommended item lifecycle:

- Queued
- Copied
- Active
- Done
- Skipped
- Trashed; archive applies to sections, not item lifecycle.

Copying must not silently equal completion. “Copy and advance” can be configurable.

### Context provenance

Capture optional, privacy-filtered metadata:

- Source application bundle identifier and display name.
- Window/document title when available.
- URL in supported browsers/editors.
- Capture timestamp.
- Capture strategy and confidence.

Users must be able to disable each provenance field.

### Context bundles

Allow one source capture to own multiple follow-up prompts and notes. Support drag reorder, multi-select, copying as plain text/Markdown/prompt block, and completion at child or bundle level.

### Open interoperability

Use local SQLite for transactions and search, plus deterministic Markdown/JSON export, documented schema versioning, migrations, and backup. Add URL scheme, CLI, macOS Services, Shortcuts, and App Intents in P1 after core permission and capability boundaries are stable.

Later adapters:

- PopClip
- Raycast
- Alfred
- BetterTouchTool
- Keyboard Maestro
- MCP
- Obsidian/Drafts export

## Product priorities

This market-derived list is aligned to the normative [PRD](03-prd.md). PRD and traceability matrix override competitor-research recommendations if later edits diverge.

### P0: trustworthy core

1. Native reliable selection acquisition with explicit fallback chain.
2. Configurable global shortcut plus menu-bar/manual alternatives; Services remain P1.
3. Ordered sections/queue with add, edit, reorder, copy, done, skip, undo, archive, trash, and search.
4. Captured context and manually authored prompts/notes coexist in queue; nested context bundles remain P1.
5. Local crash-safe SQLite persistence.
6. Deterministic Markdown/JSON export and backup.
7. Permission onboarding, capture health check, and recovery.
8. Clipboard generation/race handling with no sentinel or automatic restoration.
9. Sensitive-application exclusions, protected-content rejection, and opt-in provenance.
10. VoiceOver, keyboard-only, high-contrast, reduced-motion, RTL, IME foundations.
11. Configurable settings for shortcut, gesture, window placement, appearance, sounds, animation, capture behavior, copy behavior, and launch behavior.
12. Signed, notarized, stable-bundle distribution and update strategy.

### P1: professional workflow

1. Nested context bundles/projects and richer provenance/reopen behavior.
2. Merge/split items and reusable output templates.
3. Filters and advanced retention rules.
4. Duplicate warning/merge without persisting content hashes in diagnostics.
5. URL scheme, CLI, Services, Shortcuts, and App Intents.
6. PopClip, Raycast, Alfred, BetterTouchTool, and Keyboard Maestro examples.
7. Cooper/PasteBar/Markdown importers where license and schema permit.
8. Privacy dashboard and network ledger if any future network feature exists.

### P2: expansion

1. Images/files and on-device OCR.
2. Reusable prompt templates and variables.
3. Optional encrypted iCloud sync.
4. Sandboxed extension system with per-extension capabilities.
5. Apple Intelligence, local Ollama, and BYO cloud provider actions.
6. MCP adapter.
7. Mobile companion.
8. Windows and Linux after macOS reliability targets are met.

### Avoid in v1

- Passive unlimited clipboard history.
- Full task management: dates, recurrence, calendar, collaboration.
- Full PKM/notebook graph.
- Mandatory account, cloud sync, or hosted backend.
- Built-in hosted AI or subscription dependency.
- Automatic capture of every selection.
- Cross-platform input abstraction that weakens native macOS reliability.
- Unbounded plugin execution.
- Rich document editor.
- Analytics enabled by default.

## Benchmark matrix

| Bronze subsystem | Primary benchmark | Secondary benchmark | What to copy conceptually | What to avoid |
| --- | --- | --- | --- | --- |
| Selected-text capture | Marker, ActionClip | PopClip, OnText, Pluks | AX provider evidence, permission explanation, contextual invocation, explicit hotkey | Ambient selection recording or risky clipboard restoration |
| Transient scratchpad | Antinote | Scratchpad, Tot | Instant global access, disposability, export, recovery | Permanent-note complexity |
| Edge/panel UI | SideNotes | Unclutter | Fast reveal, remembered placement, full-screen behavior | Hidden controls and focus loss |
| Queue mechanics | Pastebot, Paste Stack, CleanClip | Stacked, PastePal, Batch Clipboard, FlowClip | Visible order, next item, sequential recovery | Silent consumption on failure |
| Task state | Things Quick Entry | Todoist | Clear completion, undo, low-friction entry | Dates/projects/recurrence scope |
| Local storage/privacy | Antinote | Marker, Clipbara, Maccy, PastePal | Local default, no analytics, explicit exclusions | Passive sensitive capture |
| Extensibility | PopClip | Raycast, Alfred, BetterTouchTool | URL/Shortcuts/CLI/App Intent surfaces | Arbitrary unpermissioned execution |
| Configuration | BetterTouchTool | Raycast | Shortcut recording, collision handling, per-app rules | Power-user complexity in core flow |
| A11y/i18n | PastePal | Apple native applications | VoiceOver, Voice Control, broad locales; independently test Bronze text scaling | Marketing claims without evidence |
| Tauri structure | PasteBar | Cooper | Rust boundary, local database, compact distribution | License contamination or webview-only AX assumptions |
| Capture failure model | Historical Cooper `v0.2.1` issue #3 | Current Cooper audit, Todoist layout limitation | Native APIs, fallbacks, diagnostics, layout tests | Raw cross-platform listener as sole path |
| OS integration | Apple Quick Note/Spotlight | Things Autofill | Services, App Intents, source context | Rebuilding OS commodity features |

## Competitive acceptance criteria

Bronze v1 should be considered differentiated only when:

- Capture works across agreed browser, native, Electron, IDE, terminal, and document-editor matrix.
- AX/manual paths never mutate clipboard; experimental synthetic fallback explicitly mutates shared pasteboard, makes no restoration guarantee, and never silently loses observed request.
- Every primary operation works by keyboard and VoiceOver.
- Double-Shift can be disabled and replaced.
- Shortcut conflicts and unsupported keyboard layouts produce actionable feedback.
- App works fully offline with blocked network.
- User can inspect, export, back up, restore, and delete all data.
- Sensitive applications can be excluded before first capture.
- Queue state remains consistent after crash/restart.
- Copy, copy-and-advance, complete, skip, and undo are distinct and configurable.
- RTL, CJK IME, long localized strings, 200% text resize, 400%/320 CSS px reflow, high contrast, Differentiate Without Color, reduced motion, and reduced transparency pass release tests.
- Signed updates retain permissions where macOS permits or provide accurate recovery when trust resets.
- No feature requires a Bronze account.

## Final recommendation

Build narrow intersection competitors leave open:

> reliable selection capture + transient context bundles + explicit work state + copy-back + local ownership + evidence-backed accessibility.

Antinote, SideNotes, ActionClip, Marker, Pastebot, Paste Stack, CleanClip, and Things provide strongest behavioral benchmarks. PastePal provides strongest public Mac i18n/accessibility declaration in reviewed set, without independent conformance proof. Clipbara provides useful native OSS signing/privacy benchmark. PasteBar provides broadest comparable Tauri source reference but is under noncommercial terms; Pluks provides a vendor-visible Tauri comparison without reviewed source. Cooper demonstrates desired loop; historical `v0.2.1` and current `v0.3.5` expose distinct reliability failures to avoid. macOS 26 Spotlight makes generic clipboard-history work poor investment.
