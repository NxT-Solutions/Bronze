# Bronze adoption

Hedgehog was adopted onto this repo so new change lands through a
CLI-enforced graph. It does not own the existing workspace, and it does
not convert Bronze toward any shipped Hedgehog core.

## Commands and their source

All four `verify` commands come from root `package.json` script `verify`
(Story 1.8 / DW-1), which is:

`biome check . && turbo run typecheck && turbo run test && turbo run validate && cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test --workspace && python3 tooling/planning-checks.py`

| Layer | Command | Source |
| --- | --- | --- |
| rust | `cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test --workspace` | the Cargo portion of `package.json` `verify` |
| ui | `pnpm exec biome check . && pnpm exec turbo run typecheck && pnpm exec turbo run test && pnpm exec turbo run validate` | the JS portion of the same script; `pnpm exec` is how those binaries are reached outside `pnpm run` |
| docs | `python3 tooling/planning-checks.py` | the last clause of `verify`, and `AGENTS.md` / `.bmad-loop` planning pack |
| join | `pnpm verify` | the full aggregate gate |

There is no CI workflow under `.github/workflows`. `pnpm verify` is the
maintainer bar.

`native/macos/BronzeNative/Package.swift` declares `BronzeNativeTests`,
but no `swift test` (or other Swift) command is part of `pnpm verify` or
any Makefile. This adoption does not invent one. Swift/C ABI edits are
written under the rust layer's scope (`native/**`) and gated by the
Cargo workspace tests that already exist.

## Layer order

Change order, not construction of a new stack:

1. **rust** — Cargo workspace crates, the Tauri crate, and the in-process
   Swift package. Capture, storage, settings, and FFI move here first.
2. **ui** — pnpm workspace packages and the desktop WebView tree
   (`apps/desktop/src/**`), after native/Rust contracts exist.
3. **docs** — planning pack and implementation artifacts, after code so
   requirement IDs and links stay honest.
4. **join** — exclusive `scope: ["**"]` running full `pnpm verify`. It
   catches a rust-green change that broke typecheck, a ui-green change
   that broke clippy, or a docs-green change that drifted from code.

`verify_radius` is `["**"]` on rust and ui because those commands read
the whole workspace (Cargo `--workspace`, root `biome check .`), not
only the write globs. docs omits a radius and falls back to its own
scope — `planning-checks.py` is the command path, and the files it
reads sit in that scope.

## What was left out

- **No stack migration.** Tauri 2, React, Rust, in-process Swift, pnpm,
  Turbo, Biome stay. No Nx, Nest, Next, Drizzle, or other shipped-core
  stack.
- **No legacy-code review.** `reviewer` judges only the unit under
  change.
- **No completion backfill.** Stories already done under bmad-loop
  (epic 1, 2.1, 3.1, 3.2, and closed deferred-work) are not graph nodes.
- **No BMAD planning shelf.** Product drivers are already locked in
  `docs/03-prd.md` and `docs/18-adrs.md`.
- **No `swift test` gate.** It is not in the repo's own verify script.
- **Stories 3.9, 3.10, 5.5, and 9.3 are not reserved as human-only.**
  The operator decides ad hoc whether a person must run them. Do not
  claim WCAG, VoiceOver, or notarization without `docs/evidence/`.
- **Do not resume bmad-loop run `6a79`.**

## Repo shape, as of adoption

Snapshot read 2026-09-16 at git ref `ca38558`. This is what the tree
looked like then, not a live model. Refresh by re-running
`hedgehog-adopt`, not by hand-editing this section.

**Observed architecture: `none`.** `pnpm-workspace.yaml` lists
`packages/*` and `apps/*` with no declared dependency direction.
`Cargo.toml` lists workspace members; crate `Cargo.toml` files are still
mostly empty (only `bronze-platform-macos` depends on `bronze-settings`).
That is not an unambiguous hexagonal or layered package graph.

**Layout**

- Root pnpm workspace (`packageManager: pnpm@11.9.0`) plus Cargo
  workspace plus Swift package at `native/macos/BronzeNative`.
- JS packages: `packages/contracts`, `packages/i18n`,
  `packages/test-support`, `packages/ui`; app `apps/desktop` (Vite/React
  under `src/`, Tauri under `src-tauri/`).
- Rust crates at repo root: `bronze-domain`, `bronze-capture`,
  `bronze-storage`, `bronze-settings`, `bronze-diagnostics`,
  `bronze-platform`, `bronze-platform-macos`.
- Planning authority: `docs/03-prd.md`, `docs/18-adrs.md`,
  `_bmad-output/planning-artifacts/epics.md`.
- Operator-only MyBMAD sidecar lives under `tooling/bmad-dashboard/` and
  is outside this chain.

**Entry points**

- Desktop: `apps/desktop/src-tauri` + `apps/desktop/src`.
- Native ABI: `native/macos/BronzeNative`.
- Verify: `pnpm verify` from repo root.

**Conventions observed**

- Requirement IDs stay in stories, specs, and tests.
- Event-tap callbacks must not touch AX, DB, windows, or clipboard.
- User-facing strings belong in locale catalogs.
- ADR-002, ADR-009, and ADR-018 stay Proposed unless the operator
  accepts them.
- Recent commit subjects are bmad-loop merge lines, not Conventional
  Commits. New Hedgehog commits use Conventional Commits as the fallback
  the adopt skill names when the repo has no house style.

Coverage is partial by design. `hedgehog status` describes work under
this graph since adoption, not the fraction of the whole repo.
