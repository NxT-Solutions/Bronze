# macOS foundations

Source: [HIG](https://developer.apple.com/design/human-interface-guidelines),
[Get started](https://developer.apple.com/design/get-started/), and the
platform pages for typography, layout, buttons, writing, motion, and
designing for macOS. Bronze ships only on Mac. iOS / visionOS / watchOS
notes below are context, not work to implement.

## Designing for macOS

People use a large, high-resolution display, often with other apps
open, at 1–3 feet. Inputs are keyboard, pointer, and VoiceOver. Windows
resize. Put depth in fewer nested levels, not more modes.

- Let windows resize; do not pack critical controls against the bottom
  edge (people slide windows off-screen).
- Menu bar and keyboard shortcuts are first-class, not optional.
- Hit targets stay usable with a pointer; 44×44 pt is the HIG floor
  for custom buttons. Compact Mac rows may be shorter when they are
  system-like list rows, but keep spacing so they do not collide.

## Typography

SF Pro is the macOS system font. Do not embed SF files. Do not add
tracking to SF Text in a running app — the system already adjusts
tracking per size. Display titles may use a slight negative tracking.

macOS built-in text styles (HIG table):

| Style | Weight | Size | Line height |
| --- | --- | --- | --- |
| Large Title | Regular | 26 | 32 |
| Title 1 | Regular | 22 | 26 |
| Title 2 | Regular | 17 | 22 |
| Title 3 | Regular | 15 | 20 |
| Headline | Bold | 13 | 16 |
| Body | Regular | 13 | 16 |
| Callout | Regular | 12 | 15 |
| Subheadline | Regular | 11 | 14 |
| Footnote / Caption | Regular–Medium | 10 | 13 |

Default body size on Mac is 13 pt; minimum 10 pt. Prefer Regular,
Medium, Semibold, Bold. Avoid Ultralight / Thin / Light.

Hierarchy comes from size, weight, and color — not from letter-spacing
hacks or drop shadows.

## Layout

Group related items with space, hairline separators, or a shared
background. Keep content distinct from controls.

Put the important thing first in reading order (leading / top; honor
RTL). Align edges. Progressive disclosure for overflow (Show more),
not a truncated mystery.

Give controls room. Unrelated actions must not sit on top of each
other. A text field at `width: 100%` next to a button in one row is a
layout defect.

macOS: keep critical information off the bottom strip of a window.

## Buttons

A button starts an instantaneous action. Style + label + role.

- One prominent (filled) button per view for the most likely action.
- Same-size siblings; distinguish with style, not mismatched heights.
- Press state is required. Disabled / busy must stay readable.
- Label with a verb, title-style capitalization on Mac push buttons.
- Trailing ellipsis when the button opens another window or needs more
  input (`Export…`).
- Never make a destructive action the filled primary.
- macOS help: at most one help control per window, usually
  bottom-trailing in Settings.

Menus are compact rows, not a second toolbar of pills.

## Lists and settings

Settings use grouped sections, short labels, and an explanation only
when the label is not enough. Describe the on-state; people infer off.

Rows: human title leading, value or status trailing. Never show raw
action IDs (`queue.copy`) as the only label.

Search filters the list; it does not replace it.

## Writing

Voice: calm, precise, local-first. Tone matches severity (permission
denied is direct; copy success is brief).

Be clear. Cut words. No jargon, no “we”, no “oops”. Errors say what
happened and what to do, next to the problem.

Empty states say the next action. Settings labels stay practical.
Placeholder text shows format or purpose, not decoration.

Every sentence lives in the locale catalog. No concatenation of
catalog fragments into a sentence.

## Motion

Motion supports, never overshadows. Brief and precise. Frequent
controls do not get a show every click.

People must be able to cancel or ignore motion. Reduce Motion turns
animations off. Motion is never the only channel for meaning (also
text, busy state, live region).

## Color and materials

Use color with meaning (destructive red, status green) and keep body
text on opaque surfaces at contrast ≥ 4.5:1 for the zinc pairs.

Liquid Glass is an Apple system material. A WebView may not fake it
with blur and translucency. Opaque card + hairline is the Bronze
fallback until a native material ships with an ADR and an opaque
fallback.

## Icons

Official app icons come from the macOS app bundle (`icon(forFile:)`),
never a hardcoded pack or SF Symbol stand-in for a third-party app.
SF Symbols are fine for *our* chrome if they are local — Bronze
currently uses type and a small brand mark, not a symbol font CDN.
