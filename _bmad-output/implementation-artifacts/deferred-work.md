### DW-1: planning-checks.py and current verify aggregate do not execute the cargo fmt/clippy/test workspace gates (only doc parity)
origin: spec-deferred 6b8f7fbf7741
location: tooling/planning-checks.py
source_spec: `1-3-scaffold-cargo-workspace.md`
severity: low
reason: verification-gap reviewer and emit_epics.py show cargo commands are story-specific; aggregate in 1.8; current run exercised manually
status: done 2026-08-27
resolution: resolved by sweep bundle dw-cargo-fmt-clippy-verify-gates
resolution-undo: 65221d4f1b11eff0ac4987a215f01d214b9fc51b922cea9a5ea231208d398ee6 2026-08-27 7374617475733a206f70656e

### DW-2: cross-references to new crates missing from planning docs and apps/desktop package metadata
origin: spec-deferred cf822f742eeb
location: docs/06-system-architecture.md, docs/14-agentic-implementation-plan.md
source_spec: `1-3-scaffold-cargo-workspace.md`
severity: low
reason: architecture spine and later stories reference them; 1.3 scope was only crates + verify
status: done 2026-08-27
resolution: resolved by sweep bundle dw-crate-path-doc-cross-refs
resolution-undo: 9cbed12e8a32fa3c3ed3af4f75f83dfe1921ec5d390e1a7b8d4a821f26ace450 2026-08-27 7374617475733a206f70656e

### DW-3: Follow-up review still recommended for 1-5-empty-tauri-app-with-csp-deny after the damping cap was spent
origin: review-budget-followup
location: n/a
source_spec: `1-5-empty-tauri-app-with-csp-deny.md`
severity: low
reason: The follow-up-review damping cap (limits.max_followup_reviews = 1) was spent with the story finalized (status: done, verify green) while the review pass still recommended an independent follow-up. The work was committed by bmad-loop run 20260827-110451-2bb6; this entry preserves the lingering recommendation for a deliberate later review.
status: open

### DW-4: formatters.ts only covers NumberFormat/DateTimeFormat/PluralRules; docs/11 §5 and arch require RelativeTimeFormat, ListFormat, Collator, DisplayNames, Segmenter too
origin: spec-deferred b9647ccf14a2
location: packages/i18n/src/formatters.ts:1
source_spec: `1-6-i18n-catalogs-and-pseudo-locales.md`
severity: low
reason: no other callers yet; story focused on catalog+validate+fallback gate; full formatters belong with UI consumption stories
status: done 2026-08-27
resolution: resolved by sweep bundle dw-i18n-adapter-and-formatters
resolution-undo: 4e1b85342874265351d02e2c5380b03d7471802bdbb7f580577ef76fcd5fc723 2026-08-27 7374617475733a206f70656e

### DW-5: no Bronze adapter wrapper or typed t() facade exported; createI18n returns raw i18next instance
origin: spec-deferred 6b8c7f8970b8
location: packages/i18n/src/create-i18n.ts:37
source_spec: `1-6-i18n-catalogs-and-pseudo-locales.md`
severity: medium
reason: ADR-014 and docs/11 call for "Bronze localization adapter"; later UI stories will need stable surface
status: done 2026-08-27
resolution: resolved by sweep bundle dw-i18n-adapter-and-formatters
resolution-undo: 4e1b85342874265351d02e2c5380b03d7471802bdbb7f580577ef76fcd5fc723 2026-08-27 7374617475733a206f70656e

### DW-6: extract script and generated-message-ids regeneration not wired; manual seed only
origin: spec-deferred 1b06d7df56ab
location: packages/i18n/scripts/extract.ts:1
source_spec: `1-6-i18n-catalogs-and-pseudo-locales.md`
severity: low
reason: story non-goal to scan sources (1.7+); CI will need when real TSX lands
status: open

### DW-7: Follow-up review still recommended for 1-6-i18n-catalogs-and-pseudo-locales after the damping cap was spent
origin: review-budget-followup
location: n/a
source_spec: `1-6-i18n-catalogs-and-pseudo-locales.md`
severity: low
reason: The follow-up-review damping cap (limits.max_followup_reviews = 1) was spent with the story finalized (status: done, verify green) while the review pass still recommended an independent follow-up. The work was committed by bmad-loop run 20260827-110451-2bb6; this entry preserves the lingering recommendation for a deliberate later review.
status: open

### DW-8: Follow-up review still recommended for 1-7-shadcn-react-aria-foundation after the damping cap was spent
origin: review-budget-followup
location: n/a
source_spec: `1-7-shadcn-react-aria-foundation.md`
severity: low
reason: The follow-up-review damping cap (limits.max_followup_reviews = 1) was spent with the story finalized (status: done, verify green) while the review pass still recommended an independent follow-up. The work was committed by bmad-loop run 20260827-110451-2bb6; this entry preserves the lingering recommendation for a deliberate later review.
status: open

### DW-9: packages/ui globals.test.ts only regex-matches media-query strings, not !important or computed override of Tailwind utilities (A11Y-003).
origin: spec-deferred 85c20f214fb5
location: packages/ui/src/styles/globals.test.ts:12
source_spec: `1-8-workspace-verify-aggregate.md`
severity: medium
reason: Pre-existing Story 1.7 test; still passed after hygiene stripped !important. This review restored !important but did not extend the test.
status: done 2026-08-27
resolution: resolved by sweep bundle dw-globals-a11y-important-tests
resolution-undo: 02d83bf315ea5c8706dc9e070754e26a382a998ff08d84abc9aca036a67a5059 2026-08-27 7374617475733a206f70656e

### DW-10: The root verify script addition has no automated regression guard; presence of the fmt/clippy sub-commands (and their fail-closed ordering) is asserted only by manual grep inside this spec and ad-hoc
origin: spec-deferred ab433c465df9
location: package.json:7
source_spec: `spec-dw-1-cargo-fmt-clippy-verify-gates.md`
severity: medium
reason: Removing the two subcommands from package.json:7 lets `pnpm verify` still exit 0 (biome+turbo+cargo-test+planning pass); only manual inspection or deliberate violation demos (which are not part of recurring gates) would catch omission. Matches the verification style used for cargo test in 1.8.
status: open

### DW-11: Story 1.8 ACs and related planning docs continue to list the verify aggregate as containing only cargo test for the Rust side (fmt/clippy still appear deferred or omitted).
origin: spec-deferred aafb20b1cc07
location: _bmad-output/implementation-artifacts/1-8-workspace-verify-aggregate.md , epics.md
source_spec: `spec-dw-1-cargo-fmt-clippy-verify-gates.md`
severity: low
reason: 1-8 spec AC, design note, and epics.md describe "cargo test --workspace" without the new gates; no later story owns the aggregate per DW-1.
status: open

### DW-12: Stale `crates/` references and missing bronze-settings cross-reference remain in other planning artifacts (ARCHITECTURE-SPINE.md, 1-3 spec, epic context) after the targeted update.
origin: spec-deferred 046fa2989049
location: _bmad-output/planning-artifacts/architecture/architecture-bronze-app-2026-08-27/ARCHITECTURE-SPINE.md:204
source_spec: `spec-dw-2-crate-path-doc-cross-refs.md`
severity: low
reason: DW-2 bundle intent and spec explicitly limited the scope to exactly docs/06-system-architecture.md and docs/14-agentic-implementation-plan.md; other files that duplicated the old diagrams were intentionally untouched.
status: open

### DW-13: Updated tree diagrams omit root-level peers (Cargo.toml, package.json, tooling/, docs/) and do not document cargo vs pnpm/turbo manager differences for the sibling entries.
origin: spec-deferred 983821958004
location: docs/06-system-architecture.md:181 ; docs/14-agentic-implementation-plan.md:36
source_spec: `spec-dw-2-crate-path-doc-cross-refs.md`
severity: low
reason: Pre-existing abbreviated diagram style; narrow intent asked only to remove the fictional crates/ prefix and add the one table row.
status: open

### DW-14: Ownership table now lists bronze-settings but still omits apps/desktop/src-tauri (a workspace member) and the new row is the only one using (SET-*) requirement tag.
origin: spec-deferred cea7016c5836
location: docs/06-system-architecture.md:207
source_spec: `spec-dw-2-crate-path-doc-cross-refs.md`
severity: low
reason: Table historically listed selective packages; the inserted row text was taken verbatim from the spec's prescribed value for this DW.
status: open

### DW-15: 06 and 14 diagrams remain asymmetric (no JS/Rust grouping, different fence styles ~~~ vs ```, asymmetric coverage of tooling/docs/test-support, src vs src-tauri naming).
origin: spec-deferred 64ef912fbeda
location: n/a
source_spec: `spec-dw-2-crate-path-doc-cross-refs.md`
severity: low
reason: Pre-existing inconsistencies between the two documents' views of the layout; this change only performed the crate flattening and single table addition.
status: open

### DW-16: bronze-settings "Must not own" column lists "capture decisions" which overlaps the responsibility stated for bronze-capture.
origin: spec-deferred eaec7c729565
location: docs/06-system-architecture.md:207
source_spec: `spec-dw-2-crate-path-doc-cross-refs.md`
severity: low
reason: Exact row text (including the must-not phrase) was dictated by the spec's "using:" instruction derived from the bundle intent.
status: open

### DW-17: docs/14 bronze-domain comment still says "capture state/result", which contradicts the 06 domain ownership row (entities/lifecycle/undo) and belongs with bronze-capture.
origin: spec-deferred bd3d34b39201
location: docs/14-agentic-implementation-plan.md:36
source_spec: `spec-dw-2-crate-path-doc-cross-refs.md`
severity: low
reason: Pre-existing 14 comment text was preserved on flatten; 06 domain row is "Entities, lifecycle, order, undo invariants" while 14 labels bronze-domain with capture state/result at root indent.
status: open

### DW-18: REACT_SIDE_EFFECT is specified as useTranslation() in a component; tests only assert getI18n() after createI18n because @bronze/i18n has no React runtime or renderer and Never forbids new dependency p
origin: spec-deferred 2b881cd5cf0c
location: packages/i18n/test/locale.test.ts:132
source_spec: `spec-dw-i18n-adapter-and-formatters.md`
severity: medium
reason: packages/i18n/package.json has react-i18next but no react/react-dom; vitest environment is node; intent matrix names useTranslation in a component; getI18n().t is a singleton proxy.
status: open

### DW-19: Each createI18n() call does createInstance().use(initReactI18next), which rebinds react-i18next's module singleton to the latest instance.
origin: spec-deferred 0e40572d2ddc
location: packages/i18n/src/create-i18n.ts:35
source_spec: `spec-dw-i18n-adapter-and-formatters.md`
severity: medium
reason: Tests already invoke createI18n multiple times (en, en-XA, zh-Hant-HK, pt-BR). getI18n() reflects the last init. Pre-existed before this story's adapter wrap.
status: open
