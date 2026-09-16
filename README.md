# Bronze planning pack

Bronze is a local-first macOS selection-to-action queue: capture selected text, add prompts or notes, order work, copy it back into any app, then complete or archive it. Planning authority is `docs/`. Application code lives on `bmad/bronze-autonomous` (Tauri 2, React/HTML WebView, Rust, in-process Swift).

Research cutoff: **2026-08-27**. Working name: **Bronze**.

## Run locally (this Mac)

cwd: repository root (`/Users/noah/fdev/projects/bronze-app`).

Live pins (2026-09-16): Node 24.21.0 (latest 24 LTS; Tauri 2 asks for LTS), pnpm 12.4.2, rustc 1.98.1 (`rust-toolchain.toml`), Xcode 26.6 + Swift for `libBronzeNative.a`. The [implementation bootstrap](docs/20-implementation-bootstrap.md) table is the 2026-08-27 detection record, not the live pin. Install only what is missing. Do not use Corepack.

```bash
# Node 24.21.0 (latest 24 LTS). Node Current is 26.8.2 and is not the pin.
# https://nodejs.org or: brew install node@24 && brew upgrade node@24
npm install -g pnpm@12.4.2

# Rust 1.98.1 (rust-toolchain.toml). Homebrew rustc on PATH may still print 1.98.0.
rustup toolchain install 1.98.1
rustup run 1.98.1 rustc --version

# Xcode 26.6 / CLT so `swift build` works
xcode-select -p
swift --version
```

Install JS deps once, then start the Tauri 2 debug app:

```bash
pnpm install
pnpm --filter desktop tauri dev
```

Equivalent from `apps/desktop`: `pnpm tauri dev`. That script is the only desktop start command (`apps/desktop/package.json` → `"tauri": "tauri"`). There is no root `dev` script. `beforeDevCommand` is empty; Tauri serves `apps/desktop/src` as `frontendDist`.

**What you will see in `tauri dev`.** The Quick Panel (`quick`) is visible on launch. Library, Settings, and Help are real Tauri windows — open them from the app menu or the Quick Panel buttons (`show_chrome_window`). Opening the `*.html` files in a browser has **no** Tauri invoke: composer, copy, permissions, and backup stay dead there.

Hand-test the queue in the app, not the browser:

- Composer **Add** (or Cmd-Enter) persists to the local SQLite store and refreshes the list
- Complete / trash / skip / move / edit call existing domain actions (QUE-002)
- **Copy** writes the 6-4 pasteboard path (`pbcopy`); profiles are Plain and Markdown
- Settings load/save `SettingsV1`; permission **Retest** and **Open System Settings** run from the Settings window
- Library search is substring-only (`QUE_007_COMPLETE=false`; ADR-018 stays Proposed)
- Backup / export / import write under the Rust-owned app data dir; WebView paths are rejected

Surfaces share a zinc/neutral palette (`#fafafa` / `#18181b` / `#e4e4e7`) — not parchment `#f7f4ef` or brand brown `#8c6239`. Default UI locale is **en** with real spaces.

**Still stub / backlog in this build.** No live `NSStatusItem`. Seeded shortcuts stay **disabled** (menu + composer remain). AX selection capture is **not live** (`AX_CAPTURE_LIVE=false`); Capture in the menu only prompts used permissions. Human stories 3.9, 3.10, 5.5, and 9.3 stay backlog. After a grant or a token change, quit Bronze fully and re-run `pnpm --filter desktop tauri dev`.

**Permissions (macOS).** On native start, Bronze requests **Accessibility** and **Input Monitoring** when they are not already granted (real OS dialogs). The first capture path requests once more if still ungranted. Settings → permission health **Retest** re-attempts those prompts; if macOS will not show another dialog, use **Open System Settings** after that attempt (not instead of it).

- **Privacy & Security → Accessibility** — AX selection (prompted; composer remains if denied)
- **Privacy & Security → Input Monitoring** — listen-only event tap / chord (prompted; menu + composer remain if denied)
- **Screen Recording** — not used; Bronze never prompts for it and you should not grant it

See the prompts with `pnpm --filter desktop tauri dev` (no Corepack). After a grant, quit and relaunch (`requires_relaunch` is a real permission state). Input Monitoring often prompts only once per TCC identity; a later Retest may be silent. A `tauri dev` rebuild can receive a new TCC identity.

**Triggers in this build.** App-menu contract includes Capture, Library, Settings, Help, and Quit. There is still no live menu-bar `NSStatusItem`. `SettingsV1.capture.standardChord` is only the settings view of `capture.selection`; the seeded registry leaves every `ShortcutActionId` disabled. Manual composer is the working add path. Library search is substring-only (`QUE_007_COMPLETE=false`).

**Stop.** In the `tauri dev` terminal: `Ctrl+C`. Then quit Bronze from the Dock / Force Quit if the process stays resident.

**Verify (maintainer bar, not required to launch):** `pnpm verify`. Local debug package (SEC-005): `tooling/package-debug.sh`.

**Known limitations (not silent accepts).** ADR-002, ADR-009, and ADR-018 stay **Proposed**. QUE-007 locale search is **blocked** on ADR-018. Human stories 3.9, 3.10, 5.5, and 9.3 stay backlog. `bronze-desktop` must keep `bronze-platform-macos` at `default-features = false` (links `libBronzeNative.a`; do not re-enable `abi-stub`). No WCAG / VoiceOver / notarization claim without `docs/evidence/`.

## Start here (planning pack)

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

Planning pack IDs in [docs/03-prd.md](docs/03-prd.md) and [docs/18-adrs.md](docs/18-adrs.md) remain authority. Implementation is on local branch `bmad/bronze-autonomous`. `main` stays at the planning baseline until explicitly merged.

Local debug package (SEC-005): `tooling/package-debug.sh` — arm64 unless DG-01 says otherwise, checksums + SBOM stub, no `get-task-allow`, no notarization.
