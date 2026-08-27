---
name: Bronze
description: Local macOS selection-to-action queue. Distills the existing Bronze visual system; not a new brand and not Copper/Cooper trade dress.
status: final
created: '2026-08-27'
updated: '2026-08-27'
sources:
  - docs/05-ux-ui-interaction-spec.md
  - docs/10-accessibility-conformance-plan.md
  - assets/README.md
  - assets/bronze-ui-concept.png
colors:
  # Opaque semantic surfaces. Material/vibrancy is optional and must fall back to these.
  background: '#F7F4EF'
  foreground: '#1C1917'
  card: '#FFFFFF'
  card-foreground: '#1C1917'
  muted: '#EFE8DE'
  muted-foreground: '#5C564E'
  border: '#756B60'
  input: '#756B60'
  ring: '#8C6239'
  primary: '#8C6239'
  primary-foreground: '#FFFFFF'
  accent: '#C4A265'
  accent-foreground: '#1C1917'
  success: '#2F6F4F'
  success-foreground: '#FFFFFF'
  warning: '#8A4E0E'
  warning-foreground: '#FFFFFF'
  destructive: '#B42318'
  destructive-foreground: '#FFFFFF'
  background-dark: '#1A1714'
  foreground-dark: '#F7F4EF'
  card-dark: '#2A2520'
  card-foreground-dark: '#F7F4EF'
  muted-dark: '#2A2520'
  muted-foreground-dark: '#E3D5C3'
  border-dark: '#A3988A'
  input-dark: '#A3988A'
  ring-dark: '#D4B483'
  primary-dark: '#D4B483'
  primary-foreground-dark: '#1A1714'
  accent-dark: '#D4B483'
  accent-foreground-dark: '#1A1714'
  success-dark: '#7BC49A'
  success-foreground-dark: '#1A1714'
  warning-dark: '#E0B070'
  warning-foreground-dark: '#1A1714'
  destructive-dark: '#F0A8A0'
  destructive-foreground-dark: '#1A1714'
typography:
  body:
    fontFamily: 'ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "SF Pro Text", sans-serif'
    fontSize: 14px
    fontWeight: '400'
    lineHeight: '1.4'
  label:
    fontFamily: 'ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "SF Pro Text", sans-serif'
    fontSize: 13px
    fontWeight: '500'
    lineHeight: '1.3'
  title:
    fontFamily: 'ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "SF Pro Text", sans-serif'
    fontSize: 16px
    fontWeight: '600'
    lineHeight: '1.3'
  caption:
    fontFamily: 'ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "SF Pro Text", sans-serif'
    fontSize: 12px
    fontWeight: '400'
    lineHeight: '1.35'
  mono:
    fontFamily: 'ui-monospace, "SF Mono", Menlo, monospace'
    fontSize: 13px
    fontWeight: '400'
    lineHeight: '1.45'
rounded:
  sm: 8px
  md: 10px
  lg: 12px
spacing:
  '1': 4px
  '2': 8px
  '3': 12px
  '4': 16px
  '5': 20px
  '6': 24px
components:
  button-primary:
    background: '{colors.primary}'
    foreground: '{colors.primary-foreground}'
    radius: '{rounded.md}'
  item-card:
    background: '{colors.card}'
    foreground: '{colors.card-foreground}'
    radius: '{rounded.md}'
    padding: '{spacing.3}'
    border: '{colors.border}'
  focus-ring:
    color: '{colors.ring}'
    width: 2px
  status-success:
    color: '{colors.success}'
    foreground: '{colors.success-foreground}'
  status-warning:
    color: '{colors.warning}'
    foreground: '{colors.warning-foreground}'
  status-error:
    color: '{colors.destructive}'
    foreground: '{colors.destructive-foreground}'
  status-neutral:
    color: '{colors.muted-foreground}'
    foreground: '{colors.foreground}'
  quick-panel:
    background: '{colors.background}'
    foreground: '{colors.foreground}'
    radius: '{rounded.lg}'
  composer:
    background: '{colors.card}'
    foreground: '{colors.foreground}'
    radius: '{rounded.sm}'
    border: '{colors.input}'
---

## Brand & Style

Bronze is temporary working memory attached to the current task: instant to summon, quiet when idle, explicit about state, forgiving when capture fails. It must not look or behave like surveillance software, a clipboard recorder, or a Copper/Cooper clone.

This file distills the approved visual system in `docs/05-ux-ui-interaction-spec.md` and the original concept in `assets/bronze-ui-concept.png` (inspiration, not a pixel specification). It does not invent a new brand. Concept provenance is recorded in `assets/README.md`. Do not copy Copper screenshots, icons, name, or pixel-identical trade dress.

WebView chrome inherits owned shadcn React Aria primitives. This DESIGN.md specifies only Bronze deltas: warm bronze accent, system UI type, opaque default surfaces, modest radius, 4 px spacing, and status tokens that never rely on color alone. Unlisted shadcn tokens may inherit until measured contrast fails.

## Colors

Planning hex values implement the specified warm bronze accent and opaque semantic surfaces. Measure every pair in light, dark, and Increase Contrast before treating them as ship tokens. Prefer margin above WCAG minima: normal text 4.5:1, large text 3:1, UI components and focus 3:1.

- **Background (`{colors.background}` / `{colors.background-dark}`)** is the default opaque panel and window fill. Translucency setting `material` is optional decoration only. Reduce Transparency and Increase Contrast force this opaque fill plus `{colors.border}`.
- **Foreground (`{colors.foreground}` / `{colors.foreground-dark}`)** is primary UI text and icons. Never drop below independent contrast against the surface in use.
- **Warm bronze primary (`{colors.primary}` / `{colors.primary-dark}`)** is filled action chrome: primary buttons, active section, strong selection. Pair with `{colors.primary-foreground}`.
- **Warm bronze accent (`{colors.accent}` / `{colors.accent-dark}`)** marks focus, selection, and active queue state. Pair with `{colors.accent-foreground}`. `{colors.accent}` is not a text color on `{colors.background}`; it is a fill. Text and icons on accent keep their own contrast.
- **Ring (`{colors.ring}` / `{colors.ring-dark}`)** is the focus indicator. It is a ≥2 CSS px equivalent perimeter, never shadow-only.
- **Status (`{colors.success}`, `{colors.warning}`, `{colors.destructive}`, `{colors.muted-foreground}`)** each require icon, visible label, and accessible description. Color is never the only status channel (A11Y-003).
- **Border (`{colors.border}`)** is informational structure at ≥3:1 against the adjacent surface. Subtle decorative dividers may be quieter only when they convey no state.

Avoid: Copper-like purple/violet brand, neon accents, gradient hero surfaces, color-only badges, remote/CDN fonts, transparent text on vibrancy without the opaque fallback.

## Typography

System UI stack only. No remote fonts (SEC-001). Base `{typography.body.fontSize}` is 14 px equivalent and must resize with OS text size and app scale. Never encode hierarchy by size alone: pair size with weight, spacing, and heading semantics.

- `{typography.body}` — panel copy, item previews, settings descriptions.
- `{typography.label}` — control labels, section names, status text.
- `{typography.title}` — window/panel titles and dialog names.
- `{typography.caption}` — provenance, shortcut hints, timestamps. Still ≥3:1 as large/non-body text or bump to body size.
- `{typography.mono}` — optional user-content code; UI chrome stays sans.

User content uses `lang` + `dir="auto"` from item `contentLanguage` (I18N-003). UI `lang` must not leak onto captured text. Truncation is grapheme-safe.

## Layout & Spacing

4 px base scale: `{spacing.1}` through `{spacing.6}`. Minimum item-card padding is `{spacing.3}` (12 px). Dense/compact mode still meets 24×24 CSS px pointer targets or a documented WCAG 2.5.8 spacing exception; aim 44×44 where panel density permits (A11Y-003).

Quick panel is one column of header, queue toolbar, section list, composer, footer. At 200% text and at 400% zoom / 320 CSS px, toolbar wraps into labeled overflow; composer and status stay reachable; no horizontal scroll for chrome; user content wraps; code may scroll inside a bounded region.

Use CSS logical properties (`margin-inline-start`, `inset-inline-end`) for chrome. Physical `left|right|top` panel edge is a stored screen-edge setting and does not mirror in RTL (WIN-001, I18N-003).

## Elevation & Depth

Shadow is optional and not a hierarchy device. Reduce Transparency and Increase Contrast remove shadow and strengthen `{colors.border}`. Focus never relies on shadow alone. No parallax, flashing, or auto-moving surfaces. Transitions ≤200 ms and are not required to understand state. Reorder motion disables under Reduce Motion.

## Shapes

Modest corners only: `{rounded.sm}` 8 px inputs/composer, `{rounded.md}` 10 px cards/buttons, `{rounded.lg}` 12 px panel/dialog. Avoid excessively pill-shaped controls. `rounded-full` is not a default; status may use a compact badge only if icon + text remain.

## Components

Owned shadcn React Aria primitives, audited locally (ADR-012). Do not use clickable `div` rows. Visual specs:

- **Button (primary)** — `{components.button-primary}` fill. Other variants (outline, ghost, destructive) keep independent text contrast and visible focus `{components.focus-ring}`.
- **Item card** — `{components.item-card}`: list/article grouping, `{spacing.3}` padding, `{colors.border}` edge. Contains lifecycle control, kind icon+label, wrapping preview, optional provenance, status text, context actions. Drag handle only when pointer reorder is enabled.
- **Focus ring** — `{components.focus-ring}` using `{colors.ring}`, ≥2 CSS px equivalent, ≥3:1 against adjacent background. Increase Contrast thickens/brightens; never remove without equal `:focus-visible` replacement.
- **Status indicator** — `{components.status-success}` / `status-warning` / `status-error` / `status-neutral`. Icon + label + accessible description on every state (`queued`, `copied`, `active`, `done`, `skipped`, permission health).
- **Quick panel** — `{components.quick-panel}` opaque surface. Header, toolbar, list, composer, footer remain one semantic column.
- **Composer** — `{components.composer}` multiline textarea (not `contenteditable`). Explicit Add. IME composition must not submit.

Native status item, menus, and file pickers stay AppKit. WebView does not restyle them.

## Do's and Don'ts

| Do | Don't |
|---|---|
| Opaque `{colors.background}` by default; Reduce Transparency keeps contrast | Rely on vibrancy/blur as the only surface |
| Warm bronze `{colors.primary}` / `{colors.accent}` for focus, selection, active | Invent a second brand color or copy Copper/Cooper chrome |
| System UI font at ≥14 px, resizable | Remote fonts, CDN, or size-only hierarchy |
| Status with icon, label, and name (A11Y-003) | Color-only queued/copied/error/permission |
| `{rounded.sm}`–`{rounded.lg}` (8–12 px) | Pill-shaped primary controls |
| `{spacing.3}` card padding; 24×24 CSS px targets | Hover-only or toast-only actions |
| Measure tokens in light/dark/Increase Contrast | Ship unmeasured hex from the concept PNG |
| Independent Bronze identity | Pixel-identical Copper/Cooper trade dress |
