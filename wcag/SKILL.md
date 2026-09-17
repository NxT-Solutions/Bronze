---
name: wcag
description: Applies WCAG 2.2 to Bronze WebView chrome. Use when changing Queue, Settings, Library, Help, CSS, contrast, focus, keyboard, labels, live regions, or when the user mentions WCAG, accessibility, AA, AAA, or screen readers.
---

# WCAG 2.2 — agent workflow

Read this file before any visual, interaction, or copy change that
people perceive or operate. Then read the linked files the change
touches. Do not invent a second accessibility system.

Official authority: `wcag/README.md`.

## When this applies

- Any edit under `apps/desktop/src/**` HTML, CSS, or surface JS
- Any edit to `packages/ui` styles, contrast, or chrome components
- New or changed locale strings that appear in the WebView
- User asks for WCAG, AA, AAA, contrast, focus, keyboard, or a11y
- A screenshot of Queue, Settings, Library, or Help

## Hard stops (do not violate)

- POUR first: perceivable, operable, understandable, robust.
- WebView claim *target* is WCAG 2.2 Level AA. Meet AAA where the
  product already can without new settings, media, or a color picker.
- Never claim WCAG, VoiceOver, or notarization without
  `docs/evidence/` artifacts and human gate 9.3.
- Semantic HTML. Real `button` / `a` / `label` / `summary`. No
  clickable `div`. Visible `:focus-visible` that meets 1.4.11.
- Accessible name contains the visible label (2.5.3). Do not replace
  visible wording with a divergent `aria-label`.
- Every user-facing string lives in the locale catalog. No sentence
  concatenation. HTML fallback matches `en`.
- `#action-status` stays a visually hidden live region. Feedback that
  people see lives on the control.
- Do not add `role="menu"` unless the control implements full desktop
  menu keyboard behavior (`docs/10`).
- 4.1.1 Parsing is obsolete in WCAG 2.2. Do not spend a gate on it.
- No remote fonts, CDN, hosted AI, or network by default (SEC-001,
  ADR-017).
- Do not copy Cooper/Copper trade dress.

## Workflow — any chrome change

1. Name the surface: Queue (`index.html`), Settings, Library, Help.
2. Read `screens.md` for that surface and `bronze.md` for tokens.
3. Map the change to a principle in `principles.md` and the success
   criteria in `criteria.md`.
4. Prefer meeting the AA row. If an AAA row is reachable with a token
   or markup change (contrast 7:1, 80ch prose, line-height 1.5, focus
   appearance), take it. Leave AAA Partial when it fights compact Mac
   HIG (2.5.5 44×44) or needs a new settings capability (1.4.8
   fg/bg picker).
5. Edit existing ui-layer files when possible. A new file is Hedgehog
   change-work.
6. Run the ui verify command from `.hedgehog/core.yaml`.
7. Update `docs/evidence/WCAG-22-INVENTORY.md` when a criterion’s
   support changes. Do not flip a public claim.

## Workflow — full surface audit

Copy and tick:

```
Surface: ________
- [ ] 1.1.1 Decorative images alt=""; informative images have text
- [ ] 1.3.1 Headings and labels match the visual structure
- [ ] 1.3.2 DOM order is the reading order
- [ ] 1.4.1 Status is not color-only (text or state on the control)
- [ ] 1.4.3 / 1.4.6 Text contrast 4.5:1 AA, 7:1 AAA where tokens allow
- [ ] 1.4.11 UI / non-text contrast ≥3:1 (borders, icons, focus)
- [ ] 1.4.13 Hover/focus tips are dismissible, hoverable, persistent
- [ ] 2.1.1 Every operation has a keyboard path
- [ ] 2.1.2 No keyboard trap (Escape leaves sheets and overflow)
- [ ] 2.4.1 Skip link is first focusable; target can take focus
- [ ] 2.4.2 Unique document title
- [ ] 2.4.3 Focus order matches visual order
- [ ] 2.4.6 Headings and labels describe topic or purpose
- [ ] 2.4.7 / 2.4.13 :focus-visible is 2px ring at ≥3:1
- [ ] 2.5.3 Accessible name contains the visible text
- [ ] 2.5.8 Targets ≥24×24 (AA). 44×44 is AAA Partial on compact Mac
- [ ] 3.1.1 html lang is set; RTL/pseudo still work
- [ ] 3.3.1 / 3.3.2 Errors and fields have associated text
- [ ] 4.1.2 Name, role, value on custom widgets
- [ ] 4.1.3 Status messages use role=status or role=alert
- [ ] Reduce Motion / Increase Contrast / Reduce Transparency still work
- [ ] Strings come from the catalog; HTML fallback matches en
```

## Workflow — screenshot review

Last-letter doubling in a macOS window screenshot is a capture
artifact, not official `en-XA`. Word smash is a real defect.

Do not treat a pretty screenshot as WCAG evidence. Contrast is
measured with `packages/ui/src/lib/contrast.ts`. VoiceOver is human
gate 9.3.

## Progressive disclosure

- Principles: [principles.md](principles.md)
- Success criteria: [criteria.md](criteria.md)
- Bronze application: [bronze.md](bronze.md)
- Per-window spec: [screens.md](screens.md)
- Adapter map: [README.md](README.md)
- Conformance plan: `docs/10-accessibility-conformance-plan.md`
- Inventory: `docs/evidence/WCAG-22-INVENTORY.md`
