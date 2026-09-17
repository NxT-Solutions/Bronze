# POUR

Source: [WCAG 2.2](https://www.w3.org/TR/WCAG22/) Introduction and
guidelines, and the [WCAG Overview](https://www.w3.org/WAI/standards-guidelines/wcag/).
Thirteen guidelines sit under four principles. Conformance is judged
by the success criteria, not by the principle titles.

These are tools for weighing work. They do not replace `criteria.md`.

## Perceivable

Information and user interface components must be presentable to
people in ways they can perceive.

- Text alternatives for non-text content (1.1).
- Captions and alternatives for time-based media (1.2). Bronze ships
  no prerecorded or live media in v1; those criteria are N/A until
  someone adds video or audio.
- Adaptable structure: programmatically determined headings, lists,
  labels, and a meaningful sequence (1.3).
- Distinguishable: contrast, resize, reflow, spacing, hover/focus
  content, and no color-only status (1.4).

## Operable

User interface components and navigation must be operable.

- Full keyboard access and no trap (2.1).
- Enough time; Bronze has no session timeout on the queue (2.2).
- Nothing flashes more than three times in one second (2.3).
- Navigable: skip blocks, titled windows, focus order, headings,
  visible focus (2.4).
- Input modalities: no path-only gestures, pointer cancellation,
  label in name, target size (2.5).

## Understandable

Information and the operation of the user interface must be
understandable.

- Language of the page and of parts (3.1). Item bodies stay
  `lang="und" dir="auto"` because capture language is unknown.
- Predictable: no surprising context change on focus or input;
  consistent identification (3.2). Help lives on Settings only, so
  3.2.6 Consistent Help is N/A until Help is repeated.
- Input assistance: labels, identified errors, no inaccessible
  authentication puzzle (3.3). Bronze has no account.

## Robust

Content must be robust enough that a wide variety of user agents,
including assistive technologies, can interpret it.

- Name, role, value on every control (4.1.2).
- Status messages can be presented by AT without moving focus
  (4.1.3).
- 4.1.1 Parsing is obsolete in WCAG 2.2.

## Levels

- **A** — minimum. Missing an applicable A criterion is a defect.
- **AA** — Bronze WebView target (`docs/10`). Meet every applicable
  AA criterion on the four HTML surfaces.
- **AAA** — take it when a token or markup change is enough. Do not
  add a settings color picker, sign language, or 44×44 chrome that
  breaks compact Mac HIG just to paint a row green.

A later version of WCAG adds criteria; it does not rewrite older
ones (except 4.1.1 becoming obsolete). Prefer 2.2 resources.
