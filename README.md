<p align="center">
  <img src="docs/images/bronze-mark.png" width="96" height="96" alt="Bronze mark">
</p>

<h1 align="center">Bronze</h1>

<p align="center">
  <strong>Capture what matters, queue it locally, send it back.</strong>
</p>

<p align="center">
  <a href="https://github.com/NxT-Solutions/Bronze/actions/workflows/ci.yml"><img src="https://github.com/NxT-Solutions/Bronze/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-18181b" alt="MIT"></a>
  <a href="https://github.com/NxT-Solutions/Bronze/releases"><img src="https://img.shields.io/github/v/release/NxT-Solutions/Bronze?include_prereleases&amp;label=release" alt="Release"></a>
</p>

<p align="center">
  Local-first macOS selection-to-action queue. No account. No telemetry. No hosted AI by default.
</p>

<p align="center">
  <img src="docs/images/bronze-hero.png" width="920" alt="Bronze queue">
</p>

Bronze sits beside the app you are already in. Capture the selection, order the work, copy it back. It is not a clipboard recorder, a task manager, a note vault, or an AI client.

<p align="center">
  <img src="docs/images/bronze-queue.png" width="440" alt="Bronze queue">
  &nbsp;
  <img src="docs/images/bronze-settings.png" width="440" alt="Bronze settings">
</p>

## Install

macOS 14 (Sonoma) or later, Apple Silicon and Intel. Homebrew 7 refuses the cask until the tap is trusted.

```bash
brew tap NxT-Solutions/nxt-solutions-packages
brew trust nxt-solutions/nxt-solutions-packages
brew install --cask bronze
brew upgrade --cask bronze
```

Or download `bronze-macos-arm64.pkg` or `bronze-macos-x86_64.pkg` from [Releases](https://github.com/NxT-Solutions/Bronze/releases). Settings shows the running version. **Check for updates** is a button (ADR-023 Proposed). Homebrew installs copy `brew upgrade --cask bronze`. A package install opens the GitHub release.

The cask appears after the first published `.pkg`. Until then, use the GitHub asset. Unsigned packages are not a notarization claim.

## Highlights

- **Capture** selected text from the frontmost app (Accessibility first, bounded clipboard fallback)
- **Queue** items locally in SQLite, with copy, complete, skip, and trash
- **Copy** writes a pasteboard payload you can drop into any app
- **On this Mac** title engines, plus optional loopback Ollama or opt-in hosted keys
- **Offline by default** — no account, analytics, or remote fonts

## Develop

Pins: Node 24.21.0, pnpm 12.4.2, rustc 1.98.1, Xcode for `libBronzeNative.a`. Do not use Corepack.

```bash
pnpm install
pnpm --filter desktop tauri dev
```

Hand-test only in the native window. Opening the HTML files in a browser has no Tauri invoke.

```bash
pnpm verify
tooling/package-debug.sh
```

`pnpm verify` is the maintainer bar. Pull requests run the [CI](.github/workflows/ci.yml) quality jobs. CI does not notarize and does not run the Swift-linked desktop crate on Ubuntu.

Permissions: Accessibility and Input Monitoring. Bronze never asks for Screen Recording.

## Quality

| Check | Where |
| --- | --- |
| Biome, typecheck, JS tests, i18n validate | `quality-js` |
| Planning-pack gates | `quality-docs` |
| `cargo fmt`, Clippy, portable crate tests | `quality-rust` |
| macOS Clippy and tests except `bronze-desktop` | `quality-macos` |
| Full workspace including Swift | `pnpm verify` on a Mac |

Do not claim WCAG, VoiceOver, or notarization without files in `docs/evidence/`.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Work lands through a pull request to `main`. Changelog: [CHANGELOG.md](CHANGELOG.md). Security: [SECURITY.md](SECURITY.md).

ADR-002 is Accepted (split arm64 and Intel packages; the operator asked for the Intel build). ADR-009, ADR-018, and ADR-023 stay Proposed unless a human accepts them.

## License

MIT. See [LICENSE](LICENSE). Copyright 2026 Noah Gillard.

## Planning pack

Requirement IDs in [docs/03-prd.md](docs/03-prd.md) and [docs/18-adrs.md](docs/18-adrs.md) remain authority.

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
23. [bmad-loop policy](docs/22-bmad-loop-policy.md)

Implementation agents: [AGENTS.md](AGENTS.md). Local run notes and the long hand-test list live in [docs/20-implementation-bootstrap.md](docs/20-implementation-bootstrap.md) and the git history of this file. Visual direction: [assets/bronze-ui-concept.png](assets/bronze-ui-concept.png) (inspiration only). Asset provenance: [assets/README.md](assets/README.md).
