# Surfaces

Four HTML windows share `chrome.css`. Native titles stay “Bronze”,
“Bronze Settings”, “Bronze Library”, “Bronze Help” (2.4.2).

## Queue — `apps/desktop/src/index.html`

Purpose: capture in, act on the current item.

- Skip link → `#composer-body`.
- `h1` Bronze, `h2` Inbox, item `h3`.
- Composer: label for the textarea, one filled Add, `role="alert"`
  on add failure.
- Capture and copy results: visually hidden `role="status"`. Visible
  copy feedback is the action tip on the button (1.4.13).
- Empty state is a sentence that names the next action.
- Item: title → body (pre-wrap) → compact Show more `button` when
  the preview overflows (`aria-expanded`, catalog name) → source +
  decorative icon → Copy + icon row. Each icon is a real `button`
  with catalog `aria-label`; the tip is `aria-hidden` and matches.
  Not `role="menu"`.
- Edit is a modal `<dialog>` with name, Escape, and focus restore.
  No `window.prompt`.
- Move up / Move down replace drag (2.5.7).
- Complete, Skip, Move, and Trash do not require motion to understand.
  Reduce Motion / `data-reduce-motion` updates the list immediately
  (2.3.3). This is not a WCAG claim.
- A pending title uses the catalog string “Writing title…” as the
  accessible name. The spinner is decorative (`aria-hidden`). Reduce
  Motion shows that string with no fade and no spinner animation
  (2.2.2, 2.3.1, 2.3.3). Information is not the spinner alone (1.1.1,
  1.4.1). This is not a WCAG claim.
- A new card may ease existing cards down to make room, then slide in
  from the inline-start edge and show a temporary `--ring` pulse
  (three cycles in 1s, then gone). The pulse is not the focus
  indicator. The sort button is a real control whose accessible name
  is the next order and whose description is the current order
  (`settings.field.queueSort.newest` / `.oldest`) via the owned tip
  (1.4.13). Changing sort collapses the loaded page into the first
  card, then expands the new first page. The first successful
  first-page load expands only, with no insert slide. Reduce Motion,
  `data-reduce-motion`, and `data-motion="reduce"` skip the make-room
  translate, the slide, the eased scroll, the pulse, the icon
  transform, the sort collapse/expand, the first-page expand, and the
  body Show more open/close. The label and `aria-expanded` still
  change. The card is in the list either way (2.2.2, 2.3.1, 2.3.3).
  This is not a WCAG claim.

## Settings — `apps/desktop/src/settings.html`

Purpose: data, privacy, permission truth, shortcut inventory.

- Skip link → `#settings-search`.
- `h1` Settings. Permission health, Shortcuts, and Export preview
  are `h2`. Fieldset legends label groups (1.3.1).
- Search settings filters groups; it does not hide the page chrome.
- Every field has a label. Reset sits beside the field, not as the
  only name.
- Permission status is a text pill plus why-text. Color is not the
  only signal (1.4.1).
- Help opener is a trailing outlined button, not a second primary. One Help
  surface (3.2.6 stays N/A).

## Library — `apps/desktop/src/library.html`

Purpose: find and manage stored items.

- Skip link → `#library-search`.
- `h1` Library. Segment nav (Archive / Trash) is labelled by that
  heading.
- `h2` Items before the list. Empty-state `h2` stays when the list
  is empty.
- Header search field + toolbar `role="status"` count.
- Same item chrome as Queue, without row actions.
- Footer tools are secondary. Export… / Import… use an ellipsis.

## Help — `apps/desktop/src/help.html`

Purpose: what Bronze is, how to capture, export a local bundle.

- Skip link → `#help-about`.
- `h1` Help, `h2` About Bronze, `h2` Diagnostics preview.
- Prose ≤80ch, line-height 1.5, not justified (1.4.8 Partial).
- One filled control: Export Support Bundle….
- No automatic upload (`data-automatic-upload="false"`).

## Shared defects to refuse

- Clickable `div` or icon-only control without a name
- `aria-label` that does not contain the visible text
- Painting default chrome at 3:1 just to satisfy 1.4.11 (use Increase Contrast)
- Body or muted text below 4.5:1 (AA) or, where tokens allow, 7:1
- Removing `:focus-visible` or using `outline: none` without a
  replacement
- Dumping status into the composer
- `role="menu"` without full menu keyboard behavior
- Claiming WCAG AA/AAA or VoiceOver from a screenshot or token test
- Treating last-letter screenshot smear as the catalog
