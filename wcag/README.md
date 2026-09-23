# WCAG 2.2 — Bronze

Vendor-neutral accessibility authority for every coding agent that works
this repo. Cursor, Claude Code, Codex, Grok, and future OSS contributors
read the same files. Do not fork a second copy of these rules inside a
vendor folder.

Official sources (W3C WAI, current):

- [WCAG Overview](https://www.w3.org/WAI/standards-guidelines/wcag/)
- [WCAG 2.2 Recommendation](https://www.w3.org/TR/WCAG22/) (12 December 2024)
- [How to Meet WCAG 2.2](https://www.w3.org/WAI/WCAG22/quickref/)
- [Understanding WCAG 2.2](https://www.w3.org/WAI/WCAG22/Understanding/)
- [Techniques for WCAG 2.2](https://www.w3.org/WAI/WCAG22/Techniques/)
- [WCAG2ICT](https://www.w3.org/TR/wcag2ict-22/) for non-web ICT

WCAG 2.2 is also ISO/IEC 40500:2025 (October 2023 text). Bronze uses the
December 2024 Recommendation. Content that meets 2.2 also meets 2.1 and
2.0. WCAG 3 is a draft and is not a claim target.

## Read order

1. `SKILL.md` — when to apply, workflows, checklists
2. `principles.md` — POUR
3. `criteria.md` — A / AA / AAA mapped onto Bronze
4. `bronze.md` — tokens, claim rules, stack constraints
5. `screens.md` — Queue, Settings, Library, Help

## Agent adapters

| Agent | Loads first | Then reads |
| --- | --- | --- |
| Any / OSS / Codex / Grok | `AGENTS.md` | `wcag/SKILL.md` on UI or a11y work |
| Claude Code | `CLAUDE.md` | same |
| Cursor | `.cursor/rules/wcag.mdc` on chrome globs; `.cursor/skills/wcag/` | same |
| Claude Code skill | `.claude/skills/wcag/` | same |

Adapters are pointers. If an adapter and this folder disagree, this
folder wins.

## Claim rule

Do not publish “conforms to WCAG 2.2 AA” or “AAA” from token tests,
axe, or this library. WebView target is WCAG 2.2 AA (`docs/10`). Whole
app uses EN 301 549 plus VoiceOver. Public claims need
`docs/evidence/`. Inventory rows may say Supports /
Partial / N/A without a public claim.
