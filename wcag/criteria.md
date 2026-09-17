# Success criteria → Bronze

Normative titles follow [WCAG 2.2](https://www.w3.org/TR/WCAG22/).
This file is a Bronze mapping, not a substitute for the Recommendation.
How to Meet: https://www.w3.org/WAI/WCAG22/quickref/

**Support** here is the intended WebView implementation. It is not a
public conformance claim. Published rows live in
`docs/evidence/WCAG-22-INVENTORY.md`. VoiceOver and EN 301 549 stay
human.

4.1.1 Parsing is omitted (obsolete in 2.2).

## Perceivable

| SC | Level | Bronze |
| --- | --- | --- |
| 1.1.1 Non-text Content | A | Source icons `alt=""`. Brand mark `aria-hidden`. No informative images without text. |
| 1.2.1–1.2.9 Time-based media | A–AAA | N/A. No prerecorded or live audio/video. |
| 1.3.1 Info and Relationships | A | `h1` per window; section `h2`; item `h3`. Fieldsets, labels, lists. |
| 1.3.2 Meaningful Sequence | A | DOM order is the reading and tab order. |
| 1.3.3 Sensory Characteristics | A | Instructions never rely on color, shape, or position alone. |
| 1.3.4 Orientation | AA | Windows reflow in portrait and landscape. |
| 1.3.5 Identify Input Purpose | AA | No personal-data autocomplete fields in v1. N/A until those exist. |
| 1.3.6 Identify Purpose | AAA | Partial. Landmarks (`main`, `search`, `nav`) and buttons; no full purpose taxonomy. |
| 1.4.1 Use of Color | A | Status pills include text. Differentiate Without Color underlines status. |
| 1.4.2 Audio Control | A | N/A. No autoplaying audio. |
| 1.4.3 Contrast (Minimum) | AA | Normal text ≥4.5:1. Locked in `contrast.ts`. |
| 1.4.4 Resize Text | AA | 200% text resize without loss. A11Y-003. |
| 1.4.5 Images of Text | AA | UI text is real text. Status is not a PNG. |
| 1.4.6 Contrast (Enhanced) | AAA | Light/dark body and muted text ≥7:1. Pill text ≥7:1. |
| 1.4.7 Low or No Background Audio | AAA | N/A. No foreground speech with background audio. |
| 1.4.8 Visual Presentation | AAA | Partial. Line-height 1.5, prose ≤80ch, not justified. No user fg/bg picker. |
| 1.4.9 Images of Text (No Exception) | AAA | Supports. Chrome does not use images of text. |
| 1.4.10 Reflow | AA | 400% / 320 CSS px; ordinary UI is one scroll direction. |
| 1.4.11 Non-text Contrast | AA | Borders, icons, and focus ≥3:1 against adjacent colors. |
| 1.4.12 Text Spacing | AA | Author styles must not clip when the user sets WCAG spacing. |
| 1.4.13 Content on Hover or Focus | AA | Action tips are dismissible, hoverable, and persist until dismissed or invalid. |

## Operable

| SC | Level | Bronze |
| --- | --- | --- |
| 2.1.1 Keyboard | A | Every operation has a keyboard path. No timing-only gesture. |
| 2.1.2 No Keyboard Trap | A | Tab cycles. Escape closes the edit sheet and overflow. |
| 2.1.3 Keyboard (No Exception) | AAA | Supports on WebView chrome. Native status item is out of this claim. |
| 2.1.4 Character Key Shortcuts | A | No single-character shortcuts in the WebView. Global chords are remappable and do not fire while composing. |
| 2.2.1 Timing Adjustable | A | No timed interaction on the four surfaces. Double-tap gap is a setting (EN 5.8). |
| 2.2.2 Pause, Stop, Hide | A | Motion is short state feedback, not a moving ad. Reduce Motion wins. |
| 2.2.3 No Timing | AAA | Supports on WebView. Capture chord timing is optional, not required. |
| 2.2.4 Interruptions | AAA | Partial. Capture announcements are polite status, not forced focus. |
| 2.2.5 Re-authenticating | AAA | N/A. No authenticated session. |
| 2.2.6 Timeouts | AAA | N/A. No data-loss timeout. |
| 2.3.1 Three Flashes or Below Threshold | A | No flashing content. |
| 2.3.2 Three Flashes | AAA | Supports. Nothing flashes. |
| 2.3.3 Animation from Interactions | AAA | Supports. `prefers-reduced-motion` and `data-reduce-motion` disable motion. |
| 2.4.1 Bypass Blocks | A | Skip link is the first focusable control on each HTML window. |
| 2.4.2 Page Titled | A | Bronze / Bronze Settings / Bronze Library / Bronze Help. |
| 2.4.3 Focus Order | A | Matches visual order. Sheet restores trigger focus. |
| 2.4.4 Link Purpose (In Context) | A | Nav and help links are self-describing. |
| 2.4.5 Multiple Ways | AA | Partial. Four separate windows, not a multi-page site. Menu bar + Settings reach Help. |
| 2.4.6 Headings and Labels | AA | Headings and labels describe topic or purpose. |
| 2.4.7 Focus Visible | AA | `:focus-visible` 2px ring. |
| 2.4.8 Location | AAA | Partial. Window title plus `aria-current` on Library segments. |
| 2.4.9 Link Purpose (Link Only) | AAA | Supports on authored links (Archive / Trash / Search / Help). |
| 2.4.10 Section Headings | AAA | Supports. Settings, Help, Inbox, Library items use section headings. |
| 2.4.11 Focus Not Obscured (Minimum) | AA | Open overflow raises its card; sticky chrome does not cover focus. |
| 2.4.12 Focus Not Obscured (Enhanced) | AAA | Partial. Same mechanism; not proven for every scroll offset. |
| 2.4.13 Focus Appearance | AAA | 2 CSS px ring, ≥3:1 against adjacent and against unfocused. |
| 2.5.1 Pointer Gestures | A | No path-based gesture required. |
| 2.5.2 Pointer Cancellation | A | Buttons activate on up. Abort by leaving the control. |
| 2.5.3 Label in Name | A | Accessible name contains the visible text. |
| 2.5.4 Motion Actuation | A | No shake-to-act. |
| 2.5.5 Target Size (Enhanced) | AAA | Partial. Controls are 32×32 (`--control-h: 2rem`), not 44×44. Compact Mac HIG. |
| 2.5.6 Concurrent Input Mechanisms | AAA | Supports. Pointer, keyboard, and (human) VoiceOver are not locked out. |
| 2.5.7 Dragging Movements | AA | Reorder uses Move up / Move down. No drag requirement. |
| 2.5.8 Target Size (Minimum) | AA | Interactive targets ≥24×24. Status pills are text, not targets. |

## Understandable

| SC | Level | Bronze |
| --- | --- | --- |
| 3.1.1 Language of Page | A | `html lang` set; `apply-locale` updates lang/dir. |
| 3.1.2 Language of Parts | AA | Chrome follows the UI locale. Captured bodies are `lang="und" dir="auto"`. |
| 3.1.3 Unusual Words | AAA | N/A. No glossary feature; help uses plain words. |
| 3.1.4 Abbreviations | AAA | N/A. No abbreviation expansion mechanism. |
| 3.1.5 Reading Level | AAA | Partial. Help is short plain language; not measured to lower-secondary. |
| 3.1.6 Pronunciation | AAA | N/A. |
| 3.2.1 On Focus | A | Focus does not change context. |
| 3.2.2 On Input | A | Changing a field does not auto-submit or jump windows. |
| 3.2.3 Consistent Navigation | AA | Brand header is consistent. Library segments stay in one place. |
| 3.2.4 Consistent Identification | AA | Same action uses the same name (Copy, Reset, Export…). |
| 3.2.5 Change on Request | AAA | Supports. Context changes are user-initiated. |
| 3.2.6 Consistent Help | A | N/A. Help is not a repeated mechanism on every window. One Help window, opened from Settings and the menu bar. |
| 3.3.1 Error Identification | A | Composer and capture errors are text in `role="alert"` or status. |
| 3.3.2 Labels or Instructions | A | Every field has a visible or programmatically associated label. |
| 3.3.3 Error Suggestion | AA | Capture failures name the recovery (select text, grant AX, type). |
| 3.3.4 Error Prevention (Legal, Financial, Data) | AA | Partial. Export/import confirm via native pickers; no legal commit. |
| 3.3.5 Help | AAA | Partial. Labels and Help window; no per-field help control. |
| 3.3.6 Error Prevention (All) | AAA | Partial. Edit sheet can cancel; destructive trash is reversible from Library. |
| 3.3.7 Redundant Entry | A | N/A until a multi-step form asks the same data twice. |
| 3.3.8 Accessible Authentication (Minimum) | AA | N/A. No login. |
| 3.3.9 Accessible Authentication (Enhanced) | AAA | N/A. No login. |

## Robust

| SC | Level | Bronze |
| --- | --- | --- |
| 4.1.2 Name, Role, Value | A | Native HTML controls. Expanded/busy/current/invalid exposed. |
| 4.1.3 Status Messages | AA | `#action-status`, capture status, search count use `role="status"`. Errors use `role="alert"`. |

## AAA rows Bronze will not chase in v1

Do not add product scope to paint these green:

- All of 1.2 (media, sign language, audio description)
- 1.4.8 user-selected foreground and background colors
- 1.3.5/3.3.8/3.3.9 personal-data or authentication UI
- 2.5.5 44×44 on every compact Mac control
- 3.1.3–3.1.6 glossary / abbreviation / pronunciation tools
