# Surfaces

Four HTML windows share `chrome.css`. Native titles stay “Bronze”,
“Bronze Settings”, “Bronze Library”, “Bronze Help”.

## Queue — `apps/desktop/src/index.html`

Purpose: capture in, act on the current item.

- Composer first: labeled textarea, one filled Add.
- Empty state names the next action (capture or type).
- Item: title (15/590) → body (13, pre-wrap, lists) → source + official
  icon → Copy (filled) + compact icon row (Complete, Skip, Edit, Move
  up, Move down, Trash).
- Collapse is max-height + pre-wrap, not `-webkit-line-clamp`.
- Enter animation is opacity only (`bronze-enter`, `--ease`).
- Complete / Skip / Trash leave the list: height collapse
  (`grid-template-rows` 1fr→0fr) plus fade, then the node is removed.
  Complete slides slightly up. Skip fades. Trash shrinks. Tokens are
  `--duration` and `--ease`.
- Move up / down is one FLIP `translateY` shot on the card and
  neighbors (`--duration`, `--ease-out`). Copy, Edit, and icon hover
  do not move.
- Reduce Motion (`prefers-reduced-motion: reduce` or
  `data-reduce-motion`) skips slide and collapse. The list updates
  immediately. Motion is never required to understand the action.

## Settings — `apps/desktop/src/settings.html`

Purpose: data, privacy, permission truth, shortcut inventory.

- Search settings filters groups; it does not hide the page chrome.
- Grouped fieldsets (Data, Privacy) with Reset as ghost.
- Permission health: why-text leading, status pill trailing, alternative
  in muted caption, Retest when useful. Screen Recording stays Not used.
  The section names the running app copy. Opening System Settings for
  Accessibility or Input Monitoring also reveals that copy in Finder.
- Shortcuts: section title + live status (only Shift double-tap is
  live). Record field and Skip test sit on one row without overlap.
- Registry rows: localized action name leading, assignment trailing.
  Raw IDs stay on `data-action` only.
- Help opener is an outlined button, trailing, not a second primary.

## Library — `apps/desktop/src/library.html`

Purpose: find and manage stored items. Read-only archive relative to
the live queue.

- Header matches Settings: brand + compact `chrome-search` (icon,
  placeholder, clear). Search is a field, not a third page.
- Segment: Archive / Trash. Count (`N items`) trails on the same
  toolbar row.
- Empty state is a quiet caption, hidden when the list is non-empty.
- Same item chrome as Queue, without row actions (no silent drop of
  body or source).
- Footer tools use the outlined button. Do not fill every tool.

## Help — `apps/desktop/src/help.html`

Purpose: what Bronze is, how to capture, export a local bundle.

- Prose stack: heading, then short paragraphs with vertical gap.
- Diagnostics preview is secondary (mono caption).
- One filled control: Export support bundle (ellipsis meaning is
  already in “Export … bundle”).
- No automatic upload (`data-automatic-upload="false"`).
- One help surface; do not add a second help button on this window.

## Shared defects to refuse

- Overlapping Skip / Reset on a 100% wide input
- Raw `app.togglePanel` / `queue.copy` as the visible label
- Composer used as a status dump
- Drop shadows as the only hierarchy
- Equal filled buttons in a row
- Clickable `div`
- Claiming the screenshot’s last-letter smear is the catalog
