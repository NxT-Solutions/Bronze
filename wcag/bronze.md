# Bronze application

How WCAG 2.2 lands in this repo. If this file and a live token
disagree, the live token file plus `contrast.test` win, and this file
must be updated in the same change.

## Claim and evidence

- WebView *target*: WCAG 2.2 AA (`docs/10`).
- AAA is an implementation stretch, not a public claim.
- Whole desktop: EN 301 549 clauses 5, 11, 12 plus Apple VoiceOver.
- Agents must not mark stories 3.9, 3.10, 5.5, or 9.3 done.
- Inventory: `docs/evidence/WCAG-22-INVENTORY.md`. Human gates:
  `docs/evidence/HUMAN-GATES.md`.
- Contrast math: `packages/ui/src/lib/contrast.ts` (relative luminance).
  Sampling is not a public AA/AAA claim.

## Stack constraints

- Tauri 2 WebView, `frontendDist` is unbundled `apps/desktop/src`.
  No axe-in-browser runtime, no remote audit service.
- Motion is CSS + WAAPI, ≤200ms. Reduce Motion wins.
- Event-tap callbacks never touch AX, DB, windows, clipboard, icons,
  or models.
- System SF only. No remote fonts or CDN (SEC-001).

## Live tokens

Defined in `apps/desktop/src/chrome.css` and mirrored in
`packages/ui/src/styles/globals.css`.

| Token | Light | Role | WCAG |
| --- | --- | --- | --- |
| `--background` | `#fafafa` | window | |
| `--foreground` | `#18181b` | text | 1.4.3 / 1.4.6 ≥7:1 |
| `--card` | `#ffffff` | surfaces | |
| `--muted-foreground` | `#52525b` | secondary text | ≥7:1 on muted/card |
| `--border` / `--input` / `--stroke` | `#e4e4e7` / 8% hairline | grouping | 1.4.11 Partial; Increase Contrast is the 3:1 path |
| `--success` | `#2f6f4f` | granted pill | white text ≥4.5:1 |
| `--warning` | `#8a4e0e` | warning pill | white text ≥4.5:1 |
| `--destructive` | `#b42318` | denied / trash | white text ≥4.5:1 |
| `--ring` | `#18181b` | focus | 2.4.7 / 2.4.13 |
| `--control-h` | `2rem` (32px) | push buttons | 2.5.8 AA; 2.5.5 AAA Partial |

Dark (`.dark` in globals; live desktop chrome is light):

| Token | Dark | WCAG |
| --- | --- | --- |
| `--muted-foreground` | `#a1a1aa` | ≥4.5:1 on `#27272a` |
| `--border` / `--input` | `#3f3f46` | HIG hairline; Increase Contrast is the 3:1 path |
| `--success` / `--warning` / `--destructive` | pastels with `#18181b` text | ≥7:1 |

Increase Contrast / `data-increase-contrast` forces black borders.
`prefers-reduced-motion` and `data-reduce-motion` kill animation.

Do not paint live chrome with DESIGN.md `#f7f4ef` / `#8c6239`.

## Focus and targets

- `:focus-visible` is `outline: 2px solid var(--ring); outline-offset: 2px`.
- Do not remove it without an equal replacement that still meets
  2.4.7, 2.4.11, 1.4.11, and 2.4.13.
- Interactive controls use `min-height` and `min-width` of
  `--control-h` (32px) — above 2.5.8’s 24px, below 2.5.5’s 44px.
  Queue icon-row and chip `.btn-icon` stay `1.5rem` (24px).
- Status pills are `<p>`, not targets.

## Skip, headings, live regions

- First focusable node on each HTML window is `.skip-link`.
- Queue `h2` is Inbox (`panel.section.active`) before `#queue`.
- Library list `h2` is Items (`library.heading.items`) before
  `#library-queue`.
- `#action-status` is visually hidden `role="status"`.
- Composer errors are `role="alert"`.

## Locale

HTML `data-i18n` fallbacks must match
`packages/i18n/locales/en/app.json`. After adding keys, regenerate
pseudo locales and `generated-message-ids.ts`.

## Provenance

Reimplement from behavior-level requirements. No Copper screenshots,
icons, or pixel-identical chrome.
