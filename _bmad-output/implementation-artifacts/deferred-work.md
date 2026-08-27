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
status: open

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
status: open

### DW-5: no Bronze adapter wrapper or typed t() facade exported; createI18n returns raw i18next instance
origin: spec-deferred 6b8c7f8970b8
location: packages/i18n/src/create-i18n.ts:37
source_spec: `1-6-i18n-catalogs-and-pseudo-locales.md`
severity: medium
reason: ADR-014 and docs/11 call for "Bronze localization adapter"; later UI stories will need stable surface
status: open

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
status: open

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
