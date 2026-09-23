# WCAG 2.2 inventory (WebView)

This is an implementation inventory for the four HTML surfaces
(Queue, Settings, Library, Help). It is **not** a public
conformance claim, ACR, or VPAT. Do not quote it as “Bronze
conforms to WCAG 2.2 AA” or “AAA”.

- Standard: [WCAG 2.2](https://www.w3.org/TR/WCAG22/) (12 December 2024)
- Scope: `apps/desktop/src` WebView chrome only
- Out of scope here: native menu bar, status item, TCC dialogs, VoiceOver
- AT, Voice Control, Switch Control, and an independent audit are
  optional checklist rows in `HUMAN-GATES.md`
- Token math: `packages/ui/src/lib/contrast.ts` (A11Y-003). Sampling
  is not a public AA/AAA claim.

Results:

- **Supports** — implemented on the WebView with automated or
  inspectable evidence. AT not yet used to confirm.
- **Partial** — implemented in part, or needs human AT / a product
  capability that is not in v1.
- **N/A** — no content or feature that triggers the criterion.
- **Does not support** — applicable and missing. None listed for
  WebView AA after this pass.

4.1.1 Parsing is omitted (obsolete in WCAG 2.2).

## A

| SC | Result | Evidence |
| --- | --- | --- |
| 1.1.1 Non-text Content | Supports | Source icons `alt=""`. Brand mark `aria-hidden`. |
| 1.2.1 Audio-only and Video-only (Prerecorded) | N/A | No media. |
| 1.2.2 Captions (Prerecorded) | N/A | No media. |
| 1.2.3 Audio Description or Media Alternative (Prerecorded) | N/A | No media. |
| 1.3.1 Info and Relationships | Supports | `h1` / `h2` / `h3`, labels, fieldsets, lists. AT untested. |
| 1.3.2 Meaningful Sequence | Supports | DOM order matches visual order. |
| 1.3.3 Sensory Characteristics | Supports | Instructions are not color- or position-only. |
| 1.4.1 Use of Color | Supports | Status pills include text; Differentiate Without Color underlines status. |
| 1.4.2 Audio Control | N/A | No autoplay audio. |
| 2.1.1 Keyboard | Supports | Controls are native buttons, links, fields, dialog. Global capture has menu/composer alternatives. |
| 2.1.2 No Keyboard Trap | Supports | Escape closes edit sheet and dismisses icon tips. |
| 2.1.4 Character Key Shortcuts | Supports | No single-character WebView shortcuts. |
| 2.2.1 Timing Adjustable | Supports | No timed WebView interaction. Double-tap is optional. |
| 2.2.2 Pause, Stop, Hide | Supports | Motion is short state feedback; Reduce Motion wins. |
| 2.3.1 Three Flashes or Below Threshold | Supports | No flashing content. |
| 2.4.1 Bypass Blocks | Supports | `.skip-link` is first focusable on each window. |
| 2.4.2 Page Titled | Supports | Distinct titles on the four windows. |
| 2.4.3 Focus Order | Supports | Source order. Sheet restores trigger focus. AT untested. |
| 2.4.4 Link Purpose (In Context) | Supports | Archive / Trash / Search / Help names. |
| 2.5.1 Pointer Gestures | Supports | No path-only gesture. |
| 2.5.2 Pointer Cancellation | Supports | Activation on up. |
| 2.5.3 Label in Name | Supports | Visible text is the accessible name. Queue icon-row names are catalog `aria-label`; tips are `aria-hidden` and match (`a11y.test.mjs`). |
| 2.5.4 Motion Actuation | Supports | No device-motion actions. |
| 3.1.1 Language of Page | Supports | `html lang`; `apply-locale` updates lang/dir. |
| 3.2.1 On Focus | Supports | Focus does not change context. |
| 3.2.2 On Input | Supports | Fields do not auto-submit. |
| 3.2.6 Consistent Help | N/A | Help is not repeated on every window. |
| 3.3.1 Error Identification | Supports | `role="alert"` / status text. |
| 3.3.2 Labels or Instructions | Supports | Labels associated with fields. |
| 3.3.7 Redundant Entry | N/A | No multi-step form that re-asks data. |
| 4.1.2 Name, Role, Value | Partial | Native HTML roles. VoiceOver confirmation is 9.3. |
| 4.1.3 Status Messages | Supports | `role="status"` / `role="alert"`. AT untested. |

## AA

| SC | Result | Evidence |
| --- | --- | --- |
| 1.2.4 Captions (Live) | N/A | No live media. |
| 1.2.5 Audio Description (Prerecorded) | N/A | No media. |
| 1.3.4 Orientation | Supports | Windows reflow; no orientation lock. |
| 1.3.5 Identify Input Purpose | N/A | No personal-data autocomplete fields. |
| 1.4.3 Contrast (Minimum) | Supports | Light/dark text pairs ≥4.5:1 (`contrast.test`). |
| 1.4.4 Resize Text | Partial | Layout aims at 200% (A11Y-003). Human zoom matrix is 9.3. |
| 1.4.5 Images of Text | Supports | UI is real text. |
| 1.4.10 Reflow | Partial | Column layout and overflow tests exist. 400%/320 human matrix is 9.3. |
| 1.4.11 Non-text Contrast | Partial | Focus ring ≥3:1. Default grouping is HIG hairline (`#e4e4e7` / 8% mix). Increase Contrast / `prefers-contrast: more` is the 3:1 border path. |
| 1.4.12 Text Spacing | Partial | Author styles do not clip at default. User-override matrix is 9.3. |
| 1.4.13 Content on Hover or Focus | Supports | Copy action tip on the control; icon-row tips on hover and `:focus-visible`; Escape dismisses; `#action-status` is SR-only. |
| 2.4.5 Multiple Ways | Partial | Four windows plus menu bar, not a multi-page site. |
| 2.4.6 Headings and Labels | Supports | Inbox / Items / Settings groups / Help sections. |
| 2.4.7 Focus Visible | Supports | 2px `:focus-visible` ring. |
| 2.4.11 Focus Not Obscured (Minimum) | Supports | Hover or focus on a card icon raises that card. |
| 2.5.7 Dragging Movements | Supports | Move up / Move down. No drag requirement. |
| 2.5.8 Target Size (Minimum) | Supports | Push buttons `min 2rem` (32px). Queue icon-row and chip `.btn-icon` use `1.5rem` (24px). Pills are not targets. |
| 3.1.2 Language of Parts | Supports | Chrome follows UI locale. Language switcher options use endonyms and option `lang`. Bodies are `lang="und" dir="auto"`. |
| 3.2.3 Consistent Navigation | Supports | Shared brand header; Library segments stay put. |
| 3.2.4 Consistent Identification | Supports | Same actions keep the same names. |
| 3.3.3 Error Suggestion | Supports | Capture failures name a recovery. |
| 3.3.4 Error Prevention (Legal, Financial, Data) | Partial | Native pickers for export/import; no legal transaction. |
| 3.3.8 Accessible Authentication (Minimum) | N/A | No login. |

## AAA

| SC | Result | Evidence |
| --- | --- | --- |
| 1.2.6 Sign Language (Prerecorded) | N/A | No media. |
| 1.2.7 Extended Audio Description (Prerecorded) | N/A | No media. |
| 1.2.8 Media Alternative (Prerecorded) | N/A | No media. |
| 1.2.9 Audio-only (Live) | N/A | No media. |
| 1.3.6 Identify Purpose | Partial | Landmarks and buttons; no full purpose taxonomy. |
| 1.4.6 Contrast (Enhanced) | Partial | Light body/muted ≥7:1. Status pills stay HIG colors (≥4.5:1). |
| 1.4.7 Low or No Background Audio | N/A | No speech over audio. |
| 1.4.8 Visual Presentation | Partial | Line-height 1.5, prose ≤80ch, not justified. No user fg/bg picker. |
| 1.4.9 Images of Text (No Exception) | Supports | No images of text. |
| 2.1.3 Keyboard (No Exception) | Partial | WebView chrome is keyboard-operable. Native chrome is 9.3. |
| 2.2.3 No Timing | Supports | WebView has no required timing. |
| 2.2.4 Interruptions | Partial | Status is polite; capture can announce while working. |
| 2.2.5 Re-authenticating | N/A | No session. |
| 2.2.6 Timeouts | N/A | No data-loss timeout. |
| 2.3.2 Three Flashes | Supports | Nothing flashes. |
| 2.3.3 Animation from Interactions | Supports | Reduce Motion / `html[data-motion]` / `data-reduce-motion`. |
| 2.4.8 Location | Partial | Window title plus Library `aria-current`. |
| 2.4.9 Link Purpose (Link Only) | Supports | Authored links are self-describing. |
| 2.4.10 Section Headings | Supports | Section `h2` on all four windows. |
| 2.4.12 Focus Not Obscured (Enhanced) | Partial | Same as 2.4.11; not proven at every scroll offset. |
| 2.4.13 Focus Appearance | Supports | 2 CSS px ring at ≥3:1. |
| 2.5.5 Target Size (Enhanced) | Partial | 32×32 compact Mac controls, not 44×44. |
| 2.5.6 Concurrent Input Mechanisms | Partial | Pointer and keyboard work. AT lock-out untested (9.3). |
| 3.1.3 Unusual Words | N/A | No glossary feature. |
| 3.1.4 Abbreviations | N/A | No abbreviation mechanism. |
| 3.1.5 Reading Level | Partial | Help is short; not measured to lower-secondary. |
| 3.1.6 Pronunciation | N/A | No pronunciation mechanism. |
| 3.2.5 Change on Request | Supports | Context changes are user-initiated. |
| 3.3.5 Help | Partial | Labels plus Help window; no per-field help. |
| 3.3.6 Error Prevention (All) | Partial | Edit can cancel; trash is reversible from Library. |
| 3.3.9 Accessible Authentication (Enhanced) | N/A | No login. |

## Known gaps that stay open

- VoiceOver, Voice Control, Switch Control, Full Keyboard Access (9.3)
- 200%/400% human reflow matrix (A11Y-003 / 9.3)
- Independent ACR review (DG-08)
- 2.5.5 44×44 vs compact Mac HIG
- Default grouping hairlines vs 1.4.11 (Increase Contrast is the 3:1 path)
- 1.4.8 user-selected foreground and background
- Native status item / menu bar (EN 301 549, not this inventory)
