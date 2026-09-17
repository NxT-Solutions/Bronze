# Apple Human Interface Guidelines — Bronze

Vendor-neutral design authority for every coding agent that works this
repo. Cursor, Claude Code, Codex, Grok, and future OSS contributors
read the same files. Do not fork a second copy of these rules inside a
vendor folder.

Official sources (Apple, current):

- [Design principles](https://developer.apple.com/design/human-interface-guidelines/design-principles) (reintroduced 8 June 2026)
- [Human Interface Guidelines](https://developer.apple.com/design/human-interface-guidelines)
- [Get started](https://developer.apple.com/design/get-started/)

## Read order

1. `SKILL.md` — when to apply, workflows, checklists
2. `principles.md` — Purpose through Delight
3. `macos.md` — macOS typography, layout, controls, writing, motion
4. `bronze.md` — how those rules land in this WebView app
5. `screens.md` — Queue, Settings, Library, Help

## Agent adapters

| Agent | Loads first | Then reads |
| --- | --- | --- |
| Any / OSS / Codex / Grok | `AGENTS.md` | `hig/SKILL.md` on UI work |
| Claude Code | `CLAUDE.md` | same |
| Cursor | `.cursor/rules/apple-hig.mdc` on chrome globs; `.cursor/skills/apple-hig/` | same |
| Claude Code skill | `.claude/skills/apple-hig/` | same |

Adapters are pointers. If an adapter and this folder disagree, this
folder wins.
