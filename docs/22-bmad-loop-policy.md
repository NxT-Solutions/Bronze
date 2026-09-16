# bmad-loop policy

Configured 2026-08-27 against bmad-loop 0.11.1. Live file: `.bmad-loop/policy.toml` (gitignored by stock init because mux is machine-local). Tracked copy: `.bmad-loop/policy.example.toml`.

## Semantics

| Key | Value |
| --- | --- |
| adapter | `grok` (project profile; Grok Build CLI in tmux) |
| adapter.model / adapter.dev | `grok-4.6` (`--reasoning-effort xhigh`) |
| adapter.review / adapter.triage | `grok-4.6` |
| review.enabled | true |
| review.trigger | always |
| limits.max_review_cycles | 3 |
| limits.max_dev_attempts | 2 |
| session/token budgets | defaults (`warn`, 2M/story, 4M/session) |
| notify.desktop / notify.file | true |
| scm.isolation | worktree |
| scm.target_branch | `bmad/bronze-autonomous` |
| scm.branch_per | story |
| scm.merge_strategy | squash |
| scm.seed_adapter_defaults | true |
| gates.mode | none (unattended); `on_status_contradiction` remains escalate |
| sweep.auto | per-epic |
| mux.backend | tmux |
| verify.commands | `python3 tooling/planning-checks.py` until `pnpm verify` exists |

No OpenCode reasoning-effort suffixes. No publish/deploy commands.

## Validation 2026-08-27

`bmad-loop validate --project "$PWD"` reports skills OK: `.claude/skills` contains `bmad-build-auto` plus review layers. Adapter `grok` (project profile) is selected. Headless `grok -p` can read those skills from the prompt path, project `AGENTS.md` context, story files under `_bmad-output/implementation-artifacts/`, and `sprint-status.yaml`. Folder trust lives in `~/.grok/trusted_folders.toml`; `--trust` covers fresh worktrees.

## Warnings

1. Stock gitignore excludes `.bmad-loop/policy.toml`. Copy `policy.example.toml` after clone.
2. `sprint_plan.py` title/`Epic List` heading warnings are cosmetic.
3. Four human stories remain backlog (3.9, 3.10, 5.5, 9.3). A full unattended run must not treat them as done.
4. Verify command is planning-checks only until the workspace verify script exists (Story 1.8).
5. `git.worktree-clean` fails if policy example or evidence files are uncommitted.
6. Cursor Agent is logged in interactively but not in tmux (needs `CURSOR_API_KEY`). Do not point unattended runs at `cursor-agent` unless that env is set.
7. Grok headless requires folder trust. `--always-approve` does not skip the trust dialog.
