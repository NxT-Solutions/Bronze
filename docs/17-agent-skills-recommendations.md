# Agent-skill recommendations

Research date: 2026-08-27. No skill was installed during this task. Install only after reading its `SKILL.md`, repository license, scripts, permissions, and maintenance state. Install counts and stars are popularity signals, not security review.

Catalog: [skills.sh](https://skills.sh/).

## Recommended core set

| Skill | Snapshot signal | Use in Bronze | Recommendation |
| --- | --- | --- | --- |
| `anthropics/skills@frontend-design` | ~824k installs; repo ~171.9k stars | original polished UI, avoids generic generated look | strong; still obey Bronze accessibility tokens/spec |
| `vercel-labs/agent-skills@vercel-react-best-practices` | ~668k installs; repo ~30.5k stars | React composition/render/performance review | strong for WebView UI |
| `vercel-labs/agent-skills@web-design-guidelines` | ~582k installs | UI review checklist | strong companion, not native a11y proof |
| `mattpocock/skills@tdd` | ~778k installs; repo ~238.2k stars | behavior-first domain/UI work | strong for bounded tasks |
| `obra/superpowers@verification-before-completion` | ~191k installs; repo ~278.2k stars | prevents compile-only “done” claims | strong; aligns release gates |
| `obra/superpowers@subagent-driven-development` | ~188k installs | parallel slices/handoffs | useful after contracts/ownership clear |
| `jakubkrehel/skills@better-accessibility` | ~9.7k installs; repo ~4.5k stars, MIT | Web UI accessibility reviews | good supplement; native macOS still manual/official guidance |

Typical install syntax from catalog:

```sh
npx skills add anthropics/skills@frontend-design
npx skills add vercel-labs/agent-skills@vercel-react-best-practices
npx skills add vercel-labs/agent-skills@web-design-guidelines
npx skills add mattpocock/skills@tdd
npx skills add obra/superpowers@verification-before-completion
npx skills add obra/superpowers@subagent-driven-development
npx skills add jakubkrehel/skills@better-accessibility
```

Do not run all skills on every task. Minimal routing:

- UI creation: frontend-design + React best practices + better-accessibility.
- Domain/native/store: TDD + verification-before-completion.
- Large milestone: subagent-driven-development only after task graph and file ownership are explicit.
- Review: web-design-guidelines + better-accessibility + Bronze conformance spec.

## Tauri and i18n candidates: audit first

| Candidate | Signal | Concern | Use |
| --- | --- | --- | --- |
| `mindrally/skills@tauri-development` | ~1.1k installs; repo ~244 stars, Apache-2.0 | smaller evidence base; may lag current Tauri 2 security APIs | use only after comparing to official Tauri docs |
| `mindrally/skills@internationalization-i18n` | ~853 installs; same repo | generic guidance may miss native Swift/menu and bidi details | supplement this pack, not replace it |
| `...@tauri-v2` from `nodnarbnitram` catalog result | ~7k installs but source repo ~15 stars at research date | popularity/repository mismatch; higher supply-chain/content risk | do not install without full manual audit |

If testing candidate in isolated environment:

```sh
npx skills add mindrally/skills@tauri-development
npx skills add mindrally/skills@internationalization-i18n
```

Review exact fetched revision and generated instructions before allowing write or shell actions.

## Do not delegate these decisions to generic skill

- macOS TCC permission model, event-tap mode, Secure Input, AX selection behavior;
- native bridge memory/thread ownership;
- clipboard mutation and causality safety;
- bundle identity, signing, hardened runtime, notarization;
- WCAG/EN 301 549 conformance claim;
- import/path/IPC capability threat model;
- Cooper/Copper license/trade-dress boundary.

Use official Apple, Tauri, W3C, ETSI, Unicode, and repository sources listed in [sources](19-sources.md).

## Skill intake checklist

Before install/use:

1. Read entire `SKILL.md` and referenced scripts/assets.
2. Record source repository, exact revision, license, maintainer, last update.
3. Inspect shell/network/file-write behavior and prompt-injection exposure.
4. Confirm guidance matches pinned Tauri/React/shadcn versions.
5. Run in isolated task first; review diffs and commands.
6. Never let skill override `AGENTS.md`, security model, requirement IDs, or user scope.
7. Re-audit when revision changes.

## Suggested agent loop

```text
task contract + requirement IDs
  → TDD skill for tests/contracts
  → subsystem implementation
  → design/React/a11y skill only if UI touched
  → Bronze-native manual evidence if macOS boundary touched
  → verification-before-completion
  → traceability/evidence handoff
```

Skills accelerate method. They do not provide platform permission, accessibility, security, or release evidence.
