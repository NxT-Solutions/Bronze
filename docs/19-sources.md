# Sources and evidence index

Research cutoff: 2026-08-27. Prefer primary/normative sources. Product prices/features are first-party snapshots and must be rechecked.

## Copper reference

- [Copper landing/product/video](https://shadcn.com/copper)
- [Copper privacy policy](https://shadcn.com/copper/privacy)
- [Copper terms](https://shadcn.com/copper/terms)
- Landing asset: `https://shadcn.com/copper.mp4` (47.253 s; no text tracks observed)

Behavioral notes: [reference analysis](01-product-reference-analysis.md), [video shotlist](../research/reference-video-shotlist.md).

## Cooper OSS

- [Repository](https://github.com/TouchMyBar/cooper)
- [Apache-2.0 license](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/LICENSE)
- [Current audited commit](https://github.com/TouchMyBar/cooper/commit/14ff307023b0e94d48a209db578a3e411bee7d10)
- [v0.3.5 release](https://github.com/TouchMyBar/cooper/releases/tag/v0.3.5)
- [Issue #3 historical `v0.2.1` macOS crash](https://github.com/TouchMyBar/cooper/issues/3)
- [Issue #2 maintainer response](https://github.com/TouchMyBar/cooper/issues/2#issuecomment-5384321155)
- [Unmerged PR #1](https://github.com/TouchMyBar/cooper/pull/1)
- [macOS tap implementation](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/src-tauri/src/mac_tap.rs)
- [capture implementation](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/src-tauri/src/capture.rs)
- [database](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/src-tauri/src/db.rs)
- [commands](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/src-tauri/src/commands.rs)
- [Tauri config](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/src-tauri/tauri.conf.json)
- [build workflow](https://github.com/TouchMyBar/cooper/blob/14ff307023b0e94d48a209db578a3e411bee7d10/.github/workflows/build.yml)
- [promotional Reddit post](https://www.reddit.com/r/ClaudeAI/comments/1ve1dgb/i_recreated_shadcns_copper_as_a_free_opensource/)

Full evidence: [Cooper audit](16-cooper-oss-audit.md).

## Apple platform

- [CGEvent tap creation](https://developer.apple.com/documentation/coregraphics/cgevent/tapcreate%28tap%3Aplace%3Aoptions%3Aeventsofinterest%3Acallback%3Auserinfo%3A%29)
- [`listenOnly` event-tap option](https://developer.apple.com/documentation/coregraphics/cgeventtapoptions/listenonly)
- [CGEvent timestamp](https://developer.apple.com/documentation/coregraphics/cgeventtimestamp)
- [tap disabled by timeout](https://developer.apple.com/documentation/coregraphics/cgeventtype/tapdisabledbytimeout)
- [Input Monitoring preflight](https://developer.apple.com/documentation/coregraphics/cgpreflightlisteneventaccess%28%29)
- [Apple DTS: listen-only event taps and Input Monitoring](https://developer.apple.com/forums/thread/707680)
- [macOS Input Monitoring user guide](https://support.apple.com/guide/mac-help/control-access-to-input-monitoring-on-mac-mchl4cedafb6/mac)
- [AX process trust](https://developer.apple.com/documentation/applicationservices/1459186-axisprocesstrustedwithoptions)
- [AX UI element messaging timeout](https://developer.apple.com/documentation/applicationservices/1459345-axuielementsetmessagingtimeout)
- [AX selected-text attribute](https://developer.apple.com/documentation/applicationservices/kaxselectedtextattribute)
- [AX selected-text range](https://developer.apple.com/documentation/applicationservices/kaxselectedtextrangeattribute)
- [AX attributes](https://developer.apple.com/documentation/applicationservices/carbon_accessibility/attributes)
- [secure text-field subrole](https://developer.apple.com/documentation/applicationservices/kaxsecuretextfieldsubrole)
- [frontmost application](https://developer.apple.com/documentation/appkit/nsworkspace/frontmostapplication)
- [NSPasteboard](https://developer.apple.com/documentation/appkit/nspasteboard)
- [pasteboard changeCount](https://developer.apple.com/documentation/appkit/nspasteboard/changecount)
- [Secure Keyboard Entry](https://support.apple.com/en-il/guide/terminal/trml109/mac)
- [NSStatusItem](https://developer.apple.com/documentation/appkit/nsstatusitem)
- [application activation policy](https://developer.apple.com/documentation/appkit/nsapplication/activationpolicy-swift.enum)
- [Application Support directory](https://developer.apple.com/documentation/foundation/url/applicationsupportdirectory)
- [protecting local data with containers](https://developer.apple.com/documentation/xcode/protecting-local-app-data-using-containers)
- [App Sandbox guidance](https://developer.apple.com/documentation/security/protecting-user-data-with-app-sandbox)
- [Hardened Runtime](https://developer.apple.com/documentation/security/hardened-runtime)
- [notarizing macOS software](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution)
- [Apple HIG accessibility](https://developer.apple.com/design/human-interface-guidelines/accessibility/)
- [VoiceOver evaluation criteria](https://developer.apple.com/help/app-store-connect/manage-app-accessibility/voiceover-evaluation-criteria/)
- [Apple accessibility testing](https://developer.apple.com/documentation/accessibility/performing-accessibility-testing-for-your-app)
- [AppKit accessibility announcement notification](https://developer.apple.com/documentation/appkit/nsaccessibility-swift.struct/notification/announcementrequested)
- [NSWorkspace accessibility display preferences](https://developer.apple.com/documentation/appkit/nsworkspace)
- [accessibility display-options change notification](https://developer.apple.com/documentation/appkit/nsworkspace/accessibilitydisplayoptionsdidchangenotification)
- [macOS 26 Spotlight clipboard](https://support.apple.com/en-euro/guide/mac-help/mchl40d5b86b/26/mac/26)
- [Universal Clipboard](https://support.apple.com/en-us/102430)

## Tauri

- [Tauri security](https://v2.tauri.app/security/)
- [capabilities per window/platform](https://v2.tauri.app/learn/security/capabilities-for-windows-and-platforms/)
- [permissions](https://v2.tauri.app/security/permissions/)
- [runtime authority](https://v2.tauri.app/security/runtime-authority/)
- [content security policy](https://v2.tauri.app/security/csp/)
- [asset protocol scope](https://v2.tauri.app/security/asset-protocol/)
- [security lifecycle](https://v2.tauri.app/security/lifecycle/)
- [global shortcut plugin](https://v2.tauri.app/plugin/global-shortcut/)
- [global shortcut JavaScript API](https://v2.tauri.app/reference/javascript/global-shortcut/)
- [WebView versions](https://v2.tauri.app/reference/webview-versions/)
- [macOS signing/notarization](https://v2.tauri.app/distribute/sign/macos/)
- [WebDriver testing](https://v2.tauri.app/develop/tests/webdriver/)
- [window builder: all workspaces](https://docs.rs/tauri/latest/tauri/window/struct.WindowBuilder.html#method.visible_on_all_workspaces)

## Accessibility, internationalization, UI

- [WCAG 2.2](https://www.w3.org/TR/WCAG22/)
- [ETSI EN 301 549 V3.2.1 PDF](https://www.etsi.org/deliver/etsi_en/301500_301599/301549/03.02.01_60/en_301549v030201p.pdf)
- [ETSI EN 301 549 version directory](https://www.etsi.org/deliver/etsi_en/301500_301599/301549/)
- [ETSI EN 301 549 V4.1.0 approval draft](https://www.etsi.org/deliver/etsi_EN/301500_301599/301549/04.01.00_30/en_301549v040100va.pdf)
- [European Commission harmonization status](https://digital-strategy.ec.europa.eu/en/policies/web-accessibility-directive-standards-and-harmonisation)
- [RFC 5646 / BCP 47](https://www.rfc-editor.org/info/rfc5646/)
- [RFC 4647 language-tag lookup](https://www.rfc-editor.org/rfc/rfc4647.html)
- [ECMA-402 Internationalization API](https://tc39.es/ecma402/)
- [Unicode MessageFormat](https://www.unicode.org/reports/tr35/tr35-messageFormat.html)
- [shadcn React Aria announcement](https://ui.shadcn.com/docs/changelog/2026-07-react-aria)
- [shadcn RTL guidance](https://ui.shadcn.com/docs/rtl)

## Competitors

Detailed dated URLs, price checks, feature/privacy/a11y evidence, verification hierarchy, and volatility notes: [competitor source register](../research/competitor-source-register.md). Methodology defines A–D evidence grades. Principal official references:

- [ActionClip](https://actionclip.app/), [PopClip](https://www.popclip.app/), [OnText](https://gityeop.github.io/OnText/), [Marker](https://getmarkerapp.net/), [Pluks](https://pluks.app/)
- [Antinote](https://antinote.io/features), [SideNotes](https://www.apptorium.com/sidenotes), [Steecky](https://steecky.app/)
- [PasteBar](https://www.pastebar.app/), [PasteBar repo](https://github.com/PasteBar/PasteBarApp), [Pastebot](https://tapbots.com/pastebot), [PastePal](https://indiegoodies.com/pastepal)
- [Drafts](https://docs.getdrafts.com/gettingstarted/), [Things](https://culturedcode.com/things/)
- [Raycast](https://www.raycast.com/pricing), [Alfred](https://www.alfredapp.com/powerpack/), [BetterTouchTool](https://folivora.ai/buy/), [Keyboard Maestro](https://www.keyboardmaestro.com/main/)
- [Maccy](https://maccy.app/), [Paste](https://pasteapp.io/pricing), [Paste Stack](https://pasteapp.io/help/using-paste-stack), [CleanClip](https://cleanclip.cc/), [Stacked](https://github.com/sharnobyl/stacked), [Clipbara](https://github.com/mobrava/Clipbara), [Unclutter](https://unclutterapp.com/), [CopyQ](https://copyq.readthedocs.io/en/stable/)
- [Pieces pricing](https://pieces.app/pricing)
- [Promta](https://promta.app/), [Prompt Box](https://promptboxapp.com/), [Snippety](https://snippety.app/pricing)
- [Scratchpad](https://sindresorhus.com/scratchpad), [Tot](https://tot.rocks/), [Jot](https://jot.arunbrahma.com/)

## Agent skills

- [skills.sh catalog](https://skills.sh/)
- [Anthropic skills repository](https://github.com/anthropics/skills)
- [Vercel agent skills](https://github.com/vercel-labs/agent-skills)
- [Matt Pocock skills](https://github.com/mattpocock/skills)
- [obra superpowers](https://github.com/obra/superpowers)
- [Jakub Krehel skills](https://github.com/jakubkrehel/skills)
- [Mindrally skills](https://github.com/mindrally/skills)

## Evidence interpretation

- Standards/platform/repository source: use as primary technical evidence.
- Product page/App Store: first-party claim, not independent verification.
- GitHub issue/Reddit: anecdotal/third-party; useful for failure hypothesis, never sole release decision.
- Code inference: explicitly label and reproduce in test before treating as Bronze requirement.
