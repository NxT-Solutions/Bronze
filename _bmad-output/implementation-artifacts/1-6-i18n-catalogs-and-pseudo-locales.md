---
title: 'Story 1.6: i18n catalogs and pseudo-locales'
type: 'feature'
created: '2026-08-27'
status: 'done'
review_loop_iteration: 0
followup_review_recommended: true
context:
  - _bmad-output/implementation-artifacts/epic-1-context.md
  - docs/18-adrs.md
  - docs/11-i18n-localization.md
  - docs/06-system-architecture.md
  - docs/03-prd.md
  - AGENTS.md
  - _bmad-output/implementation-artifacts/1-2-scaffold-pnpm-turbo-workspace.md
  - _bmad-output/implementation-artifacts/1-5-empty-tauri-app-with-csp-deny.md
warnings: []
deferred:
  - summary: >-
      formatters.ts only covers NumberFormat/DateTimeFormat/PluralRules; docs/11 §5 and arch require RelativeTimeFormat, ListFormat, Collator, DisplayNames, Segmenter too
    evidence: |-
      no other callers yet; story focused on catalog+validate+fallback gate; full formatters belong with UI consumption stories
    location: >-
      packages/i18n/src/formatters.ts:1
    severity: low
  - summary: >-
      no Bronze adapter wrapper or typed t() facade exported; createI18n returns raw i18next instance
    evidence: |-
      ADR-014 and docs/11 call for "Bronze localization adapter"; later UI stories will need stable surface
    location: >-
      packages/i18n/src/create-i18n.ts:37
    severity: medium
  - summary: >-
      extract script and generated-message-ids regeneration not wired; manual seed only
    evidence: |-
      story non-goal to scan sources (1.7+); CI will need when real TSX lands
    location: >-
      packages/i18n/scripts/extract.ts:1
    severity: low
baseline_revision: 'ddf3cd61e91d94a561706afd99f9775323a8ebc2'
baseline_commit: 'ddf3cd61e91d94a561706afd99f9775323a8ebc2'
commit: '66ad22c89ee65ece6d1c98a2f248b7ec780c1deb'
operator_actions: []
---

<intent-contract>

## Intent

**Problem:** The @bronze/i18n package slot exists but is empty; without catalogs, ICU setup, missing-key enforcement, and pseudo-locale test data, any later TSX will be forced to hard-code English literals, violating I18N-001, I18N-002, I18N-003, and G-06 before the gates exist.

**Approach:** Seed @bronze/i18n as a self-contained TypeScript package using react-i18next + i18next-icu, ship en + en-XA + ar-XB catalogs with stable semantic keys and ICU plurals, implement RFC 4647 script-preserving progressive fallback resolver, provide validate script that exits non-zero on key/ICU/placeholder mismatch, add unit tests that assert the exact fallback chain for zh-Hant-HK, and ensure zero user-facing English literals are introduced in any TS/TSX authored under this story.

## Boundaries & Constraints

**Always:**
- Every catalog lives under packages/i18n/locales/{bcp47}/*.json using only stable semantic IDs (never English prose as keys).
- Fallback chain must preserve script/variant subtags before base language then en (zh-Hant-HK → zh-Hant → zh → en); never jump to base language while script tag present.
- ICU MessageFormat required for any plural/select; user-supplied content always isolated, never interpolated into message keys or concatenated.
- validate script must fail (non-zero) on missing keys between en base and any other locale, mismatched placeholder names or ICU syntax, invalid BCP 47, or English values that look concatenated.
- All new source under packages/i18n must be free of sentence literals intended for end users; any literals are technical or test-only.
- Test the fallback resolver and ICU plural path in unit tests.

**Block If:**
- Adding a hard-coded user-facing sentence in any TSX/JSX (this story must not create any TSX).
- Requiring human linguistic review or shipping non-pseudo non-en locales.

**Never:**
- Human translation work or reviewed locale files beyond the three required.
- Native InfoPlist / Swift string catalogs (later story).
- Sentence concatenation in code, catalogs, or tests.
- Storing or emitting user content inside message IDs or as raw keys.
- Adopting Proposed ADR-002/009/018.
- Modifying sprint-status.yaml.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|---------------|----------------------------|----------------|
| FALLBACK_SCRIPT | 'zh-Hant-HK' requested, en base present | chain = ['zh-Hant-HK', 'zh-Hant', 'zh', 'en'] | en used; never 'zh' before 'zh-Hant' |
| FALLBACK_BASE | 'pt-BR' with no pt-BR catalog | chain ends at 'pt' then 'en' | uses 'en'; no crash |
| VALIDATE_MATCH | en + en-XA have identical keys + identical {placeholder} sets + valid ICU | exit 0, no output | — |
| VALIDATE_KEY_MISS | en has "queue.count", en-XA omits it | exit 1; reports missing key | stops before any build |
| VALIDATE_PLACEHOLDER | en uses {count, plural...}, pseudo uses {count} only | exit 1; reports placeholder mismatch | |
| PSEUDO_EN_XA | English source string with {ph} and normal text | produces >=140% length, accents, boundary markers, {ph} preserved verbatim | |
| NO_CONCAT | catalog value contains no un-interpolated adjacent literals that form sentences | accepted | concat-like values rejected by validate |
| USER_CONTENT | t('foo', { name: userText }) | userText isolated in output; never becomes key | |

</intent-contract>

## Code Map

- `packages/i18n/package.json` -- pins i18next@26.4.0, react-i18next@17.0.12, i18next-icu@2.4.4; devDeps typescript@7.0.2, vitest@4.1.11, tsx@4.23.12, @types/react@19.2.18; scripts: "test", "validate", "typecheck"
- `packages/i18n/tsconfig.json` -- strict, ES2022, moduleResolution bundler, jsx react-jsx, outDir dist, noEmit for typecheck
- `packages/i18n/src/locale.ts` -- export canonicalizeBcp47, computeFallbackChain (RFC 4647 progressive), isSupported
- `packages/i18n/src/create-i18n.ts` -- creates i18next instance with static resources, ICU plugin, initReactI18next, default lng 'en', fallbackLng 'en', keySeparator false, ns 'app'
- `packages/i18n/src/formatters.ts` -- re-export of Intl formatters cached by locale + t wrapper
- `packages/i18n/src/generated-message-ids.ts` -- exported const object or type union of all semantic keys for future compile-time safety (seeded from en catalog)
- `packages/i18n/src/index.ts` -- barrel: export * from each; createI18n() factory
- `packages/i18n/locales/en/app.json` -- minimal seeds exercising ICU plural and interpolation, e.g. selection count, app name; no English sentence concat
- `packages/i18n/locales/en-XA/app.json` -- pseudo-expanded copy of en/app (generated)
- `packages/i18n/locales/ar-XB/app.json` -- bidi pseudo copy of en/app (generated)
- `packages/i18n/scripts/validate.ts` -- loads en as base via import, walks other tags, asserts key parity, placeholder/ICU parity per message, valid tags; process.exit(1) on any violation
- `packages/i18n/scripts/pseudo.ts` -- reads en, writes en-XA (accent + pad + [] markers, keep {..} untouched), ar-XB (bidi isolates + rtl markers)
- `packages/i18n/scripts/extract.ts` -- placeholder for future source extraction (empty or minimal for now)
- `packages/i18n/test/locale.test.ts` -- vitest: asserts computeFallbackChain('zh-Hant-HK') === ['zh-Hant-HK','zh-Hant','zh','en'] and other cases; also exercises i18n plural path
- `turbo.json` -- add "test": {}, "typecheck": {} tasks (no cache for i18n validate if side-effecty)
- `packages/i18n/vitest.config.ts` -- vitest config for this package (or rely on future root)

## Tasks & Acceptance

**Execution:**
- `packages/i18n/package.json` -- replace stub with full manifest declaring deps, scripts for test/validate/typecheck, type module
- `packages/i18n/tsconfig.json` -- write strict config
- `packages/i18n/src/` -- mkdir, write locale.ts, create-i18n.ts, formatters.ts, generated-message-ids.ts, index.ts with correct re-exports and logic
- `packages/i18n/locales/en/app.json` -- write initial catalog with ≥1 ICU plural and ≥1 interpolation; no concat
- `packages/i18n/locales/en-XA/` and `ar-XB/` -- mkdir + write generated pseudo catalogs (run pseudo script or hand-author matching rules)
- `packages/i18n/scripts/` -- mkdir, write validate.ts (enforces AC), pseudo.ts (generator), extract.ts (stub)
- `packages/i18n/test/locale.test.ts` -- write the fallback chain test plus ICU usage test
- `packages/i18n/vitest.config.ts` -- write if required for isolated run
- `turbo.json` -- edit to declare test and typecheck tasks
- after manifest edit: `pnpm install` -- updates lockfile, installs new packages
- `pnpm --filter @bronze/i18n typecheck` -- clean
- `pnpm --filter @bronze/i18n test` -- all green
- `pnpm --filter @bronze/i18n validate` -- exit 0
- `python3 tooling/planning-checks.py` -- PASS
- `npx --yes markdownlint-cli2 'docs/**/*.md' README.md AGENTS.md` -- 0 issues
- confirm no new user-facing English sentence literals appear in any *.ts/*.tsx added

**Acceptance Criteria:**
- Given packages/i18n exists
- When catalogs and validators are added
- Then build fails on missing keys or placeholder mismatch
- And script-preserving fallback unit tests include zh-Hant-HK → zh-Hant → zh → en
- And no user-facing English literals in new TSX

## Spec Change Log

## Review Triage Log

### 2026-08-27 — Review pass
- intent_gap: 0
- bad_spec: 3: (high 1, medium 1, low 1)
- patch: 5: (high 0, medium 3, low 2)
- defer: 7
- reject: 10
- addressed_findings:
  - `[medium]` `[patch]` Wired computeFallbackChain into createI18n + extended fallback coverage in test
  - `[medium]` `[patch]` Fixed pseudoArXB char reversal; regenerated ar-XB/en-XA catalogs so transforms apply
  - `[medium]` `[patch]` Exported validate helpers + added unit tests exercising mismatch/ICU detection paths
  - `[low]` `[patch]` Removed bogus `allowBuilds` placeholder injected into pnpm-workspace.yaml
  - `[low]` `[patch]` Re-ran full verification matrix (7 tests, validate OK, typecheck OK) after patches

### 2026-08-27 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 13: (high 0, medium 7, low 6)
- defer: 0
- reject: 20
- addressed_findings:
  - `[medium]` `[patch]` Fail validate when base ICU plural/select is collapsed to a plain `{name}` placeholder
  - `[medium]` `[patch]` Parse top-level placeholder names with brace depth instead of first-`}` regex
  - `[medium]` `[patch]` Reject English catalog values that look concatenated
  - `[medium]` `[patch]` Stop running validate()/process.exit at import; export collectCatalogErrors for tests
  - `[medium]` `[patch]` Add VALIDATE_KEY_MISS fixture test
  - `[medium]` `[patch]` Add VALIDATE_PLACEHOLDER fixture test
  - `[medium]` `[patch]` Add NO_CONCAT fixture test
  - `[low]` `[patch]` Do not skip empty-string translations in per-message checks
  - `[low]` `[patch]` Scan locale directory names for invalid BCP 47
  - `[low]` `[patch]` Unit-test pseudoEnXA length, markers, and verbatim `{ph}`
  - `[low]` `[patch]` Unit-test user interpolation isolation on capture.source
  - `[low]` `[patch]` Unit-test pseudoArXB bidi isolates around simple placeholders
  - `[low]` `[patch]` Guard pseudo.ts CLI entry; do not trim padding below the 140% target

### 2026-08-27 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 7: (high 0, medium 2, low 5)
- defer: 0
- reject: 28
- addressed_findings:
  - `[medium]` `[patch]` loadLocale honors localesDir; runValidate tested on seeded tree and fixture missing-key / ICU / invalid-tag failures
  - `[medium]` `[patch]` FALLBACK_BASE: exact pt-BR → pt → en chain and createI18n('pt-BR') uses en
  - `[low]` `[patch]` Golden-test committed en-XA/ar-XB against generators; regenerated stale en-XA padding
  - `[low]` `[patch]` Assert en-XA accents on letters
  - `[low]` `[patch]` Wrap each repeated simple ar-XB placeholder at restore (no first-match replace)
  - `[low]` `[patch]` Remove identical computeFallbackChain if/else branches
  - `[low]` `[patch]` VALIDATE_MATCH: success CLI emits no stdout

## Design Notes

Fallback implementation deliberately walks the BCP 47 subtags from right to left, stopping before stripping a script or variant tag. This guarantees zh-Hant-HK never collapses directly to zh. ICU is wired at i18next init so that plural messages are parsed by the ICU engine rather than i18next's default pluralizer; this satisfies the "ICU plurals required for counts" rule.

Pseudo generators are pure and deterministic: en-XA inserts combining accents on letters, inflates length by padding, and wraps output in 【】 markers while regex-protecting every {placeholder} and ICU syntax. ar-XB inserts RLI/PDI isolates around interpolated content and reverses top-level word order for bidi stress testing without corrupting ICU or placeholders.

Catalogs start with a tiny set of seeds only to prove the machinery; no product strings are authored here (those arrive with UI stories under the guard of the validate gate).

The validate script is intentionally a standalone Node runner (via tsx) so it can be invoked from CI or `pnpm --filter` without requiring a full turbo pipeline or UI.

## Verification

**Commands:**
- `pnpm --filter @bronze/i18n test` -- expected: exit 0; locale.test.ts passes zh-Hant-HK chain and plural formatting
- `pnpm --filter @bronze/i18n validate` -- expected: exit 0; no key or placeholder diffs reported
- `pnpm --filter @bronze/i18n typecheck` -- expected: exit 0
- `pnpm install` -- expected: success; pnpm-lock.yaml updated only for the three new runtime + four dev packages under @bronze/i18n
- `python3 tooling/planning-checks.py` -- PASS
- `npx --yes markdownlint-cli2 'docs/**/*.md' README.md AGENTS.md` -- 0 issues
- `git ls-files --others --exclude-standard | grep -E 'packages/i18n' || true` -- only intended new files

**Manual checks (if no CLI):**
- Inspect packages/i18n/locales/en/app.json values contain no English sentence fragments joined by + or adjacent literals outside { }.
- Confirm packages/i18n/src/locale.ts exports computeFallbackChain and the zh case is exercised exactly.
- rg -n ' [A-Z][^`]+ [a-z].*[a-z]\.' packages/i18n/src packages/i18n/test || true -- no sentence literals in source.

## Auto Run Result

Status: done

Summary: Seeded `@bronze/i18n` with en / en-XA / ar-XB catalogs, ICU + script-preserving fallback, and a validate CLI. Follow-up review patched `runValidate` to load fixture trees, added CLI-surface and pt-BR runtime tests, regenerated stale en-XA padding, and wrapped repeated ar-XB placeholders at restore.

Files changed:
- `packages/i18n/package.json` — scripts, i18next/icu/react-i18next, vitest/tsx/typescript
- `packages/i18n/tsconfig.json` — strict ESM package config
- `packages/i18n/vitest.config.ts` — node tests under `test/`
- `packages/i18n/src/locale.ts` — RFC 4647 script-preserving fallback
- `packages/i18n/src/create-i18n.ts` — i18next + ICU instance with fallback chain
- `packages/i18n/src/formatters.ts` — cached Number/DateTime/Plural formatters
- `packages/i18n/src/generated-message-ids.ts` — seeded semantic ids
- `packages/i18n/src/index.ts` — public barrel
- `packages/i18n/locales/{en,en-XA,ar-XB}/app.json` — seed catalogs
- `packages/i18n/scripts/validate.ts` — key/ICU/placeholder/concat/BCP 47 gate; loadLocale respects localesDir
- `packages/i18n/scripts/pseudo.ts` — en-XA / ar-XB generators; simple placeholders wrapped at restore
- `packages/i18n/scripts/extract.ts` — stub
- `packages/i18n/test/locale.test.ts` — fallback, ICU plural, user interpolation, pt-BR runtime
- `packages/i18n/test/validate.test.ts` — validate failure paths, runValidate fixtures, pseudo contracts, golden catalogs
- `turbo.json` — test, typecheck, validate tasks
- `pnpm-lock.yaml` — new package deps

Review findings breakdown:
- patches applied: 7 (high 0, medium 2, low 5)
- items deferred: 0 this pass (3 pre-existing frontmatter items preserved)
- items rejected: 28

Follow-up review recommendation: true (patched medium 2, low 5; score `3 × 2 + 5 = 11` ≥ 5; no high)

Verification performed:
- `pnpm --filter @bronze/i18n test` — 25 passed
- `pnpm --filter @bronze/i18n validate` — exit 0, no stdout
- `pnpm --filter @bronze/i18n typecheck` — exit 0
- `python3 tooling/planning-checks.py` — PASS (56 requirement IDs)
- `npx --yes markdownlint-cli2 'docs/**/*.md' README.md AGENTS.md` — 0 issues
- `git ls-files --others --exclude-standard` for packages/i18n — none
- Manual: en/app.json has no concatenated sentences; computeFallbackChain zh-Hant-HK exercised; source comments only, no user-facing sentence literals

Residual risks:
- Formatter surface, typed `t()` adapter, and extract/id regeneration remain deferred to later UI stories
- ar-XB leaves whole-message ICU blocks unreversed so plurals stay valid
- Orchestrator-owned `sprint-status.yaml` and `deferred-work.md` were not modified

