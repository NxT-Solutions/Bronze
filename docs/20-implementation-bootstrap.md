# Implementation bootstrap

Version: 1.0 bootstrap record  
Recorded: 2026-08-27  
Branch: `bmad/bronze-autonomous`  
Baseline HEAD: `20691ed1a385392d1b92f05fa6d639a72cbe8636` (`20691ed :chart_with_upwards_trend: Analysis of application`)  
Project root: `/Users/noah/fdev/projects/bronze-app`  
Authority: Phase 0 of autonomous delivery. This document records detected and selected toolchain versions. ADR-002 was later Accepted on 2026-09-25 (split arm64 and Intel packages; the operator asked for the Intel build).

## Current operator pins (2026-09-16)

These replace the August 2026 *selected* column for local run. The detection table below stays the bootstrap record.

| Tool | Live pin | Where |
| --- | --- | --- |
| Node.js | 24.21.0 (latest 24 LTS; Current is 26.8.2, not pinned) | `package.json` `engines.node` |
| pnpm | 12.4.2 | `package.json` `packageManager`; `npm install -g pnpm@12.4.2` (never Corepack) |
| rustc / cargo | 1.98.1 | `rust-toolchain.toml` |
| Xcode | 26.6 | host (unchanged) |

## 1. Preflight

| Check | Result |
| --- | --- |
| Working tree at start | clean |
| `main` | untouched; remains at `20691ed` |
| Created branch | `bmad/bronze-autonomous` from `20691ed` |
| Push | not performed |
| Existing worktree | `/Users/noah/orca/workspaces/bronze-app/makara` on `NoahNxT/makara` at same HEAD |

uv 0.12.6 and tmux 3.7c were already installed through Homebrew. No additional Homebrew install was required.

## 2. Detected host

| Item | Detected |
| --- | --- |
| macOS | 26.5.2 (25F84) |
| Architecture | arm64 (`aarch64-apple-darwin`) |
| Xcode | 26.6 (17F113) |
| Xcode path | `/Applications/Xcode.app/Contents/Developer` |
| Apple clang | 21.0.0 (clang-2100.1.1.101) |
| Swift | Apple Swift 6.3.3 (swiftlang-6.3.3.1.3 clang-2100.1.1.101); swift-driver 1.148.6 |
| Git | 2.50.1 (Apple Git-155) |

Host development remains Apple Silicon. Production range is ADR-002 Accepted: macOS 14.0+ with named arm64 and x86_64 packages.

## 3. Detected and selected toolchain

| Tool | Detected | Selected for this bootstrap |
| --- | --- | --- |
| Node.js | v24.19.0 (`/opt/homebrew/opt/node@24/bin/node`) | v24.19.0 |
| npm | 11.17.0 | 11.17.0 (npx only; JS workspace uses pnpm) |
| pnpm | 11.9.0 | 11.9.0 host; lock to current stable when JS workspace is created |
| Python | 3.14.7 | 3.14.7 (uv tool runner only) |
| uv | 0.12.6 (Homebrew 2026-08-25) | 0.12.6 |
| tmux | 3.7c | 3.7c |
| Homebrew | 6.0.19 | 6.0.19 |
| rustup | 1.29.0 (2026-03-05) | 1.29.0 |
| rustc | 1.98.0 (88d9e12ae 2026-08-18, Homebrew) | 1.98.0 |
| cargo | 1.98.0 (797e8a9bc 2026-08-05, Homebrew) | 1.98.0 |
| LLVM (via rustc) | 22.1.8 | 22.1.8 |
| OpenCode | 1.18.23 (`/opt/homebrew/bin/opencode`) | 1.18.23 |
| markdownlint-cli2 | v0.23.2 (markdownlint v0.41.1) | documentation lint |
| cargo-tauri | not installed | install with workspace bootstrap, not globally assumed |
| Biome CLI | not on PATH | pin via pnpm workspace when JS packages exist |
| swift-format | not on PATH | install during native workspace bootstrap |

Planning registry snapshot in [technology baseline](../research/technology-baseline.md) remains dated input, not a lockfile. Exact JS/Rust crate versions lock when Milestone 1 scaffolding lands.

## 4. Model routing

xAI access verified with `opencode models xai --refresh` on 2026-08-27. Available models included `xai/grok-4.6` and `xai/grok-build-0.1`.

| Role | Selected model | Fallback |
| --- | --- | --- |
| Planning / bootstrap | `xai/grok-4.6` | none; never silently switch provider |
| Development | `xai/grok-build-0.1` | none |
| Review and deferred-work triage | `xai/grok-4.6` | none |

Do not configure OpenCode model variants or reasoning-effort suffixes: bmad-loop 0.11.1 does not correctly support them.

## 5. BMAD pins (installed)

| Component | Pin | Evidence |
| --- | --- | --- |
| BMAD Method | 6.11.0 | `_bmad` installer output: BMad Core and BMad Method v6.11.0 |
| bmad-loop skills | 0.11.1 | installer: BMAD Loop Skills v0.11.1 |
| bmad-loop orchestrator | 0.11.1 from `git+https://github.com/bmad-code-org/bmad-loop.git@v0.11.1` (`bb7cebec`) | `bmad-loop --version` → `bmad-loop 0.11.1` |
| Modules | `bmm`, `bmad-loop` | `_bmad/bmm`, `_bmad/bmad-loop` |
| Tool integrations | OpenCode runtime plus Claude Code compatibility tree | 52 skills in `.agents/skills` and `.claude/skills`; 52 commands in `.opencode/commands` |
| Output folder | `_bmad-output` | created empty |
| User name | Noah | `_bmad/config.user.toml` |
| Communication / document language | English | installer flags |

Installer used was `npx bmad-method@6.11.0 install`, not `bmad-loop-setup`. Orchestrator was installed with `uv tool install` and initialized with `bmad-loop init --project "$PWD" --cli opencode`.

Verified before planning mutations:

- `_bmad/`
- `.agents/skills/` including `bmad-prd`, `bmad-ux`, `bmad-architecture`, `bmad-create-epics-and-stories`, `bmad-sprint-planning`
- `.claude/skills/` including `bmad-build-auto`
- `.bmad-loop/policy.toml` (written by init; gitignored by stock v0.11.1 because mux backend is machine-local)
- `.bmad-loop/bmad_loop_hook.py`

Stock init gitignores `.bmad-loop/policy.toml`. Phase 5 records the committed project policy separately if the schema requires a tracked copy.

## 6. Non-decisions

This bootstrap does **not**:

- accept ADR-002 at bootstrap time (later Accepted 2026-09-25: split arm64 and Intel packages);
- accept ADR-009 (protected App Group container);
- accept ADR-018 (FTS tokenizer);
- claim WCAG, VoiceOver, or platform conformance;
- introduce application source;
- push remotes or create pull requests.
