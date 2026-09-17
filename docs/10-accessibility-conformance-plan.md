# Accessibility conformance plan

## 1. Claim and standards

Do not use “WCAG proof” or “A11y proof.” Accessibility is continuous property of specific build, platform, settings, and content. Target and publish evidence:

- WebView UI: [WCAG 2.2](https://www.w3.org/TR/WCAG22/) Level AA.
- Whole desktop software: [ETSI EN 301 549 V3.2.1](https://www.etsi.org/deliver/etsi_en/301500_301599/301549/03.02.01_60/en_301549v030201p.pdf), especially clauses 5, 11, and 12.
- macOS behavior: [Apple VoiceOver evaluation criteria](https://developer.apple.com/help/app-store-connect/manage-app-accessibility/voiceover-evaluation-criteria/) and [Apple accessibility guidance](https://developer.apple.com/design/human-interface-guidelines/accessibility/).
- Public artifact: ACR/VPAT-style matrix, test environment, known limitations, remediation status, and audit date.

WCAG is written for web pages. Claim WCAG 2.2 AA for WebView surfaces only; use relevant EN 301 549 clauses 5, 11, and 12 plus Apple/native testing for app as whole.

Standards version gate: V3.2.1 remains current harmonized EN 301 549 baseline at research cutoff; [V4.1.0 approval draft](https://www.etsi.org/deliver/etsi_EN/301500_301599/301549/04.01.00_30/en_301549v040100va.pdf) exists but is not conformance baseline. Recheck [ETSI directory](https://www.etsi.org/deliver/etsi_en/301500_301599/301549/) and [EU harmonization status](https://digital-strategy.ec.europa.eu/en/policies/web-accessibility-directive-standards-and-harmonisation) before implementation/release claims.

## 2. Normative mapping priorities

Rendered review of EN 301 549 found relevant software provisions:

- 11.1: text alternatives, adaptable structure, sensory characteristics, contrast, resize, reflow, non-text contrast, text spacing.
- 11.2: keyboard, no trap, character-key shortcuts, timing, focus order, focus visible, pointer cancellation/gestures.
- 11.3: language, consistent behavior, errors, labels/instructions.
- 11.4: name, role, value and status messages.
- 11.5: use of platform accessibility services, focus and selection tracking/modification.
- 11.6: accessible documents and no disruption of accessibility features.
- 11.7: user preferences.
- 12: accessible product documentation and support.

Clause 5.8 says double-strike acceptance window must be adjustable to at least 0.5 seconds where double-strike is used. Clause 5.9 requires a mode not requiring simultaneous actions. Bronze therefore makes modifier double tap optional/configurable and always provides menu/standard chord/manual routes.

### 2.1 Complete WCAG inventory gate

Priority summaries above do not replace conformance accounting. Draft
WebView rows live in [`docs/evidence/WCAG-22-INVENTORY.md`](evidence/WCAG-22-INVENTORY.md)
(A, AA, and AAA). That file is not a public AA/AAA claim.

Before DG-08 can close, ACR source must contain one *reviewed* row for
every WCAG 2.2 Level A and AA success criterion, including criteria that
appear unrelated to desktop utility UI. Each row records `Supports`,
`Partially Supports`, `Does Not Support`, or `Not Applicable`, with
scoped rationale, implementation, build/platform, and evidence.
Explicitly evaluate newer criteria such as Consistent Help (3.2.6),
Redundant Entry (3.3.7), and Accessible Authentication (3.3.8); do not
omit them by sampling. Any applicable partial/non-support blocks public
AA claim until remediated or claim scope changes honestly.

## 3. User journeys under test

Each journey must pass keyboard-only, VoiceOver, Voice Control, and Switch Control unless tool cannot exercise global source app; then split source and Bronze evidence explicitly.

| Journey | Evidence |
| --- | --- |
| onboard without granting permissions | logical focus, readable reasons, manual route remains usable |
| grant/revoke/recover Input Monitoring and Accessibility | status differentiated, System Settings guidance, retest result announced |
| capture supported text | no timed gesture required; source focus preserved; native announcement/persistent status tested while Bronze WebView hidden |
| handle no selection/secure field/unsupported source | content-safe error, recovery control reachable |
| add/edit/cancel/save item with IME | composition preserved, errors associated, draft recovery |
| select/reorder/move item | no drag requirement, position/state announced |
| copy one/many with output profile | visible and spoken confirmation, lifecycle predictable |
| complete/undo/trash/restore | state/name/value updated, focus survives removal |
| search/filter/clear | result count announced without chatter, empty state accessible |
| change shortcut/settings/reset | recorder accessible, conflicts explained, old value retained on failure |
| export/import/backup/restore | native picker accessible, progress/result, destructive confirmation |
| open menu-bar menu/quick panel/settings/help | native roles/names/actions, no focus trap |

## 4. Semantic component requirements

- Use native HTML controls first. shadcn primitives are owned source, not automatic conformance.
- Lists use list semantics; headings reflect hierarchy; each card has meaningful accessible grouping.
- Completion checkbox label includes item summary; selection checkbox has distinct wording.
- Dialog: name/description, initial focus, modal/inert background, Escape, focus restoration.
- Menu: correct menu/menuitem roles only when desktop menu keyboard behavior fully implemented; otherwise use labeled popover with buttons.
- Toast/status: `role=status`/polite region for routine result; alert only for urgent blocking error. Critical actions persist outside toast.
- Tooltip never sole label/instruction. Authored hover/focus content is keyboard- and pointer-accessible, dismissible without moving focus/pointer unless WCAG exception applies, hoverable, and persistent until dismissed, trigger leaves, or information becomes invalid, satisfying WCAG 1.4.13.
- For WCAG 2.5.3 and Voice Control, accessible name contains exact visible control text, preferably as prefix. Do not replace visible wording with divergent `aria-label` text.
- Icon-only controls have programmatic and localized accessible names.
- Current, selected, expanded, checked, invalid, busy, and disabled states exposed.
- Avoid virtualized list in MVP; if later added, AT navigation/position semantics require dedicated ADR and audit.

## 5. Keyboard and focus

- Every operation works without pointer; no gesture/timing-only path.
- Standard tab/shift-tab order matches visual/logical order.
- AA gates: every keyboard-operable control has visible focus under 2.4.7; author-created content does not entirely hide it under 2.4.11; authored indicator meets applicable 1.4.11 non-text contrast. Bronze enhanced target also meets WCAG 2.4.13 Focus Appearance's ≥2 CSS px equivalent perimeter and ≥3:1 state-change contrast; 2.4.13 is Level AAA, not part of AA claim.
- Character-key shortcuts can be disabled/remapped and do not trigger while editing/composing.
- Context-aware shortcuts do not steal text-editing keys.
- Modal/popover focus is contained only while truly modal; panel never traps user.
- Summon/hide and Escape restore source/trigger focus where platform permits.
- Removing/moving content follows deterministic focus rules in UX spec.
- Sticky Keys and Slow Keys included in trigger matrix.

## 6. Visual, reflow, and motion

- Text contrast: 4.5:1 normal, 3:1 large; UI/non-text and focus 3:1. Measure actual rendered tokens in light/dark/high-contrast.
- Test 200% text resize; WebView zoom/reflow at 400% and 320 CSS px width. No two-dimensional scroll for ordinary UI. Toolbar overflow uses a labeled `panel.toolbar.overflow` control; the composer stays in document flow. Automated layout tests are not a public WCAG claim (A11Y-003).
- Text spacing override does not clip or hide content.
- Targets meet WCAG 2.2 2.5.8 AA 24×24 CSS px or valid spacing/exception; aim 44×44 where panel density permits.
- Status never color-only. Icons include text/accessible state.
- Respect `prefers-reduced-motion`, macOS Reduce Motion, Increase Contrast, Differentiate Without Color, and Reduce Transparency.
- Opaque fallback guarantees contrast; blur/vibrancy optional.
- No flashing content or motion needed to understand/reorder.

## 7. Native macOS accessibility

- Quick editor uses normal activating window for dependable keyboard/VoiceOver focus.
- Status item/menu has localized label, role, enabled/state/action.
- Window titles identify Bronze and purpose.
- Native permission health and file pickers use system controls where possible.
- App must not suppress platform accessibility services, custom focus, or selection.
- Use Accessibility Inspector to inspect both WebView and native surfaces.
- Validate announcement timing: native focus change and WebView live region should not double-speak.
- Native display bridge reads Reduce Motion, Reduce Transparency, Increase Contrast, and Differentiate Without Color from `NSWorkspace`, observes [`accessibilityDisplayOptionsDidChangeNotification`](https://developer.apple.com/documentation/appkit/nsworkspace/accessibilitydisplayoptionsdidchangenotification), and updates every visible/hidden window. App overrides may only strengthen active system preferences.

## 8. Automated gates

Per component/feature:

- Testing Library queries by role/name, not implementation selectors.
- `axe-core`/`jest-axe` on stable states: default, empty, loading, error, dialog, menu/popover, selected, completed, RTL, zoom.
- lint semantic traps, missing names, invalid ARIA, positive tabindex.
- contrast token tests plus browser rendered sampling for key states.
- Playwright/Tauri WebDriver keyboard journeys where supported.
- snapshot accessibility trees selectively to detect lost role/name/state.

Automation cannot prove VoiceOver, platform focus, permissions, global triggers, color perception, reflow usability, or meaningful labels.

## 9. Manual matrix

Minimum per release candidate:

- VoiceOver with keyboard and trackpad basics.
- Full Keyboard Access.
- Voice Control visible labels/number overlay.
- Switch Control scan/navigation.
- Sticky Keys, Slow Keys, Key Repeat variants.
- 200%/400%, large system text setting where applicable.
- Light, dark, Increase Contrast, Differentiate Without Color, Reduce Motion, Reduce Transparency.
- Built-in and external keyboard; QWERTY, AZERTY, QWERTZ, Dvorak/Colemak if supported.
- English, Arabic RTL, Japanese IME, pseudo-expansion, bidi stress.
- Mouse, trackpad, keyboard-only.

Record OS build, hardware, app build/signature, settings, assistive tech, steps, result, issue, and evidence link.

## 10. Documentation and support

EN 301 549 clause 12 means help/support must also be accessible. Provide:

- accessible HTML/Markdown help, not image-only instructions;
- permission walkthrough with text alternatives;
- complete shortcut table and remapping guidance;
- supported-app matrix and failure recovery;
- local/privacy/data backup explanation;
- accessibility features and known limitations — including human gates 3.9, 3.10, 5.5, and 9.3;
- redacted diagnostics preview before any local export; no automatic upload (SUP-001/002, SEC-006);
- contact/support path usable without inaccessible form;
- exported ACR/conformance matrix in accessible format.

Marketing video needs captions, transcript, keyboard controls, pause, and audio description of essential visual steps if Bronze publishes one.

## 11. Release evidence template

For every applicable criterion record:

```text
Standard/criterion:
Applicability:
Requirement IDs:
Implementation:
Automated evidence:
Manual evidence:
Platforms/settings tested:
Result: supports | partially supports | does not support | not applicable
Known limitation/remediation:
Reviewer/date/build:
```

Independent accessibility audit and completed row-for-row WCAG 2.2 A/AA inventory are required before public “conforms” claim. Fix severity based on blocked task and user impact, not automated rule count.
