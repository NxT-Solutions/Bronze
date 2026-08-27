# Bronze planning pack

Bronze is a proposed local-first macOS selection-to-action queue: capture selected text, add prompts or notes, order work, copy it back into any app, then complete or archive it. This folder contains research, product requirements, architecture, quality gates, and an agent-ready implementation plan. No application code exists yet.

Research cutoff: **2026-08-27**. Working name: **Bronze**; renameable before implementation.

## Start here

1. [Executive summary](docs/00-executive-summary.md)
2. [Reference product and video analysis](docs/01-product-reference-analysis.md)
3. [Competitive landscape](docs/02-competitive-landscape.md)
4. [Product requirements document](docs/03-prd.md)
5. [Functional specification](docs/04-functional-spec.md)
6. [UX and interaction specification](docs/05-ux-ui-interaction-spec.md)
7. [System architecture](docs/06-system-architecture.md)
8. [macOS capture reliability](docs/07-macos-capture-reliability.md)
9. [Data model and portability](docs/08-data-model-and-portability.md)
10. [Security, privacy, and threat model](docs/09-security-privacy-threat-model.md)
11. [Accessibility conformance plan](docs/10-accessibility-conformance-plan.md)
12. [Internationalization](docs/11-i18n-localization.md)
13. [Settings and shortcuts](docs/12-settings-and-shortcuts.md)
14. [Testing, quality, and release](docs/13-testing-quality-release.md)
15. [Agentic implementation plan](docs/14-agentic-implementation-plan.md)
16. [Backlog and traceability](docs/15-backlog-traceability.md)
17. [Cooper OSS audit](docs/16-cooper-oss-audit.md)
18. [Agent-skill recommendations](docs/17-agent-skills-recommendations.md)
19. [Architecture decisions](docs/18-adrs.md)
20. [Sources and evidence](docs/19-sources.md)
21. [Implementation bootstrap](docs/20-implementation-bootstrap.md)
22. [Pre-implementation reconciliation](docs/21-preimplementation-reconciliation.md)

Implementation agents: [Bronze agent instructions](AGENTS.md).

Research appendices:

- [Methodology and evidence grading](research/methodology.md)
- [Competitor source register](research/competitor-source-register.md)
- [Copper video shotlist](research/reference-video-shotlist.md)
- [Standards notes](research/standards-notes.md)
- [Dated technology baseline](research/technology-baseline.md)
- [Product glossary](research/glossary.md)
- [Asset provenance](assets/README.md)

Original visual direction: [Bronze UI concept](assets/bronze-ui-concept.png). It is inspiration, not a pixel specification.

## Product sentence

> Capture what matters from the app already in front of you, turn it into an ordered local work queue, and send it back without losing your train of thought.

## Non-negotiables

- macOS first; Windows and Linux explicitly deferred.
- Tauri 2, React, TypeScript, shadcn/ui, Biome, pnpm, and Turborepo.
- Native macOS capture path; no cross-platform abstraction around unreliable input hooks.
- Accessibility evidence: WCAG 2.2 AA baseline for WebView UI, relevant EN 301 549 clauses 5, 11, and 12, Apple VoiceOver evaluation, and manual assistive-technology tests. Never market as “WCAG proof.”
- Local-only by default. No account, telemetry, hosted AI, or passive clipboard history.
- Configurable shortcuts with conflicts, timing, permission health, and accessible alternatives.
- Independent implementation, documented provenance, and original Bronze identity. Cooper is a failure corpus, not production base; v1 copies no Cooper source code.

## Scope status

Planning pack is the source of truth. Implementation proceeds on local branch `bmad/bronze-autonomous`. `main` stays at the planning baseline until explicitly merged. The original research task did not create Git state; this repository now exists.
