# Bronze application

How the HIG lands in this repo. If this file and a live token disagree,
the live token file plus `contrast.test` win, and this file must be
updated in the same change.

## Stack constraints

- Tauri 2 WebView, `frontendDist` is unbundled `apps/desktop/src`.
  No npm motion library, no Vite CDN, no remote `@font-face`.
- Motion is CSS + WAAPI in existing modules (`control.mjs`), 180ms,
  `cubic-bezier(0.22, 1, 0.36, 1)`. Queue arrival and the copied-card
  ring use `--arrive` (280ms) in `chrome.css`.
- Event-tap callbacks never touch AX, DB, windows, clipboard, icons,
  or models.

## Live tokens

Defined in `apps/desktop/src/chrome.css` (four HTML surfaces) and
mirrored in `packages/ui/src/styles/globals.css`.

| Token | Role | macOS analogue |
| --- | --- | --- |
| `--text-micro` 0.6875rem | meta, chords | Subheadline ~11 |
| `--text-caption` 0.8125rem | body copy, secondary | Body 13 |
| `--text-body` 0.9375rem | titles in cards, window title | Title 3 15 |
| `--text-title` 1.25rem | rare display | Title 2 |
| `--control-h` 2rem | push buttons | compact Mac control |
| `--radius-control` 8px | fields, push buttons | rounded rect, not capsule |
| `--radius-card` 16px | queue cards / articles | modest standalone |
| `--radius-page-card` `8 + 20` | Settings / Library / Help cards | concentric with `--space-5` |
| `--radius-pill` 999px | status pills only | capsule |
| `--background` `#fafafa` | window | |
| `--foreground` `#18181b` | text | |
| `--border` / `--stroke` `#e4e4e7` / 8% hairline | grouping | |
| `--primary` mixed charcoal | one filled action | |

`packages/ui` contrast tests lock the zinc pairs. Do not paint live
chrome with DESIGN.md `#f7f4ef` / `#8c6239`.

## Type

`--font-sans` / `--font-display`: `-apple-system`, `SF Pro Text`,
`SF Pro Display`. Body weight 400, titles 590. `letter-spacing: 0` on
body and controls. Window `h1` may use `-0.02em`.

WKWebView can draw U+0020 too tight at some scales. A hair
`word-spacing` (≤0.03em) is a rendering workaround, not branding.
Do not restore 0.04em+ tracking or word-spacing as “polish”.

## Surfaces and elevation

Cards, articles, fieldsets, health rows: hairline + opaque fill, no
drop shadow. A light shadow is allowed only on floating layers (menu,
help tag).

Corners follow concentricity in `hig/macos.md`. Queue cards stay 16px.
Page-shell cards use `--radius-page-card` so a control in the corner
shares the card’s center. Grouped shortcut rows stay square. Status
pills may be capsules; Add / Copy / record fields stay rounded
rectangles.

## Controls

- Filled: Add, Copy, Export support bundle — one per view.
- Outlined: every other push button, including Reset, Show more,
  Library archive/import, Help, and Import. `.btn-ghost` uses the
  same stroke, padding, and radius as an unclassed `button`.
- Queue card extras: compact `.btn-icon` row under Copy, not a second
  filled toolbar. Hover uses the muted mix; Trash uses `--destructive`.
- Copy feedback: tip on the control. `#action-status` is a visually
  hidden live region (docs/05). Never dump status into the composer.

## Locale

HTML `data-i18n` fallbacks must match `packages/i18n/locales/en/app.json`.
Pseudo `en-XA` / `ar-XB` are generated from `en` via
`packages/i18n/scripts/pseudo.ts`. After adding keys, regenerate and
update `generated-message-ids.ts`.

## Provenance

Reimplement from behavior-level requirements. No Copper screenshots,
icons, or pixel-identical chrome.
