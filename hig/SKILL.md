---
name: apple-hig
description: Applies Apple Human Interface Guidelines to Bronze chrome and UX. Use when changing Settings, Library, Help, queue, CSS, typography, motion, buttons, menus, or when the user mentions Apple design, HIG, polish, or lightweight UI.
---

# Apple HIG — agent workflow

Read this file before any visual or interaction change. Then read the
linked files that the change touches. Do not invent a second visual
system.

Official authority: `hig/README.md`.

## When this applies

- Any edit under `apps/desktop/src/**` HTML, CSS, or surface JS
- Any edit to `packages/ui` styles or chrome components
- User asks for Apple-like, HIG, polish, spacing, type, motion, or UX
- A screenshot of Queue, Settings, Library, or Help

## Hard stops (do not violate)

- Purpose first: Bronze is a selection-to-action queue. Chrome recedes.
- One filled primary control per view. Secondary is borderless or ghost.
- System SF only. No remote fonts, CDN, or bundled SF files (SEC-001).
- Live tokens stay zinc (`#fafafa` / `#18181b` / `#8a8a90`). DESIGN.md
  warm bronze hexes are inspiration, not live chrome.
- No fake Liquid Glass / CSS blur. Opaque surfaces. Materials need
  native rust + DESIGN fallback before anyone fakes them.
- Motion explains state, ≤200ms, `prefers-reduced-motion` and
  `data-reduce-motion` win. Delight is not decoration.
- Semantic HTML. Real `button` / `a` / `summary`. Visible
  `:focus-visible`. Locale catalog for every user-facing string. No
  sentence concatenation.
- Do not copy Cooper/Copper trade dress.

## Workflow — any chrome change

1. Name the surface: Queue (`index.html`), Settings, Library, Help.
2. Read `screens.md` for that surface and `bronze.md` for tokens.
3. Map the change to a principle in `principles.md` (Purpose, Agency,
   Responsibility, Familiarity, Flexibility, Simplicity, Craft, Delight).
4. Check `macos.md` for the control you are touching (type, button,
   list, field, menu, writing, motion).
5. Edit existing files in the ui layer when possible. A new file is
   Hedgehog change-work, not a silent extra CSS file.
6. Run the ui verify command from `.hedgehog/core.yaml`.
7. Do not claim WCAG, VoiceOver, or notarization without
   `docs/evidence/` artifacts.

## Workflow — full surface audit

Copy and tick:

```
Surface: ________
- [ ] Purpose: content is the first thing the eye hits
- [ ] Agency: chrome stays out of the way; mistakes are reversible
- [ ] Hierarchy: title > body > meta > actions (weight/size/color)
- [ ] One primary; other actions ghost or in a compact menu
- [ ] Fields and adjacent buttons do not overlap (no width:100% in a row)
- [ ] Lists show human labels, not raw IDs
- [ ] SF as designed (no extra letter-spacing on body)
- [ ] Hairline grouping, not drop-shadow hierarchy
- [ ] Corners concentric (inner = outer − padding); Mac medium controls are rounded rects, not capsules
- [ ] Paragraphs and rows have breathing room
- [ ] Feedback lives on the control (tip), live region stays SR-only
- [ ] Reduce Motion / Increase Contrast / Reduce Transparency still work
- [ ] Strings come from the catalog; HTML fallback matches `en`
```

## Workflow — screenshot review

Last-letter doubling in a macOS window screenshot is a capture artifact,
not official `en-XA`. Word smash (spaces gone) is a real defect: check
`word-spacing`, `width: 100%` in flex rows, and catalog fallbacks.

`en-XA` / `ar-XB` are advertised pseudo-locales. Hand-test default is
`en`. Do not “fix” English by stripping last letters.

## Progressive disclosure

- Principles: [principles.md](principles.md)
- macOS foundations: [macos.md](macos.md)
- Bronze application: [bronze.md](bronze.md)
- Per-window spec: [screens.md](screens.md)
- Adapter map: [README.md](README.md)
