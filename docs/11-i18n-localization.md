# Internationalization and localization

## 1. Objective

Internationalization is architecture, not post-launch translation. P0 ships complete English plus pseudo-locales and at least one reviewed RTL/CJK test path. Public locale claims require human translation and linguistic QA.

Standards: [BCP 47 / RFC 5646](https://www.rfc-editor.org/info/rfc5646/), [RFC 4647 language-tag lookup](https://www.rfc-editor.org/rfc/rfc4647.html), [ECMA-402 `Intl`](https://tc39.es/ecma402/), and [Unicode MessageFormat](https://www.unicode.org/reports/tr35/tr35-messageFormat.html).

## 2. Locale model

- Store canonical BCP 47 tag, e.g. `en`, `en-GB`, `nl-BE`, `ar`, `ja`; canonicalize and validate through `Intl.getCanonicalLocales`.
- Default follows macOS preferred languages and locale; user override applies without changing content.
- Fallback uses RFC 4647-style progressive lookup, preserving script and variant subtags before base language: `zh-Hant-HK` → `zh-Hant` → `zh` → `en` → visible missing-key error in development. Process macOS language priority entries in order; never jump directly from script-specific locale to base language.
- Language and regional formatting can be separate if user asks; P0 single locale override acceptable if documented.
- Persist neutral enums/UTC timestamps; never persist localized labels.
- Update `<html lang>` and `dir`; localize native menu/status/window/InfoPlist strings too. Live change preserves focused control by stable semantic ID, updates portal/menu/native strings before exposure, then announces completion once in new locale; no stale-language surface remains mounted.

## 3. Package structure

```text
packages/i18n/
  src/
    create-i18n.ts
    locale.ts
    formatters.ts
    generated-message-ids.ts
  locales/
    en/app.json
    en/errors.json
    en/help.json
    ar-XB/...             # bidi pseudo-locale
    en-XA/...             # expansion/accent pseudo-locale
  scripts/
    extract.ts
    validate.ts
    pseudo.ts
```

Use react-i18next/i18next behind Bronze localization adapter with i18next ICU integration, per ADR-014. Catalog validation must check missing/extra keys, placeholder names/types, plural/select branches, markup allowlist, duplicate English values needing context, and forbidden raw strings.

## 4. Message design

- Stable semantic IDs: `capture.result.noSelection`, not English text as key.
- Full sentences, translator context, feature/character limit note where real.
- ICU plural/select for counts/gender-like grammatical variation.
- No string concatenation, manual plurals, interpolated punctuation, or assuming word order.
- Interpolations are escaped text. Rich messages use small approved semantic component map, never HTML from catalog.
- Separate visual glyph from spoken shortcut phrase.

Good:

```json
{
  "queue.selectionCount": "{count, plural, =0 {No items selected} one {# item selected} other {# items selected}}",
  "capture.saved": "Captured to {sectionName}",
  "shortcut.spoken": "{modifiers} plus {key}"
}
```

Bad:

```ts
t("capturedTo") + " " + sectionName
```

User-supplied section/item/app names remain untranslated and use bidi isolation.

## 5. Formatting

Use `Intl.DateTimeFormat`, `RelativeTimeFormat`, `NumberFormat`, `ListFormat`, `Collator`, `DisplayNames`, `Segmenter`, and PluralRules. Cache formatters per locale/options.

- Store UTC epoch milliseconds; display local timezone.
- Absolute timestamp available alongside relative “3 minutes ago.”
- Sort user-visible strings with locale collator; never use locale sort for stable DB order/export/checksum.
- Search indexing has explicit locale/Unicode behavior and preserves original text.
- Grapheme boundaries drive cursor-adjacent truncation/counts; do not split surrogate pair/combining/emoji cluster.
- Byte limit and user-facing grapheme/character guidance are distinct.

## 6. RTL and bidi

- Use CSS logical properties (`margin-inline-start`, `inset-inline-end`), not left/right, except physical screen-edge settings.
- Components inherit `dir`; portal roots explicitly receive it.
- Mirror directional navigation icons only when meaning is spatial, not universal symbols or shortcut glyphs.
- Horizontal keyboard semantics follow platform/component convention; document reorder behavior in RTL.
- User content uses `dir="auto"`, `unicode-bidi: plaintext` or isolation as tested.
- Wrap interpolated user content with isolation; protect punctuation from bidi spillover.
- Physical “left edge/right edge” settings use localized physical concepts and do not silently mirror stored choice.
- Test mixed Arabic/English code, URLs, numbers, emoji, and filenames.

Follow current [shadcn RTL guidance](https://ui.shadcn.com/docs/rtl), but audit each owned component, portal, animation, chart, and icon.

### 6.1 User-content language

- Each item stores `contentLanguage` as canonical BCP 47 or `und`. Capture and manual creation default to `und` unless user explicitly assigns language.
- Bronze exposes searchable, keyboard-accessible language override and never silently detects language. Unknown remains visible as “Not specified” rather than inferred from UI locale.
- Render each item body/editor/preview with its own `lang` value, `dir="auto"`, and tested bidi isolation. Root UI `lang` must not leak onto Japanese, Arabic, or mixed captured content.
- Revisions, backup, deterministic export, and import preserve field. Changing it never translates, normalizes, or rewrites body text.

## 7. Input methods and editing

- Track composition events. Enter/shortcut does not submit while `isComposing` or key code indicates composition process.
- Do not normalize/trim on input or blur.
- Test Japanese Kana/Kanji, Chinese Pinyin, Korean 2-set, Indic composition, dead keys, combining marks, emoji picker, dictation, and RTL selection.
- Keyboard handler uses `event.key` for semantic intent and `event.code` only when physical binding explicitly selected.
- Avoid `contenteditable` in MVP; textarea has more predictable IME and accessibility behavior.
- Copy/export preserves original code points and newlines.

## 8. Shortcut localization

Stored shortcut schema:

```ts
type Shortcut = {
  action: ActionId
  scope: "global" | "app" | "panel" | "editor"
  trigger: "chord" | "modifierDoubleTap"
  modifiers?: Array<"Command" | "Option" | "Control" | "Shift" | "Fn">
  keyMeaning?: { kind: "logical"; key: string } | { kind: "physical"; code: string }
  modifierSide?: "either" | "left" | "right" | "same"
  timingMs?: number
}
```

- Display macOS glyphs visually, e.g. `⌥⇧Space`, plus localized spoken/accessibility string, e.g. “Option plus Shift plus Space.”
- Recorder announces pressed modifiers/key without swallowing VoiceOver/system chords.
- Explain logical vs physical key choice for AZERTY/QWERTZ/Dvorak/Colemak.
- Detect collision using semantic normalized form and current keyboard source. Keep old binding if registration fails.
- Do not assume `Cmd` and `Ctrl` equivalent.

## 9. Pseudo-locales

- `en-XA`: accents and expands text at least 40%, preserves placeholders, adds boundary markers.
- `ar-XB`: mirrored/bidi-stressed text with isolates and numerals.
- Optional `zz-ZZ`: extreme long labels and missing-glyph corpus.

Pseudo-locales available in development and screenshot CI. They must cover WebView and native menu strings. CI fails clipping/unreachable actions in critical screens where screenshot/layout assertions are stable.

## 10. Translation workflow

1. Developer adds English message with context and test.
2. Extract/validate catalogs in CI.
3. Freeze strings for release candidate.
4. Translator works from source, screenshots, glossary, and character constraints.
5. Linguist reviews in running signed build.
6. Native and WebView coverage report reaches 100% for advertised locale.
7. Functional QA checks plural branches, shortcuts, RTL, IME, search, export, permission help.
8. Accessibility reviewer checks translated names/instructions and VoiceOver pronunciation.

Machine translation may seed internal preview only; never advertise locale from unreviewed output.

## 11. Glossary decisions

Maintain `research/glossary.md` during implementation. Terms needing stable context: capture, selection, queue, section, prompt, note, copied, active, done, skipped, restore, Input Monitoring, Accessibility, protected field, output profile, double tap, chord. “Capture” must not imply screenshot/recording in translation.

## 12. CI gates

- no hard-coded user-visible strings in TSX/Rust/Swift except approved technical values;
- key and placeholder parity;
- plural completeness;
- no invalid BCP 47 tags;
- English, en-XA, ar-XB build and core tests;
- RTL static and interaction smoke;
- IME composer test confirms Enter does not submit during composition;
- locale-independent JSON export golden tests;
- native strings coverage — menu.status.*, panel.quick.title, and InfoPlist CFBundleDisplayName/CFBundleName resolve through the same `app.json` glossary as WebView (`en`, `en-XA`, `ar-XB`); RTL chrome smoke keeps a physical edge; item cards use per-item `lang` plus `dir="auto"` (I18N-001/002/003/004, G-06);
- a11y names remain meaningful under longest locale.

## 13. Initial locale rollout

Recommended sequence after English architecture stabilizes: Dutch (user region), German (expansion/compounds), Arabic (RTL), Japanese (IME/CJK), then broader demand. Selection should follow translator availability and QA capacity, not raw string generation ability.
