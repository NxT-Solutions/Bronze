# Competitor source register

Research date: 2026-08-27  
Purpose: primary-source register supporting Bronze competitive analysis and product decisions  
Scope: macOS selection capture, transient queues, clipboard workflows, scratchpads, prompt libraries, task quick entry, automation, privacy, accessibility, and internationalization

## How to use this register

- Product facts should cite official product, documentation, pricing, App Store, or source repository pages.
- “Not found” means reviewed official materials did not publish the claim. It does not prove a product lacks the behavior.
- App Store accessibility entries are developer declarations, not independent audits.
- Evidence grade follows [methodology](methodology.md): normative technical/repository source = A; first-party product, pricing, privacy, or App Store claim = B; issue/community report = C; analyst inference = D and must be labeled. Composite claims inherit weakest supporting grade.
- Prices are snapshots. Recheck before purchasing, publishing, or using them in marketing.
- Promotional and dynamic storefront prices carry high volatility.
- GitHub stars, forks, issue counts, release dates, and current maintainership are volatile and should be rechecked if used.
- Reddit is useful for user-reported pain and discovery, not authoritative product specification.

## Verification hierarchy

1. Official product documentation or support.
2. Official pricing/purchase page.
3. Official App Store listing.
4. Maintainer-owned source repository and issue tracker.
5. Vendor privacy policy.
6. Community discussion, only when labeled anecdotal.

## Core reference and OSS clone

| ID | Product/topic | Primary source | Evidence captured | Volatility/notes |
| --- | --- | --- | --- | --- |
| C-001 | Copper | [Official product page](https://shadcn.com/copper) | $39 one-time; macOS 14+; double-Shift selected-text capture; prompts/notes; copy-back to ChatGPT, Claude, Cursor; check-off; local file; no account/sync/collection; Accessibility permission | Price medium; features medium |
| C-002 | Cooper | [TouchMyBar/cooper](https://github.com/TouchMyBar/cooper) | Apache-2.0; Tauri 2/Rust/web UI; SQLite; macOS/Windows/Linux; double left/right Shift; fallback shortcuts; sections, attachments, themes, Markdown export; no telemetry/sync/account | Repository state high |
| C-003 | Cooper reliability | [Issue #3](https://github.com/TouchMyBar/cooper/issues/3) and [audited `v0.3.5`](https://github.com/TouchMyBar/cooper/releases/tag/v0.3.5) | Historical `v0.2.1` Apple Silicon SIGTRAP: raw `rdev` listener reached HIToolbox dispatch assertion despite permission grants. macOS `v0.3.0` removed `rdev`; `v0.3.5` uses a custom CGEventTap with separate audited risks. | Issue remains open; implementation version critical |
| C-004 | Cooper community context | [User-provided Reddit discussion](https://www.reddit.com/r/ClaudeAI/comments/1ve1dgb/i_recreated_shadcns_copper_as_a_free_opensource/) | Discovery and anecdotal user discussion around clone; not used as authoritative feature or reliability specification | Community source; high |

## Direct scratchpad and quick-capture products

| ID | Product/topic | Primary source | Evidence captured | Volatility/notes |
| --- | --- | --- | --- | --- |
| S-001 | Antinote | [Feature and comparison page](https://antinote.io/features) | $5 once; macOS 14+; local SQLite; global hotkey; checklists; AutoPaste; on-device OCR; export; offline; no account/analytics; optional encrypted iCloud | Price medium; page unusually detailed |
| S-002 | Antinote 2.0 | [Release page](https://antinote.io/updates/antinote-2-0) | Optional encrypted iCloud, more than 100 commands, signed extension browser, JavaScript extensions, Keychain-held API keys, per-endpoint privacy settings | Features medium |
| S-003 | Antinote manual | [User manual](https://antinote.io/user-manual) | Slotted/ephemeral notes, OCR behavior, URL/export, privacy controls, updates, backups | Low-medium |
| S-004 | SideNotes | [Official product page](https://www.apptorium.com/sidenotes) | $19.99, trial, screen-edge access, hot side, notes/tasks/snippets, Markdown, checklists, files/images, search, shortcuts, integrations, optional iCloud, backups, privacy statements | Price medium |
| S-005 | SideNotes App Store | [Mac App Store listing](https://apps.apple.com/us/app/sidenotes-screen-edge-notes/id1441958036?mt=12) | $19.99; English plus eight languages; current feature/change inventory; one-time purchase for current major | Price/locales medium |
| S-006 | Steecky | [Official site](https://steecky.app/) | Menu-bar notes, tasks, clipboard, snippets, links, trash; local-only/offline; app exclusions; free tier; Pro $3.99/month or $39.99/year; macOS 14+ | Price high |
| S-007 | Drafts introduction | [Getting started](https://docs.getdrafts.com/gettingstarted/) | Capture-first text workflow and action model across Apple platforms | Low |
| S-008 | Drafts capture | [macOS Capture Window](https://docs.getdrafts.com/docs/extensions/capture-window) | Global Shift-Command-2 capture; new, append, prepend behavior; Pro gating for some operations | Low-medium |
| S-009 | Drafts pricing | [Drafts Pro](https://docs.getdrafts.com/draftspro) | Core free; Pro $1.99/month or $19.99/year; feature split | Price high |
| S-010 | Scratchpad | [Official product/support page](https://sindresorhus.com/scratchpad) | $8; one plain-text note; custom hotkey; Services, Share, Shortcuts, URL scheme, PopClip integration; optional iCloud; local backups; no tracking; no localization planned | Price medium; explicit limitations useful |
| S-011 | Tot | [Official product page](https://tot.rocks/) | Seven pads; rich/plain/Markdown; checklists; menu bar; Shortcuts; Apple-device sync | Features medium |
| S-012 | Tot App Store | [Tot Pocket listing](https://apps.apple.com/us/app/tot-pocket/id1498235191) | English-only; $19.99 mobile unlock; cross-device description and version history | Price/locales medium |
| S-013 | Jot local capture | [Official product page](https://jot.arunbrahma.com/) | $4.99 one-time; macOS 14+; Option-Space capture; searchable local log; no account/sync; Markdown export | Price medium |
| S-014 | Jot menu-bar scratchpad | [getjot.app](https://getjot.app/) | Separate similarly named open-source menu-bar scratchpad; local notes; low-cost storefront | Distinguish from S-013 |
| S-015 | Apple Quick Note | [Apple Support](https://support.apple.com/guide/notes/create-a-quick-note-apdf028f7034/mac) | Globe/Fn-Q and hot-corner invocation; Safari selected-text/highlight context | OS feature low |

## Clipboard managers and queues

| ID | Product/topic | Primary source | Evidence captured | Volatility/notes |
| --- | --- | --- | --- | --- |
| P-001 | PasteBar | [Official product site](https://www.pastebar.app/) | Free, local-first, source-available clipboard manager and custom clips/boards; not OSI open source | Features medium |
| P-002 | PasteBar source/license | [PasteBarApp repository](https://github.com/PasteBar/PasteBarApp), [license at reviewed revision](https://github.com/PasteBar/PasteBarApp/blob/c89d054d29a0b51fdd3a031757a8dd21a09bc241/CC-LICENSE), and [Open Source Definition](https://opensource.org/osd) | Tauri/Rust/React/Vite/Tailwind; local data; notes/Markdown/code; forms/templates; backups; languages; CC BY-NC plus custom limited-commercial exception; noncommercial restriction is not OSI open source | Repository/license medium; legal review required |
| P-003 | Pastebot | [Official product page](https://tapbots.com/pastebot) | Pastebot 3 clipboard history, Pastebins, Stacks, filters, Quick Paste, shortcuts, blacklist, optional iCloud; older Pastebot 2 scratchpad/sequential wording is historical | Features low-medium; version critical |
| P-004 | Pastebot pricing | [Official purchase page](https://tapbots.com/pastebot/buy/) | macOS 26+; $39 direct with one year updates, later update year $19; App Store $2.99/month or $24.99/year; $12.99 is Pastebot 2 upgrade discount | Price/platform high |
| P-005 | PastePal | [Official product page](https://indiegoodies.com/pastepal) | One-time purchase model; Paste Stack, Quick Mode, collections, side bar, snippets, local privacy, optional iCloud/local sharing | Features/pricing medium |
| P-006 | PastePal Mac App Store | [Mac listing](https://apps.apple.com/us/app/clipboard-manager-pastepal/id1503446680?platform=mac) | $14.99 Pro IAP; Mac/iPhone/iPad/Vision; English plus 35 languages; Mac declares VoiceOver, Voice Control, and dark interface | Price/locales high; self-declared; 200% Larger Text is not listed for Mac |
| P-007 | Maccy | [Official site](https://maccy.app/) | Free MIT open source; native macOS; local, lightweight, keyboard-first clipboard history | Use only official domain |
| P-008 | Maccy source | [p0deje/Maccy](https://github.com/p0deje/Maccy) | MIT license, source architecture, releases, issue history | Repository high |
| P-009 | Paste | [Official pricing](https://pasteapp.io/pricing) | $2.49/month or $29.99/year personal; trial; Mac/iPhone/iPad; history, pinboards, sync | Price high; lifetime amount dynamic/not recorded |
| P-010 | Unclutter | [Official site](https://unclutterapp.com/) | $19.99 one-time; clipboard, file drop zone, sticky notes; top-screen gesture; optional cloud; 11 languages | Price medium |
| P-011 | CopyQ documentation | [Official documentation](https://copyq.readthedocs.io/en/stable/) | Free cross-platform programmable clipboard history, search, edit, tags/tabs, shortcuts, exclusions, scripting; macOS troubleshooting | Features low-medium |
| P-012 | CopyQ source | [hluk/CopyQ](https://github.com/hluk/CopyQ) | GPL source, releases, platform implementation | Repository high |
| P-013 | Batch Clipboard | [Official repository](https://github.com/jpmhouston/Batch-Clipboard) | Free MIT; FIFO batch copy/paste; Control-Command-C/V; Intel/Apple Silicon; macOS 10.15+ intent; English-only note; source/build details | Repository high |
| P-014 | FlowClip | [Official repository](https://github.com/gityeop/FlowClip) | Maccy fork; visual queue; copy batch; FIFO paste; paste all; custom separators; Accessibility requirement | Repository high |
| P-015 | Copy clipboard shelf | [Official repository](https://github.com/tarikbc/Copy) | Free/open macOS shelf; rich editor; pinboards; numbered paste stack; local/no analytics; signed update process | Repository high |
| P-016 | Pasta | [Official site](https://www.pasta-app.com/) | Local-first history, global search, snippets/placeholders, command mode, pinned items, plain paste | New product; features high |
| P-017 | macOS 26 clipboard history | [Apple Support](https://support.apple.com/en-euro/guide/mac-help/mchl40d5b86b/26/mac/26) | Command-Space then Command-4; search/copy/clear history; opt-in enablement; sensitive-data warning | OS feature low |
| P-018 | macOS Tahoe announcement | [Apple newsroom](https://www.apple.com/uk/newsroom/2025/06/macos-tahoe-26-makes-the-mac-more-capable-productive-and-intelligent-than-ever/) | Spotlight browse view includes clipboard history; App Intents and action expansion | OS feature low |
| P-019 | Universal Clipboard | [Apple Support](https://support.apple.com/en-us/102430) | Cross-device copy/paste prerequisites and behavior | OS feature low |
| P-020 | Paste Stack | [Official help](https://pasteapp.io/help/using-paste-stack) | Shift-Command-C starts temporary ordered collection; Command-V pastes/consumes top-to-bottom; direction can reverse; used items disappear | Features medium |
| P-021 | CleanClip | [Official site](https://cleanclip.cc/) | Caret-adjacent quick menu; Paste Stack sequential Command-V; Split Copy; form-filling use; broad localization | Features/pricing high |
| P-022 | Stacked | [MIT repository](https://github.com/sharnobyl/stacked) | Native session-only ordered paste stack; Shift-Command-C collection; Command-V consumption; manual no-Accessibility mode; memory-only/offline; current binary documented as not notarized | New/immature repository high |
| P-023 | Clipbara | [GPL-3.0 repository](https://github.com/mobrava/Clipbara) | Native Swift/SwiftData manager; local storage; exclusions; configurable shortcuts; pinboards; signed/notarized release claim; Sparkle update network only | Repository/release high |

## Selected-text action tools

| ID | Product/topic | Primary source | Evidence captured | Volatility/notes |
| --- | --- | --- | --- | --- |
| A-001 | ActionClip | [Official product page](https://actionclip.app/) | Native macOS 15+; selected-text widget; smart suggestions; broad built-in actions; Apple Intelligence, Ollama, BYO external AI; free usage allowance; early-bird/lifetime offers; no account | Promotion/price very high |
| A-002 | ActionClip actions | [Action inventory](https://actionclip.app/actions) | Clipboard, formatting, language, search, conversion, notes, Apple-app actions; local-versus-online availability | Features high |
| A-003 | ActionClip privacy | [Privacy policy](https://actionclip.app/privacy) | Local processing categories; external-provider behavior; what settings/history are not synced; telemetry limits | Policy medium |
| A-004 | ActionClip changelog | [Changelog](https://actionclip.app/changelog) | Current release behavior and integration fixes | High |
| A-005 | PopClip | [Official product page](https://www.popclip.app/) | Automatic selected-text action bar; current macOS utility positioning; trial | Features medium |
| A-006 | PopClip pricing | [Official purchase page](https://www.popclip.app/buy) | Standard/Lifetime license structure; dynamic pricing was not reliably extractable | Price very high; recheck manually |
| A-007 | PopClip developer platform | [Developer documentation](https://www.popclip.app/dev/) | URL, Service, Shortcut, key press, JavaScript/TypeScript, shell, AppleScript actions; selection filters; extension signing/security | Low-medium |
| A-008 | PopClip complete docs | [One-page developer docs](https://www.popclip.app/dev/all) | Input/output contract, URL templates, executable extension warnings, signed/unsigned behavior | Low-medium |
| A-009 | OnText | [Official site](https://gityeop.github.io/OnText/) | Native selected-text action utility; download and product positioning | Features high |
| A-010 | OnText workflow | [PopClip alternative guide](https://gityeop.github.io/OnText/docs/guides/popclip-alternative) | Hotkey-first selection flow; default F2; action keys; custom URL/script/Shortcut/deep-link actions | Features high |
| A-011 | OnText distribution | [Maintainer Gumroad](https://gityeop.gumroad.com/l/ontext) | Name-a-fair-price; Intel/Apple Silicon; cloud/local AI capability summary | Price high |
| A-012 | Marker | [Official site](https://getmarkerapp.net/) and [MIT repository](https://github.com/Mazide/marker) | Free macOS 14+ AX-observed selected-text buffer/history; search, exclusions, CLI, App Intents, URL scheme, local SQLite; fallback/paste paths temporarily mutate/restore clipboard | New repository/features high; restoration requires independent race review |
| A-013 | Pluks | [Official site](https://pluks.app/) and [release repository](https://github.com/darth-pixit/pluks-releases) | Free automatic select-to-copy; Tauri/Rust vendor claim; 200-entry local history; Accessibility/Input Monitoring; secure-field checks; opt-out anonymous telemetry/crash reports; public repo exposes releases, not reviewed source | Product claims/release state high |

## Prompt and snippet libraries

| ID | Product/topic | Primary source | Evidence captured | Volatility/notes |
| --- | --- | --- | --- | --- |
| L-001 | Promta | [Official site](https://promta.app/) | 25-prompt free tier; five AI improvements/month; $4.99/month, $24.99/year, $49.99 lifetime; menu bar/hotkey; variables, version history, iCloud, MCP; macOS 14+ Apple Silicon | Price/limits high |
| L-002 | Prompt Box | [Official site](https://promptboxapp.com/) | Unlimited local prompts free; browser text expansion; tags/search/favorites; selected-text save; CSV import/export; no account/server/tracking on local tier; cloud Pro prices | Price high |
| L-003 | PromptBox.ai | [Application](https://app.promptbox.ai/) | Different product from Prompt Box; system-wide expansion, nested folders/team sharing; macOS/iOS/Windows; observed $5.99/month or $79 lifetime | Price high; name collision |
| L-004 | Snippety | [Official site](https://snippety.app/) | Global Command-Shift-Space access, snippets, iCloud through user account, shell/CLI/Shortcuts, AI provider configuration | Features medium |
| L-005 | Snippety pricing | [Official pricing](https://snippety.app/pricing) | Free start; $29.99 standalone Mac lifetime; $39.99 all Apple devices; $14.99 mobile; optional $2.99/month; feature/license split | Price high |
| L-006 | Snippety Quick Access | [Official guide](https://snippety.app/help/macos/using-snippets/quick-access-menu/) | Configurable hotkey, context menu, first-nine numeric shortcuts, operations on selected text | Low-medium |
| L-007 | Espanso | [Official site](https://espanso.org/) | Free privacy-first cross-platform text expansion, packages, forms, scripts, file configuration | Features medium |
| L-008 | Espanso source | [espanso/espanso](https://github.com/espanso/espanso) | GPL-3.0; Rust; local/no tracking; macOS/Windows/Linux; regex, scripts, application configuration | Repository high |

## Automation and launcher platforms

| ID | Product/topic | Primary source | Evidence captured | Volatility/notes |
| --- | --- | --- | --- | --- |
| U-001 | Raycast pricing | [Official pricing](https://www.raycast.com/pricing) | Free core; Pro $10/month or $8/month annual; clipboard-retention and sync/AI tier split | Price high |
| U-002 | Raycast billing | [Official manual](https://manual.raycast.com/billing) | Core versus Pro entitlement details, AI trial/BYOK/local model context | High |
| U-003 | Raycast Clipboard | [Core feature page](https://www.raycast.com/core-features/clipboard-history) | Text/images/files/links/colors, pins, OCR, exclusions, AI action, keyboard flow | Features medium |
| U-004 | Raycast Clipboard manual | [Manual](https://manual.raycast.com/clipboard-history) | Retention tiers, local encrypted storage, exclusions, interaction details | Features/tiers high |
| U-005 | Raycast export | [Import/export manual](https://manual.raycast.com/import-export) | Encrypted configuration export including clipboard, notes, snippets, settings, hotkeys | Low-medium |
| U-006 | Alfred Powerpack | [Official pricing/features](https://www.alfredapp.com/powerpack/) | £34 single-user current major; £59 Mega Supporter lifetime upgrades; workflows, clipboard, snippets | Price high |
| U-007 | Alfred Clipboard | [Official help](https://www.alfredapp.com/help/features/clipboard/) | History, snippets, merge via Command-C-C, Accessibility permission | Low-medium |
| U-008 | BetterTouchTool pricing | [Official purchase page](https://folivora.ai/buy/) | $15 Standard with two years updates; $25 lifetime; personal multi-Mac terms | Price high |
| U-009 | BetterTouchTool overview | [Official site](https://folivora.ai/) | 45-day trial, no account, broad macOS automation | Medium |
| U-010 | BetterTouchTool Clipboard | [Official documentation](https://docs.folivora.ai/docs/actions/clipboard-manager/) | Local clipboard manager, search, favorites, editing, transforms; no cloud | Low-medium |
| U-011 | BetterTouchTool docs | [Documentation root](https://docs.folivora.ai/) | Global/application triggers, selected-text changed trigger, scripts, menus, webviews | Medium |
| U-012 | BetterTouchTool launcher | [Launcher docs](https://docs.folivora.ai/docs/launcher/) | Launcher, clipboard, reminders, dictation, AI and customizable instances | Medium |
| U-013 | Keyboard Maestro | [Official site](https://www.keyboardmaestro.com/main/) | $36 current major; $25 upgrade; global macros, clipboard history/named clipboards, scripts; trial and system requirements | Price/version high |

## Task quick-entry products

| ID | Product/topic | Primary source | Evidence captured | Volatility/notes |
| --- | --- | --- | --- | --- |
| T-001 | Things | [Official product page](https://culturedcode.com/things/) | Mac $49.99; iPhone/iPad separate; native task workflow | Price high |
| T-002 | Things Quick Entry | [Official support](https://culturedcode.com/things/support/articles/2249437/) | Control-Space Quick Entry; Control-Option-Space Autofill; configurable shortcuts; source-link/context integration and helper | Low-medium |
| T-003 | Things pricing/sync | [Official support](https://culturedcode.com/things/support/articles/2803552/) | One-time app purchases, free optional sync, Apple-only, no shared lists | Low-medium |
| T-004 | Things shortcuts/clipboard | [Official support](https://culturedcode.com/things/support/articles/2785159/) | Keyboard shortcuts and clipboard rows to multiple to-dos | Low-medium |
| T-005 | Todoist pricing | [Official pricing](https://www.todoist.com/pricing/) | Free tier; Pro $5/user/month annual equivalent; Business pricing and Quick Add tier context | Price high |
| T-006 | Todoist shortcuts | [Official support](https://www.todoist.com/help/articles/use-keyboard-shortcuts-in-todoist-Wyovn2) | Global Quick Add, configurable shortcut, QWERTY/QWERTZ limitation statement | Features medium |
| T-007 | TickTick pricing | [Official upgrade page](https://www.ticktick.com/upgrade) | Free/Premium split; $49.99/year observed | Price high |
| T-008 | TickTick global add | [Official help](https://help.ticktick.com/articles/7055782422935240704) | Global add on Mac/Windows and clipboard-recognition behavior | Medium |
| T-009 | Superlist pricing | [Official pricing](https://www.superlist.com/pricing) | Free tier; Basic $5/person/month annual; Super $21/person/month annual; tasks/notes/platform availability | Price high |

## Knowledge, memory, and OCR adjuncts

| ID | Product/topic | Primary source | Evidence captured | Volatility/notes |
| --- | --- | --- | --- | --- |
| K-001 | Obsidian pricing | [Official pricing](https://obsidian.md/pricing) | Core free, local, no signup/telemetry; Sync $4/month annual or $5 monthly; optional commercial support | Price high |
| K-002 | Obsidian Web Clipper | [Official help](https://obsidian.md/help/web-clipper) | Official browser extensions, highlights, templates/variables, Interpreter natural-language prompts | Medium |
| K-003 | Joplin | [Official site](https://joplinapp.org/) | Free open source, local notes/to-dos, plugins, web clipper, cross-platform | Medium |
| K-004 | Joplin Cloud | [Official plans](https://joplinapp.org/plans/) | Paid hosted sync plans and self-host option | Price high |
| K-005 | Pieces | [Official site](https://pieces.app/) | Cross-application workflow memory with on-device saved context and AI-tool integrations | Features high |
| K-006 | Pieces pricing | [Official pricing](https://pieces.app/pricing) | No free plan; card-required seven-day trial; Pro $18.99/user/month on monthly billing; Enterprise also offered | Price very high |
| K-007 | TextSniper | [Official site](https://textsniper.app/) | $7.99 single Mac, $9.99 three Macs, $11.99 unlimited/App Store observed; local OCR; no internet required | Price high |
| K-008 | ScreenFloat | [Official site](https://www.eternalstorms.at/ScreenFloat/) | $17.99/€19.99/£17.99; screenshot/OCR, floating text/images, notes/tags, Services/Shortcuts/AppleScript, optional iCloud; four languages | Price high |
| K-009 | Shottr pricing | [Official purchase page](https://shottr.cc/purchase.html) | Free indefinitely with reminders; $12 Basic; $30 Friends Club | Price high |
| K-010 | Shottr privacy | [Official privacy explanation](https://shottr.cc/blog/privacy) | Local screenshots/OCR, Accessibility use, mostly-offline exceptions for licensing/update/upload | Policy medium |

## Accessibility and internationalization evidence

| ID | Product/topic | Primary source | Evidence captured | Interpretation |
| --- | --- | --- | --- | --- |
| X-001 | PastePal accessibility | [Mac listing](https://apps.apple.com/us/app/clipboard-manager-pastepal/id1503446680?platform=mac) and [iPad listing](https://apps.apple.com/us/app/clipboard-manager-pastepal/id1503446680?platform=ipad) | Mac declares VoiceOver, Voice Control, dark interface, and 36 languages. iPad additionally declares Larger Text 200%+; that evidence is platform-specific. | Strong public declaration, not independent audit; do not transfer iPad claim to macOS |
| X-002 | SideNotes localization | [Mac App Store listing](https://apps.apple.com/us/app/sidenotes-screen-edge-notes/id1441958036?mt=12) | English plus eight languages; change history includes non-Latin performance fixes | Good localization signal; no formal conformance report found |
| X-003 | Scratchpad localization | [Official FAQ](https://sindresorhus.com/scratchpad) | Developer states no localization plan | Direct evidence of market gap |
| X-004 | Tot localization | [App Store listing](https://apps.apple.com/us/app/tot-pocket/id1498235191) | English-only listing; version history provides scattered accessibility fixes | Partial evidence only |
| X-005 | Todoist layout limitation | [Official shortcut support](https://www.todoist.com/help/articles/use-keyboard-shortcuts-in-todoist-Wyovn2) | Shortcut support caveat for QWERTY/QWERTZ layouts | Direct input-layout risk benchmark |
| X-006 | Copper | [Official product page](https://shadcn.com/copper) | Accessibility permission requirement only | No published VoiceOver/WCAG/localization evidence found during review |
| X-007 | Cooper | [Official repository](https://github.com/TouchMyBar/cooper) | Themes and shortcuts documented | No published formal A11y/i18n evidence found during review |

## Privacy and offline evidence

| ID | Product/topic | Primary source | Evidence captured | Bronze relevance |
| --- | --- | --- | --- | --- |
| R-001 | Copper | [Official page](https://shadcn.com/copper) | Local file, no account/sync/collection | Baseline privacy promise |
| R-002 | Cooper | [Repository](https://github.com/TouchMyBar/cooper) | Local SQLite, no telemetry/account/sync | Open baseline |
| R-003 | Antinote | [Feature page](https://antinote.io/features) | Offline, local SQLite, no analytics, optional encrypted iCloud, explicit exceptions | Best privacy disclosure benchmark |
| R-004 | Steecky | [Official site](https://steecky.app/) | Local-only, no cloud/account, sandbox, per-app exclusion, export | Clipboard safety benchmark |
| R-005 | PastePal | [Official page](https://indiegoodies.com/pastepal) | Local/no tracking, optional iCloud/local peer sharing, monitoring control | User-control benchmark |
| R-006 | ActionClip | [Privacy policy](https://actionclip.app/privacy) | Separates local actions from external-provider transmission; documents data excluded from sync | Action-level network disclosure benchmark |
| R-007 | BetterTouchTool | [Clipboard documentation](https://docs.folivora.ai/docs/actions/clipboard-manager/) | Clipboard stored only locally; no cloud | Local automation benchmark |
| R-008 | Obsidian | [Pricing/privacy claims](https://obsidian.md/pricing) | Local files, no signup, no telemetry | Durable-data ownership benchmark |
| R-009 | Shottr | [Privacy article](https://shottr.cc/blog/privacy) | Enumerates local processing and network exceptions | Clear privacy explanation benchmark |
| R-010 | macOS Clipboard | [Apple Support](https://support.apple.com/en-euro/guide/mac-help/mchl40d5b86b/26/mac/26) | Opt-in enablement and sensitive-content warning | Onboarding warning benchmark |

## Price snapshot

| Product | Price observed 2026-08-27 | Recheck priority |
| --- | ---: | --- |
| Copper | $39 one-time | Medium |
| Marker | Free | Medium |
| Pluks | Free | High |
| Stacked | Free | Medium |
| Clipbara | Free | Medium |
| Antinote | $5 one-time | Medium |
| Jot | $4.99 one-time | Medium |
| Scratchpad | $8 universal purchase | Medium |
| Pastebot 3 | $39 direct with one year updates; $19 later update year; App Store $2.99/month or $24.99/year | High |
| PastePal | $14.99 one-time | High |
| BetterTouchTool | $15 Standard / $25 Lifetime | High |
| SideNotes | $19.99 current-major one-time | High |
| Unclutter | $19.99 one-time | High |
| Drafts Pro | $1.99/month / $19.99/year | High |
| Prompt Box Pro | $2.99/month / $19/year / $39 lifetime offer | Very high |
| Paste | $2.49/month / $29.99/year | High |
| Snippety | $29.99 Mac / $39.99 all Apple devices | High |
| Keyboard Maestro | $36 current major | High |
| Steecky Pro | $3.99/month / $39.99/year | Very high |
| Promta Pro | $4.99/month / $24.99/year / $49.99 lifetime | Very high |
| Things Mac | $49.99 one-time | High |
| TickTick Premium | $49.99/year | High |
| Raycast Pro | $10/month / $8/month annual | High |
| ActionClip | Free 30/day; $8 early-bird against $19 advertised list | Immediate; promotion |
| Pieces Pro | $18.99/user/month monthly after seven-day trial; no free plan | High |
| Alfred Powerpack | £34 current major / £59 lifetime upgrades | High |
| TextSniper | $7.99+ one-time | High |
| ScreenFloat | $17.99/€19.99/£17.99 | High |
| Shottr | Free with reminders / $12 Basic / $30 Friends Club | High |

## Sources requiring caution

### Dynamic prices

PopClip and some App Store storefronts render regional/dynamic values. Do not infer a number from cached community posts. Open purchase page in target region immediately before citing.

### Naming collisions

- Cooper is OSS clone of Copper.
- Prompt Box at promptboxapp.com and PromptBox.ai are different products.
- Jot names refer to several unrelated quick-note applications.
- Paste is used by multiple unrelated repositories and commercial products.

### License risk

- Cooper: Apache-2.0, comparatively permissive.
- Maccy and Batch Clipboard: MIT.
- Espanso and CopyQ: GPL-3.0; code reuse changes Bronze distribution obligations.
- PasteBar: [non-commercial Creative Commons terms at reviewed revision](https://github.com/PasteBar/PasteBarApp/blob/c89d054d29a0b51fdd3a031757a8dd21a09bc241/CC-LICENSE) with project-specific limited commercial exception; not [OSI open source](https://opensource.org/osd). Treat as design/architecture research unless legal review approves code use.
- Product behavior, public APIs, and general workflow concepts may be reimplemented; do not copy protected assets, copywriting, or non-permitted source.

### Domain safety

Maccy repository and official material warn about similarly named malicious domains. Use [maccy.app](https://maccy.app/) and the maintainer repository only.

### Negative accessibility evidence

No reviewed official source published a VPAT/ACR or independent WCAG 2.2 AA native-macOS audit. This should be recorded as “no public evidence found,” never “product is inaccessible.”

## Research conclusions supported by this register

1. Copper’s distinct workflow is deliberate selection capture plus transient queue and copy-back, not generic clipboard history.
2. macOS 26 commoditizes searchable clipboard history.
3. Marker, ActionClip, PopClip, OnText, and Pluks provide strongest reviewed selected-text benchmarks, with materially different privacy/trigger models.
4. Antinote and SideNotes provide best transient scratchpad/panel benchmarks.
5. Pastebot, Paste Stack, CleanClip, Stacked, PastePal, Batch Clipboard, and FlowClip provide strongest reviewed sequential-queue benchmarks.
6. Things provides best quick-entry and completion-state benchmark.
7. PastePal provides strongest public accessibility/localization declaration.
8. BetterTouchTool, Raycast, Alfred, and Keyboard Maestro prove full workflow can be assembled, but not with Bronze’s focused simplicity.
9. Historical Cooper `v0.2.1` issue #3 demonstrates concrete risk in that raw cross-platform listener implementation; it does not prove every cross-platform listener fails. Current `v0.3.5` needs separate CGEventTap evaluation.
10. No reviewed product was found combining evidence-backed accessibility, layout-independent shortcuts, deliberate privacy, and diagnostic capture health in one focused application.

## Mandatory recheck before release

- Copper price, OS requirement, and workflow text.
- macOS current Accessibility/Input Monitoring behavior.
- Cooper open issues and whether crash #3 is fixed.
- ActionClip promotion and free limit.
- App Store price/localization/accessibility metadata.
- Raycast, Drafts, Paste, Todoist, TickTick, Superlist, and Steecky subscriptions.
- PasteBar license text.
- PopClip exact regional purchase price.
- Apple Spotlight Clipboard availability by macOS version and region.
- Competitor privacy policies after major releases.
