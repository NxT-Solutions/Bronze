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
- The queue title sits in a reserved two-line slot (`min-height` 2lh,
  clamp + ellipsis). A later generated title swaps in place without
  changing card height. Reduce Motion adds no title motion.
- A new card inserted at the loaded edge first makes room: the current
  first card (newest-first) or last card (oldest-first, last page
  only) takes `is-make-room` and the existing stack eases one card
  height over `--arrive-room` (320ms). Then the new card slides in from
  the
  inline-start edge (`is-slide-in`, `--arrive` 380ms, opacity 0 on the
  first frame). RTL slides from that same inline-start edge. Only the
  new card takes the side-slide. Newest-first eases the scrollport to
  the top of the loaded page over `--arrive-scroll` (340ms) across
  animation frames. Oldest-first makes room at the loaded end only
  when that last page is loaded, and leaves the scrollport where it
  is. A later refresh updates the same row. After both shots, motion
  classes are gone and no transform or margin remains. The list itself
  does not take `is-slide-in`.
- `#queue-sort-toggle` is a real button between the composer and the
  first card, inline-end. Its glyphs flip with `--sort-flip` (200ms)
  so newest vs oldest is visible. Hover and `:focus-visible` show the
  current order (`settings.field.queueSort.newest` / `.oldest`) on the
  owned `icon-tip`; `aria-describedby` carries the same key. Changing
  sort reloads the first page: loaded cards collapse on the block axis
  into the current first card (`is-sort-collapse`, `--sort-collapse`
  220ms), hide, then the new first card expands from that slot and the
  rest unfurl (`is-sort-expand`, `--sort-expand` 280ms). The list does
  not side-slide as one block. Reduce Motion swaps the icon and the
  list instantly. The pulse is under 3Hz.
- The new card, and a card just copied, can show a temporary 2px
  `--ring` outline at low opacity, offset just outside `--radius-card`.
  The outline fades three times (`bronze-copy-ring`, `--ring-pulse`
  1000ms) and is gone when the pulse ends. It is not the keyboard focus
  ring. `:focus-visible` stays on the focused control. Visible copy
  feedback is the action tip.
- Complete / Skip / Trash leave the list: height collapse
  (`grid-template-rows` 1fr→0fr) plus fade, then the node is removed.
  Complete slides slightly up. Skip fades. Trash shrinks. Tokens are
  `--duration` and `--ease`.
- Move up / down is one FLIP `translateY` shot on the card and
  neighbors (`--duration`, `--ease-out`). Copy, Edit, and icon hover
  do not move.
- Reduce Motion (`prefers-reduced-motion: reduce`,
  `data-reduce-motion`, or `data-motion="reduce"`) skips the make-room
  translate, the slide, the scroll ease, the pulse, the sort-icon
  transform, the sort collapse/expand, and collapse. The card appears
  in its final place with no arrival outline. Motion is never required
  to understand the action. `data-motion="full"` plays the entrance
  when Play animations is on.

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
