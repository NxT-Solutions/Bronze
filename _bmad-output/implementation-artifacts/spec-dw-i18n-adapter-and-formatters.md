---
title: 'DW: i18n adapter and formatters (DW-4, DW-5)'
type: 'chore'
created: '2026-08-27'
baseline_revision: '2a2d195f61b1107099936f89e0b2497e63e72aa5'
baseline_commit: '2a2d195f61b1107099936f89e0b2497e63e72aa5'
status: 'done'
review_loop_iteration: 0
followup_review_recommended: false
context:
  - _bmad-output/implementation-artifacts/1-6-i18n-catalogs-and-pseudo-locales.md
  - docs/11-i18n-localization.md
  - docs/18-adrs.md
  - docs/06-system-architecture.md
  - _bmad-output/implementation-artifacts/epic-1-context.md
  - AGENTS.md
  - packages/i18n/package.json
warnings: []
deferred:
  - summary: >-
      REACT_SIDE_EFFECT is specified as useTranslation() in a component; tests only assert getI18n() after createI18n because @bronze/i18n has no React runtime or renderer and Never forbids new dependency pins.
    evidence: |-
      packages/i18n/package.json has react-i18next but no react/react-dom; vitest environment is node; intent matrix names useTranslation in a component; getI18n().t is a singleton proxy.
    location: >-
      packages/i18n/test/locale.test.ts:132
    severity: medium
  - summary: >-
      Each createI18n() call does createInstance().use(initReactI18next), which rebinds react-i18next's module singleton to the latest instance.
    evidence: |-
      Tests already invoke createI18n multiple times (en, en-XA, zh-Hant-HK, pt-BR). getI18n() reflects the last init. Pre-existed before this story's adapter wrap.
    location: >-
      packages/i18n/src/create-i18n.ts:35
    severity: medium
---

<intent-contract>

## Intent

**Problem:** The @bronze/i18n package exposes a raw i18next instance from createI18n and its formatters.ts only implements NumberFormat, DateTimeFormat and PluralRules. ADR-014 and docs/11 require a Bronze localization adapter (typed t over MessageId) plus the remaining Intl formatters (RelativeTimeFormat, ListFormat, Collator, DisplayNames, Segmenter) before any UI binds to the raw surface.

**Approach:** Expand the cached formatter helpers to cover all required Intl constructors using the established per-locale+options cache key pattern. Change createI18n to return a narrow Bronze localization adapter exposing only a typed t(key: MessageId, opts?) instead of the raw I18n instance while preserving the ICU + react-i18next initialization side effects. Update internal tests and public barrel. No catalog or pseudo changes.

## Boundaries & Constraints

**Always:**
- createI18n returns only the adapter (no raw i18next type escapes from the call site for consumer code)
- t is typed exclusively over MessageId from generated-message-ids (no string literals for keys through the typed path)
- All formatters (existing and new) are cached by (locale, JSON-serialized opts); identical inputs yield identical instances
- No new user-facing English strings, sentence literals, or concatenation introduced in any source
- Existing catalog keys, RFC 4647 fallback, ICU behavior, validate, and pseudo generators remain unchanged
- User-supplied interpolation values remain isolated; no leakage into keys or catalog

**Block If:**
- (none — surface completion is fully specified by ADR-014, docs/11 §5, and the bundle ledger entries)

**Never:**
- Return or type any public createI18n result as raw i18next I18n
- Bind later UI code (including 6.1) to raw i18next
- Edit the deferred-work ledger or deferred-work.md
- Modify numbered story specs, stories.yaml, pnpm-lock.yaml, or dependency pins
- Add any network, remote resource, or non-deterministic behavior

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|---------------|----------------------------|----------------|
| TYPED_T_HAPPY | createI18n(), adapter.t('queue.count', { count: 2 }) | returns the catalog ICU string for active locale (e.g. "2 items") | typechecks only because key is in MessageId |
| TYPED_T_BAD_KEY | adapter.t('no.such.key' as any, {}) | TypeScript compile error at call site | prevents raw string keys |
| FORMATTER_CACHE_HIT | const a = getRelativeTimeFormat('en'); const b = getRelativeTimeFormat('en'); | a === b | single cached instance |
| NEW_FORMATTERS | call getRelativeTimeFormat, getListFormat, getCollator, getDisplayNames, getSegmenter with various opts | each returns its Intl.* instance, usable for .format etc. | |
| FALLBACK_T | createI18n({ lng: 'pt-BR' }), adapter.t('app.name') | 'Bronze' (falls to en) | |
| REACT_SIDE_EFFECT | createI18n() then useTranslation() from react-i18next in a component | hook returns working (untyped) t that matches catalog | init side-effect preserved |
| USER_ISOLATION | adapter.t('capture.source', { appName: '<<evil>>' }) | 'From <<evil>>' (value never treated as key) | |

</intent-contract>

## Code Map

- `packages/i18n/src/formatters.ts:1` -- Map cache typed to 3 Intl types + 3 getters; will extend union + add 5 getters (RelativeTimeFormat, ListFormat, Collator, DisplayNames, Segmenter) using same `prefix:locale:json(opts)` key pattern
- `packages/i18n/src/create-i18n.ts:20` -- async createI18n returns Promise<I18n> (raw); also exports I18nInstance alias and CreateI18nOptions; will define adapter interface + typed TFunction, return narrow adapter, keep raw instance only internally for .use(ICU).use(initReactI18next).init + fallback wiring
- `packages/i18n/src/index.ts:1` -- barrel: re-exports createI18n, I18nInstance, * from formatters (incl. old TFunction placeholder), generated-message-ids (MessageId), locale
- `packages/i18n/src/generated-message-ids.ts:4` -- MessageIds const + MessageId type + KNOWN list; already seeded; t will be typed over it
- `packages/i18n/test/locale.test.ts:2` -- imports createI18n, exercises via getFixedT + direct t calls on ICU, fallback, isolation; must migrate to adapter.t surface while keeping assertions
- `packages/i18n/src/locale.ts:25` -- computeFallbackChain (read-only for this change)
- `packages/i18n/locales/en/app.json:1` -- source of MessageIds (read)
- `docs/11-i18n-localization.md:70` -- explicitly lists the 8 Intl formatters required + cache rule
- `docs/06-system-architecture.md:506` -- lists Intl* required (subset)
- `_bmad-output/implementation-artifacts/1-6-i18n-catalogs-and-pseudo-locales.md:97` -- records the "formatters.ts -- re-export of Intl formatters cached by locale + t wrapper" and the two deferred items
- `packages/i18n/package.json:2` -- name @bronze/i18n; scripts for test/typecheck/validate (no code change)
- `packages/i18n/vitest.config.ts` and `tsconfig.json` -- test env (read-only)

## Tasks & Acceptance

**Execution:**
- `packages/i18n/src/formatters.ts` -- extend cache union and implement the five missing cached getters exactly mirroring the style, keying and casting of the three existing ones -- completes DW-4 + docs/11 §5
- `packages/i18n/src/create-i18n.ts` -- introduce `export type TFunction = (key: MessageId, opts?: Record<string, unknown>) => string;` and `export interface BronzeLocalizationAdapter { t: TFunction; }`; change `createI18n` to return `Promise<BronzeLocalizationAdapter>` (never the raw i18next instance); alias `export type I18nInstance = BronzeLocalizationAdapter;`; retain raw instance only as local var for side-effect initialization -- completes DW-5
- `packages/i18n/src/index.ts` -- export the new adapter type and TFunction (remove or update the placeholder from formatters); keep barrel minimal and complete
- `packages/i18n/test/locale.test.ts` -- update all createI18n consumers to destructure/use `.t` from the returned adapter; delete getFixedT usage; keep all existing behavioral assertions for plurals, fallback, isolation and pseudo
- `packages/i18n/src/formatters.ts` -- delete the "t wrapper placeholder" comment and untyped TFunction export (now supplied from create-i18n)

**Acceptance Criteria:**
- Given packages/i18n source after edit, when `pnpm --filter @bronze/i18n typecheck` executes, then exit code is 0 and MessageId is enforced at call sites
- Given the package, when `pnpm --filter @bronze/i18n test` executes, then exit code is 0 and ICU, fallback, isolation tests continue to pass
- Given the package, when `pnpm --filter @bronze/i18n validate` executes, then exit code is 0
- Given a consumer file importing { createI18n } from '@bronze/i18n', when assigning the result of createI18n() and calling .t with a literal not present in MessageId, then TypeScript reports an error
- Given formatters, when any of the five new getters is called (with or without options), then the correct Intl constructor type is returned and repeated calls with identical args return the cached instance (identity equality)
- createI18n's declared return type is BronzeLocalizationAdapter (or I18nInstance alias) and contains no reference to the raw i18next `I18n` type in public surface

## Spec Change Log

## Review Triage Log

## Design Notes

The adapter is intentionally a minimal surface (only .t) rather than a full proxy or subclass of i18next. This guarantees that UI code cannot accidentally depend on raw i18next methods while the .use(initReactI18next) side-effect still makes the standard react-i18next hooks function for components. The cache pattern is deliberately kept as a plain Map with JSON key (no WeakMap, no LRU) because the set of (locale, options) tuples in a desktop app is small and stable.

## Verification

**Commands:**
- `pnpm --filter @bronze/i18n typecheck` -- expected: exit 0
- `pnpm --filter @bronze/i18n test` -- expected: exit 0 (ICU, fallback, isolation, locale tests green)
- `pnpm --filter @bronze/i18n validate` -- expected: exit 0, no output on success
- `pnpm verify` -- expected: overall pass (includes the i18n filter tasks)

**Manual checks (if no CLI):**
- `rg -n 'createI18n.*:.*Promise.*i18next|from "i18next"' packages/i18n/src` -- only internal imports remain; public surface exports no raw I18n
- Inspect that MessageId is used in the t signature and that formatters.ts exports the five new functions

## Review Triage Log

### 2026-08-27 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 9 (high 0, medium 2, low 7)
- defer: 6
- reject: 5
- addressed_findings:
  - `[medium] [patch]` cache key JSON.stringify order-dependent (different key for equiv opts) — added cacheKey helper that sorts keys
  - `[medium] [patch]` DisplayNames ctor could throw without required options or on null — defaulted to {type:'language'} and normalized
  - `[low] [patch]` formatter tests only did instanceof, no .format/.of output — added format calls + assertions (relative, list, display)
  - `[low] [patch]` adapter surface matrix tests used expect(true) placeholder — replaced with typed t call + Object.keys narrow-surface assert
  - `[low] [patch]` adapter.t used `as string` without non-string guard — now String(r) fallback
  - `[low] [patch]` no JSDoc on new public types/getter — added brief JSDoc on TFunction, BronzeLocalizationAdapter, I18nInstance, getRelativeTimeFormat
  - `[low] [patch]` null/undefined opts could reach some ctors inconsistently — unified via cacheKey + ?? 
  - `[low] [patch]` I18nInstance alias had no explanatory comment — added comment
  - `[low] [patch]` isolation test still casts for runtime bad-key probe (compile surface not exercised negatively) — accepted as minimal for negative path

### 2026-08-27 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 6: (high 0, medium 1, low 5)
- defer: 0
- reject: 16
- addressed_findings:
  - `[medium] [patch]` REACT_SIDE_EFFECT test only asserted adapter shape — now asserts getI18n().t("app.name") === "Bronze" after createI18n
  - `[low] [patch]` DisplayNames cache key used caller opts while ctor defaulted `{type:"language"}` — key now derived from the same dnOpts; identity covered
  - `[low] [patch]` Collator/Segmenter only instanceof — added compare and segment assertions
  - `[low] [patch]` existing Number/DateTime/Plural getters had no identity assert after cacheKey change — added toBe
  - `[low] [patch]` cacheKey key-order independence untested — added reversed ListFormat opts identity
  - `[low] [patch]` TYPED_T_BAD_KEY test called a valid MessageId — added @ts-expect-error t("no.such.key")

### 2026-08-27 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 1: (high 0, medium 1, low 0)
- defer: 2: (high 0, medium 2, low 0)
- reject: 18
- addressed_findings:
  - `[medium] [patch]` getDisplayNames("en", {}) throws because empty opts lack required `type` — merge `{ type: "language", ...opts }` and assert identity with no-opts

## Auto Run Result

Status: done

Summary of implemented change: Follow-up review of the typed Bronze localization adapter (`createI18n` returns `{ t: TFunction }` over `MessageId`, not raw i18next) and the five remaining cached Intl getters. One patch: `getDisplayNames` now merges `{ type: "language", ...opts }` so an empty options object does not throw.

Files changed:
- `packages/i18n/src/create-i18n.ts` — narrow `BronzeLocalizationAdapter` with typed `t`; raw i18next kept internal
- `packages/i18n/src/formatters.ts` — eight cached Intl getters; DisplayNames defaults `type` even when `opts` is `{}`
- `packages/i18n/src/index.ts` — export adapter + `TFunction`
- `packages/i18n/test/locale.test.ts` — adapter.t, formatter identity/.format, `@ts-expect-error` bad key, getI18n side-effect, empty DisplayNames opts
- `_bmad-output/implementation-artifacts/spec-dw-i18n-adapter-and-formatters.md` — this spec

Review findings breakdown (this pass): patches applied 1 (medium); items deferred 2 (medium); items rejected 18.

Follow-up review recommendation: false (patched: high 0, medium 1, low 0; score `3 × 1 + 1 × 0 = 3`, threshold 5).

Verification performed:
- `pnpm --filter @bronze/i18n typecheck` — exit 0
- `pnpm --filter @bronze/i18n test` — exit 0, 29 tests passed
- `pnpm --filter @bronze/i18n validate` — exit 0
- `pnpm verify` — pass (biome warnings pre-existing in packages/ui; turbo typecheck/test/validate; cargo fmt/clippy/test; planning-checks PASS)

Residual risks:
- `useTranslation()` in a React component is not executed in this package (no React runtime; new deps forbidden); `getI18n()` is the proxy.
- Repeated `createI18n()` rebinds the react-i18next module singleton (pre-existing).
- Orchestrator-owned `_bmad-output/implementation-artifacts/deferred-work.md` was left untouched.

