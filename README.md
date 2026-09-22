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

**What you will see in `tauri dev`.** The Quick Panel is a zinc inbox: composer, active items, and a **Copy** button on each row. More actions (complete, skip, trash, edit, move) sit in overflow. Library, Settings, and Help are separate Tauri windows — open them from the **Bronze** app menu or the menu-bar status item. Opening `*.html` in a browser has **no** Tauri invoke.

Hand-test in the **native** window only:

- Composer **Add** (or Cmd-Enter) persists to local SQLite
- **Copy** on a row writes the 6-4 pasteboard path (`pbcopy`); default profile is Plain
- Complete / skip hide the row from this inbox; trash removes it
- **Capture** (status menu or Bronze app menu) and **Shift double-tap** (either side, gap 250 ms, hold ≤400 ms) snapshot `NSWorkspace` frontmost PID first, then read AX selection from **that** last non-Bronze app only (focused element, then its windows/children; attributed range when available, else `AXSelectedText` or `AXStringForRange`). The persist read does not follow Bronze or a permission dialog. Capture does **not** steal focus or force-show the panel (5-3). Use **Show** to see the queue. Select text in another app (TextEdit is the known-good check), then Capture or double-tap Shift **without** needing to keep that app frontmost. Capturing Bronze’s own WebView is rejected
- Every persist outcome posts a Notification Center banner (`UNUserNotificationCenter` in a packaged `.app`, or signed `BronzeNotice.app` during unbundled `tauri dev`; catalog `capture.announce.saved|rejected|denied|protected|failed|excluded` only — never the selection). Inbox `#capture-status` stays a visually hidden live region; `#chrome-notice` is the viewport-fixed in-app line when the queue is open or scrolled. A saved row also shows catalog `capture.source` (`From {appName}`) and a 16px official source-app icon when Rust can resolve the local `.app`. Composer **Add** rows stay unlabeled; add failures use `#chrome-notice`. Each row may show a stored `title` heading; the body is sanitized constrained markdown (including line-start and `•` lists) with **Show more** / **Show less** when it overflows. Copy writes plain plus sanitized HTML and reports catalog `copy.announce.copied` / `copy.announce.failed` on the control. Titles persist as `compact_title` and may refine on-device (ADR-019 Proposed; no hosted AI, no PCC, no bundled GGUF)
- Status item: left-click **Show**; menu is the latest five overview items (click copies), then Capture, Help, Quit
- Status or app-menu **Show** recreates the Quick Panel if you closed it
- Settings / Library / Help recreate if you closed them
- Settings load/save `SettingsV1`; permission **Retest** and **Open System Settings** run from the Settings window. Privacy excluded apps is a search plus **Choose app…** (Finder `.app` panel); WebView never receives paths
- Library search is substring-only (`QUE_007_COMPLETE=false`; ADR-018 stays Proposed)
- Backup / export / import write under the Rust-owned app data dir; WebView paths are rejected

Palette is zinc (`#fafafa` / `#18181b` / `#e4e4e7`). Default UI locale is **en**.

**Still stub / backlog.** Event-tap live fire is `capture.selection` Shift double-tap; other seeded defaults persist without a live OS grab, and Quick Panel keydown still uses hardcoded composer chords. File and image attachments are out until a new ADR and threat-model update (ADR-001). Human stories 3.9, 3.10, 5.5, and 9.3 stay backlog. After a permission grant, quit Bronze fully and re-run `pnpm --filter desktop tauri dev`.

**Permissions (macOS).** On native start, Bronze requests **Accessibility** and **Input Monitoring** when they are not already granted (real OS dialogs). The first capture path requests once more if still ungranted. Settings → permission health **Retest** re-attempts those prompts; if macOS will not show another dialog, use **Open System Settings** after that attempt (not instead of it).

- **Privacy & Security → Accessibility** — AX selection (prompted; composer remains if denied)
- **Privacy & Security → Input Monitoring** — listen-only event tap / chord (prompted; menu + composer remain if denied)
- **Screen Recording** — not used; Bronze never prompts for it and you should not grant it

See the prompts with `pnpm --filter desktop tauri dev` (no Corepack). After a grant, quit and relaunch (`requires_relaunch` is a real permission state). Input Monitoring often prompts only once per TCC identity; a later Retest may be silent. A `tauri dev` rebuild can receive a new TCC identity.

**Triggers in this build.** Status extra: left-click Show; menu is latest five overview items, Capture, Help, Quit. Bronze app menu still offers Show, Capture, Settings, Library, Help, Quit. Every seeded `ShortcutActionId` has an enabled default (`docs/12` §4). `SettingsV1.capture.standardChord` is the settings view of `capture.selection` and ships as Shift double-tap (either side, gap 250 ms, max hold 400 ms, `tapCount` 2). Event-tap live fire is that capture chord. The Settings recorder also accepts a single-modifier multi-tap (2 through 8 taps of Shift, Option, Command, Control, or Fn) and persists `tapCount` on trigger `modifier_double_tap`; those remaps do not grab the OS. Commit waits 500 ms after the last tap so a second tap does not lock before a third. Other globals persist without an OS grab. App-local chords persist in Settings; Quick Panel keydown still uses hardcoded composer chords. Library search is substring-only (`QUE_007_COMPLETE=false`).

**Stop.** In the `tauri dev` terminal: `Ctrl+C`. Then quit Bronze from the Dock / Force Quit if the process stays resident.

**Verify (maintainer bar, not required to launch):** `pnpm verify`. Local debug package (SEC-005): `tooling/package-debug.sh`.

**Known limitations (not silent accepts).** ADR-002, ADR-009, and ADR-018 stay **Proposed**. ADR-019 (on-device titles) is **Proposed**. QUE-007 locale search is **blocked** on ADR-018. Human stories 3.9, 3.10, 5.5, and 9.3 stay backlog. `bronze-desktop` must keep `bronze-platform-macos` at `default-features = false` (links `libBronzeNative.a`; do not re-enable `abi-stub`). No WCAG / VoiceOver / notarization claim without `docs/evidence/`.

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
