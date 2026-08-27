# Standards review notes

Reviewed 2026-08-27.

## WCAG 2.2

[WCAG 2.2](https://www.w3.org/TR/WCAG22/) is W3C Recommendation. Level AA conformance requires all applicable A and AA success criteria for full WebView pages/states. Bronze uses it for rendered UI, not as sole whole-desktop-app claim.

High-impact additions/areas for Bronze include Focus Not Obscured (Minimum), Dragging Movements alternative, Target Size (Minimum), Consistent Help, Redundant Entry, and Accessible Authentication where applicable, plus established keyboard/focus/name-role-value/status/contrast/reflow/error criteria. Public AA claim requires reviewed row for every Level A/AA criterion, including reasoned not-applicable rows; priority list is not substitute.

Focus Appearance (2.4.13) is Level AAA. Bronze keeps its two-CSS-pixel-equivalent/3:1 metric as enhanced target while separately gating AA Focus Visible (2.4.7), Focus Not Obscured (Minimum) (2.4.11), and applicable Non-text Contrast (1.4.11).

## EN 301 549 V3.2.1

Official PDF: [ETSI EN 301 549 V3.2.1](https://www.etsi.org/deliver/etsi_en/301500_301599/301549/03.02.01_60/en_301549v030201p.pdf). PDF is 186 pages and tagged. Relevant pages/clauses were rendered and visually reviewed, not inferred only from extraction.

Version note: V3.2.1 remains current harmonized baseline at research cutoff. ETSI published [V4.1.0 approval draft](https://www.etsi.org/deliver/etsi_EN/301500_301599/301549/04.01.00_30/en_301549v040100va.pdf) in 2026, but draft is not substituted for harmonized baseline. Recheck [ETSI version directory](https://www.etsi.org/deliver/etsi_en/301500_301599/301549/) and [European Commission harmonization status](https://digital-strategy.ec.europa.eu/en/policies/web-accessibility-directive-standards-and-harmonisation) at implementation start and each conformance claim.

- Clause 5.8: where double-strike is accepted, timing acceptance must be adjustable to at least 0.5 seconds.
- Clause 5.9: provide mode not requiring simultaneous user actions.
- Clause 11: non-web software UI maps accessibility requirements, including WCAG-based software criteria, platform accessibility services, focus/selection, user preferences.
- Clause 12: product documentation and support information/services must be accessible.

Design consequence: modifier double tap can be efficient optional route, never sole route. Adjustable ≥500 ms range, standard chord, menu-bar command, visible control, and manual entry all required.

## Apple

Whole-app evidence uses Apple’s VoiceOver evaluation criteria and Accessibility Inspector/manual assistive technologies. WKWebView axe checks do not exercise native menu item, TCC prompt, AppKit focus, global trigger, or Space/display behavior.

Test at minimum VoiceOver, Full Keyboard Access, Voice Control, Switch Control, Sticky Keys, Slow Keys, Increase Contrast, Differentiate Without Color, Reduce Motion, Reduce Transparency, text scaling/zoom, and keyboard layouts/input methods.

## Claim language

Allowed after evidence:

- “WebView UI tested against WCAG 2.2 AA.”
- “Desktop software mapped to relevant EN 301 549 clauses 5, 11, and 12.”
- “Core journeys tested with VoiceOver, Full Keyboard Access, Voice Control, and Switch Control on listed macOS builds.”

Avoid:

- “100% accessible.”
- “WCAG proof.”
- “A11y proof.”
- “Works everywhere.”

Publish build/OS/date/scope, result matrix, known limitations, and audit source.
