---
stepsCompleted: [1, 2, 3, 4]
inputDocuments:
  - docs/03-prd.md
  - docs/05-ux-ui-interaction-spec.md
  - docs/06-system-architecture.md
  - docs/18-adrs.md
  - docs/14-agentic-implementation-plan.md
  - docs/21-preimplementation-reconciliation.md
  - _bmad-output/planning-artifacts/prds/prd-bronze-app-2026-08-27/prd.md
  - _bmad-output/planning-artifacts/ux-designs/ux-bronze-app-2026-08-27/DESIGN.md
  - _bmad-output/planning-artifacts/ux-designs/ux-bronze-app-2026-08-27/EXPERIENCE.md
  - _bmad-output/planning-artifacts/architecture/architecture-bronze-app-2026-08-27/ARCHITECTURE-SPINE.md
---

# bronze-app - Epic Breakdown

## Overview

Bronze v1 macOS selection-to-action queue. Stories are sized for one fresh agent context. Risk (toolchain, native bridge, capture, store) precedes polish. Human-only evidence stories stay `blocked-human-validation`.

## Requirements Inventory

### Functional Requirements

- G-01–G-07
- CAP-001–CAP-010
- QUE-001–QUE-008
- WIN-001–WIN-005
- DAT-001–DAT-004
- SEC-001–SEC-006
- A11Y-001–A11Y-006
- I18N-001–I18N-004
- SET-001–SET-004
- SUP-001–SUP-002

### NonFunctional Requirements

- Local-only default; no telemetry/account/network (G-05, SEC-004, ADR-017)
- WCAG 2.2 AA WebView evidence without public conformance claim (G-04, ADR-013)
- BCP 47 / ICU / RTL / IME (G-06, ADR-014)
- Capture reliability G-01/G-02

### Additional Requirements

- Phase 4 UI/i18n/capture guardrails in the autonomous delivery prompt
- docs/21 closed enums

### UX Design Requirements

- Immediate persist; physical panel edge; semantic HTML; 200%/400%/320; native display prefs

### FR Coverage Map

| IDs | Epic |
| --- | --- |
| SEC-001–004, I18N-001–002, A11Y-001–003 slots | Epic 1 |
| CAP-004, ADR-004 | Epic 2 |
| CAP-001–010, SET-003 | Epic 3 |
| DAT-001–004, QUE-001–007 domain | Epic 4 |
| WIN-001–005, A11Y-003 display | Epic 5 |
| QUE-001–008 UI, QUE-004–005 | Epic 6 |
| SET-001–004, DAT-002–003 UI, I18N-001–004 | Epic 7 |
| A11Y-003 visual | Epic 8 |
| SEC-005–006, SUP-001–002, G-04 human | Epic 9 |

## Epic List

1. Epic 1: Bootable toolchain and empty app
2. Epic 2: In-process Swift bridge
3. Epic 3: Capture coordinator and AX-first selection
4. Epic 4: Transactional store and portability
5. Epic 5: Native shell and accessible feedback
6. Epic 6: Queue and library UI
7. Epic 7: Settings shortcuts and portability UI
8. Epic 8: Visual refinement
9. Epic 9: Packaging and release evidence

## Epic 1: Bootable toolchain and empty app

A developer can clone the branch and run format/lint/type/unit gates on an empty macOS Tauri workspace with no product behavior and no network.

### Story 1.1: Pin toolchain manifests

As a developer,
I want pinned Node, pnpm, Rust, and Swift identity files,
So that later stories share one bootstrap.

**Requirements:** G-01 (process), SEC-005 (reproducible later)
**ADRs:** ADR-003; ADR-002 remains Proposed
**Dependencies:** none (canary)
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** a clean `bmad/bronze-autonomous` tree
**When** toolchain pin files are added
**Then** `package.json` packageManager, Node engines, `rust-toolchain.toml` matching docs/20, and Swift tools version notes exist
**And** no application windows, capture, or SQLite yet
**And** docs/20 versions remain the recorded host baseline

**Failure / recovery:**
If a pin disagrees with docs/20, fail the story and update the pin or the bootstrap record — do not silently raise macOS deployment target (ADR-002).

**Security / privacy / diagnostics / a11y / i18n / data:**
No secrets in pin files. No telemetry SDK. No user-facing strings.

**Automated verification:**
- `python3 tooling/planning-checks.py`
- `npx --yes markdownlint-cli2 'docs/**/*.md' README.md AGENTS.md`
- `test -f package.json`
- `test -f rust-toolchain.toml`

**Signed-build / manual evidence:**
file existence only; no signed app

**Non-goals:**
Tauri window, UI, native capture, CI cloud, Intel universal2


### Story 1.2: Scaffold pnpm Turbo workspace

As a developer,
I want pnpm-workspace and Turbo with empty packages,
So that JS work has a graph before product code.

**Requirements:** SEC-001, I18N-001 (package slots)
**ADRs:** ADR-003, ADR-017
**Dependencies:** Story 1.1
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** pinned package manager
**When** workspace is scaffolded
**Then** `pnpm-workspace.yaml`, `turbo.json`, `packages/ui`, `packages/contracts`, `packages/i18n`, `packages/test-support`, `apps/desktop` exist
**And** no product features
**And** Turbo cache disabled is not required yet because no signing tasks exist

**Failure / recovery:**
If pnpm install needs network, use lockfile creation once then keep lockfile; do not add remote fonts or CDN.

**Security / privacy / diagnostics / a11y / i18n / data:**
packages/ui must not own product state. No raw user strings in TSX.

**Automated verification:**
- `python3 tooling/planning-checks.py`
- `npx --yes markdownlint-cli2 'docs/**/*.md' README.md AGENTS.md`
- `pnpm install`
- `pnpm exec turbo run lint --force || true`

**Signed-build / manual evidence:**
local install log

**Non-goals:**
shadcn components, Tauri commands, capture


### Story 1.3: Scaffold Cargo workspace

As a developer,
I want Cargo workspace crates with rustfmt and Clippy deny-warnings,
So that Rust domain has a home.

**Requirements:** DAT-001 (slot), SEC-002 (slot)
**ADRs:** ADR-003, ADR-008, ADR-010
**Dependencies:** Story 1.1
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** toolchain pin
**When** crates are created
**Then** `bronze-domain`, `bronze-capture`, `bronze-storage`, `bronze-settings`, `bronze-diagnostics`, `bronze-platform`, `bronze-platform-macos` compile empty
**And** Clippy `-D warnings` on workspace
**And** no SQL, AX, or Tauri commands yet

**Failure / recovery:**
Compilation failure blocks. Do not allow unused-mut warnings.

**Security / privacy / diagnostics / a11y / i18n / data:**
No macOS framework imports outside bronze-platform-macos.

**Automated verification:**
- `cargo fmt --check`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`

**Signed-build / manual evidence:**
local cargo logs

**Non-goals:**
schema, ABI, event tap


### Story 1.4: Scaffold Swift package

As a developer,
I want BronzeNative Swift package skeleton,
So that native work has a package before ABI.

**Requirements:** CAP-001 (slot)
**ADRs:** ADR-004
**Dependencies:** Story 1.1
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** Xcode present
**When** Package.swift is added under native/macos/BronzeNative
**Then** package builds a static library with ABI version placeholder
**And** no event tap, AX, or AppKit window code yet

**Failure / recovery:**
If SwiftPM cannot build, stop; do not embed source in Tauri without a package.

**Security / privacy / diagnostics / a11y / i18n / data:**
No logging of strings. No JS surface.

**Automated verification:**
- `swift build --package-path native/macos/BronzeNative`

**Signed-build / manual evidence:**
local swift build

**Non-goals:**
CGEventTap, AX, status item


### Story 1.5: Empty Tauri app with CSP deny

As a developer,
I want a Tauri 2 macOS app that loads packaged UI with default-deny capabilities,
So that security baseline exists before features.

**Requirements:** SEC-001, SEC-002, SEC-003, SEC-004
**ADRs:** ADR-003, ADR-010, ADR-017
**Dependencies:** Stories 1.2 and 1.3
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** JS and Cargo workspaces
**When** Tauri app is created
**Then** production CSP has no remote origins, no eval, no frames
**And** capabilities exist for quick, library, settings, onboarding and grant no shell/fs/http/sql
**And** DevTools disabled in release config
**And** forbidden-command test from a fixture window fails closed

**Failure / recovery:**
Any allowed shell/fs/http plugin is S0; remove it.

**Security / privacy / diagnostics / a11y / i18n / data:**
Zero runtime network in default build. No arbitrary path from WebView.

**Automated verification:**
- `pnpm exec tauri build --debug --no-bundle || cargo test -p bronze-desktop -- --nocapture`
- `rg -n 'shell|http|sql' apps/desktop/src-tauri/capabilities || true`

**Signed-build / manual evidence:**
capability JSON review; not notarized

**Non-goals:**
capture, queue UI, signing identity


### Story 1.6: i18n catalogs and pseudo-locales

As a developer,
I want en, en-XA, ar-XB catalogs and missing-key CI,
So that later UI cannot hard-code copy.

**Requirements:** I18N-001, I18N-002, I18N-003, G-06
**ADRs:** ADR-014
**Dependencies:** Story 1.2
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** packages/i18n exists
**When** catalogs and validators are added
**Then** build fails on missing keys or placeholder mismatch
**And** script-preserving fallback unit tests include zh-Hant-HK → zh-Hant → zh → en
**And** no user-facing English literals in new TSX

**Failure / recovery:**
Concatenated sentences fail CI.

**Security / privacy / diagnostics / a11y / i18n / data:**
User content never enters catalogs. ICU plurals required for counts.

**Automated verification:**
- `pnpm --filter @bronze/i18n test`
- `pnpm --filter @bronze/i18n validate`

**Signed-build / manual evidence:**
unit tests only; no linguistic QA

**Non-goals:**
human translation, native InfoPlist strings (later story)


### Story 1.7: shadcn React Aria foundation

As a developer,
I want owned shadcn components on React Aria with tokens and visible focus,
So that UI stories do not mix primitive bases.

**Requirements:** A11Y-001, A11Y-002, A11Y-003
**ADRs:** ADR-012, ADR-013
**Dependencies:** Stories 1.2 and 1.6
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** packages/ui exists
**When** shadcn init uses React Aria base
**Then** Button/TextField/Dialog primitives have role/name/focus tests
**And** no clickable div
**And** focus-visible not removed
**And** Reduce Motion / Increase Contrast CSS hooks exist

**Failure / recovery:**
If React Aria base cannot initialize, stop for DG-06 rather than mixing Radix.

**Security / privacy / diagnostics / a11y / i18n / data:**
axe on default/empty/focus states. No color-only status.

**Automated verification:**
- `pnpm --filter @bronze/ui test`
- `pnpm --filter @bronze/ui lint`

**Signed-build / manual evidence:**
component tests; not VoiceOver

**Non-goals:**
queue cards, virtualized lists, contenteditable


### Story 1.8: Workspace verify aggregate

As a developer,
I want one local verify command covering format, lint, types, unit, i18n, planning-checks,
So that later stories have a gate.

**Requirements:** G-01 process, SEC-001 config
**ADRs:** ADR-003
**Dependencies:** Stories 1.2–1.7
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** workspaces exist
**When** root verify script is added
**Then** `pnpm verify` runs Biome/tsc/Vitest/i18n validate/cargo test/planning-checks
**And** signing/TCC tasks are not Turbo-cached
**And** verify fails if any required step fails

**Failure / recovery:**
Do not weaken a failing check to go green.

**Security / privacy / diagnostics / a11y / i18n / data:**
planning-checks remain required.

**Automated verification:**
- `pnpm verify`

**Signed-build / manual evidence:**
command output

**Non-goals:**
GitHub Actions beyond local script


## Epic 2: In-process Swift bridge

Tauri executable links BronzeNative and completes ABI round trips without JS seeing Swift.

### Story 2.1: Versioned C ABI types

As a developer,
I want fixed-width ABI version, tagged enums, pointer-plus-length UTF-8,
So that Rust and Swift share one contract.

**Requirements:** CAP-004, SEC-002
**ADRs:** ADR-004
**Dependencies:** Story 1.4
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** BronzeNative package exists
**When** ABI headers/module are added
**Then** version check fails closed on mismatch
**And** no NUL-terminated string reliance
**And** no unwind across boundary documented in tests

**Failure / recovery:**
Invalid UTF-8 rejected. Double completion forbidden.

**Security / privacy / diagnostics / a11y / i18n / data:**
Sensitive buffers have no debug description.

**Automated verification:**
- `swift test --package-path native/macos/BronzeNative`

**Signed-build / manual evidence:**
unit tests

**Non-goals:**
AX, event tap, pasteboard


### Story 2.2: Rust macOS façade

As a developer,
I want bronze-platform-macos safe façade over the ABI,
So that domain crates never call Swift directly.

**Requirements:** CAP-004
**ADRs:** ADR-004
**Dependencies:** Stories 1.3 and 2.1
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** ABI types exist
**When** façade crate is implemented
**Then** init/shutdown and version query work
**And** bronze-domain has no macOS imports

**Failure / recovery:**
Panic at FFI boundary is contained.

**Security / privacy / diagnostics / a11y / i18n / data:**
No content logging.

**Automated verification:**
- `cargo test -p bronze-platform-macos`
- `cargo clippy -p bronze-platform-macos -- -D warnings`

**Signed-build / manual evidence:**
unit tests

**Non-goals:**
real AX queries


### Story 2.3: Link static library into Tauri

As a developer,
I want the debug app links BronzeNative in-process,
So that one TCC subject exists.

**Requirements:** SEC-005 (identity later), CAP-001 slot
**ADRs:** ADR-004, ADR-011
**Dependencies:** Stories 1.5 and 2.2
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** façade and Tauri app exist
**When** the app is built
**Then** release-like bundle contains no helper executable
**And** startup calls ABI version check

**Failure / recovery:**
Link failure blocks; do not switch to a sidecar.

**Security / privacy / diagnostics / a11y / i18n / data:**
One process identity.

**Automated verification:**
- `cargo test -p bronze-desktop`
- `swift build --package-path native/macos/BronzeNative`

**Signed-build / manual evidence:**
local debug build; not Developer ID

**Non-goals:**
notarization, App Group


### Story 2.4: ABI ownership tests

As a developer,
I want empty, embedded NUL, invalid UTF-8, large payload, cancel, shutdown-race tests,
So that bridge memory is proven before capture.

**Requirements:** CAP-004, SEC-006
**ADRs:** ADR-004, ADR-015
**Dependencies:** Story 2.3
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** linked app
**When** ABI conformance suite runs
**Then** each case completes exactly once or cancels
**And** 10k round-trip smoke optional if runtime allows; otherwise document follow-up

**Failure / recovery:**
Leak or double-free fails the story.

**Security / privacy / diagnostics / a11y / i18n / data:**
No content in diagnostics.

**Automated verification:**
- `swift test --package-path native/macos/BronzeNative`
- `cargo test -p bronze-platform-macos`

**Signed-build / manual evidence:**
tests; sanitizers when compatible

**Non-goals:**
capture matrix


## Epic 3: Capture coordinator and AX-first selection

Every observed trigger gets an ID and one terminal outcome; AX is primary; synthetic clipboard stays off.

### Story 3.1: Permission snapshot enum

As a developer,
I want closed permission states without prompting,
So that health UI later has a typed model.

**Requirements:** SET-003, SET-004, CAP-010
**ADRs:** ADR-005, ADR-015
**Dependencies:** Story 1.3
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** crates exist
**When** permission enum is implemented
**Then** values are unknown|not_requested|denied|granted_unverified|healthy|degraded|unavailable|requires_relaunch
**And** preflight APIs are wrapped but not called to prompt
**And** granted is not treated as healthy

**Failure / recovery:**
Unknown platform result maps to unknown/degraded, never healthy.

**Security / privacy / diagnostics / a11y / i18n / data:**
No key/content in snapshots.

**Automated verification:**
- `cargo test -p bronze-settings permission`
- `cargo test -p bronze-platform-macos permission`

**Signed-build / manual evidence:**
unit tests; not TCC UI

**Non-goals:**
System Settings deep link UI


### Story 3.2: Double-tap FSM tests

As a developer,
I want pure modifier FSM with property tests,
So that gesture logic is proven without the tap.

**Requirements:** CAP-002, A11Y-001
**ADRs:** ADR-005
**Dependencies:** Story 1.4
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** Swift package exists
**When** FSM is implemented with table/property tests
**Then** valid pair emits one trigger; invalid sequences emit zero
**And** disabled by default
**And** timing window allows ≥500 ms
**And** no key stream retained

**Failure / recovery:**
Stuck state or wall-clock dependence fails.

**Security / privacy / diagnostics / a11y / i18n / data:**
Non-timed routes remain required. No character logging.

**Automated verification:**
- `swift test --package-path native/macos/BronzeNative --filter FSM`

**Signed-build / manual evidence:**
unit tests; not 1000 physical trials

**Non-goals:**
live CGEventTap


### Story 3.3: Listen-only event tap thread

As a developer,
I want session listenOnly tap on a dedicated run-loop thread,
So that optional gesture can be enabled later.

**Requirements:** CAP-002, CAP-004
**ADRs:** ADR-004, ADR-005
**Dependencies:** Stories 2.3 and 3.2
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** FSM exists
**When** tap thread is implemented
**Then** callback does no AX, DB, window, clipboard, or log work
**And** SPSC is single-producer
**And** tap disable resets FSM

**Failure / recovery:**
If Input Monitoring is denied, degrade; keep chord/menu routes (even if unimplemented UI).

**Security / privacy / diagnostics / a11y / i18n / data:**
Never suppress events. No keycodes persisted.

**Automated verification:**
- `swift test --package-path native/macos/BronzeNative --filter EventTap`

**Signed-build / manual evidence:**
unit/fake tests; physical 1k trials are human

**Non-goals:**
prompting TCC


### Story 3.4: Atomic ingress snapshot

As a developer,
I want CaptureIngressContext seqlock snapshot,
So that queued work cannot retarget.

**Requirements:** CAP-004, CAP-008, CAP-009, QUE-001
**ADRs:** ADR-005, ADR-006
**Dependencies:** Stories 2.2 and 3.3
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** native façade exists
**When** ingress context is implemented
**Then** snapshot includes target PID, bundle token, activation generation, destination UUID, accept-capture generation, policy/settings revisions, route, monotonic time
**And** inconsistent read returns context_unavailable
**And** focused element is not in event-tap snapshot

**Failure / recovery:**
Never substitute current frontmost app.

**Security / privacy / diagnostics / a11y / i18n / data:**
No titles/URLs in ingress.

**Automated verification:**
- `cargo test -p bronze-capture ingress`
- `swift test --package-path native/macos/BronzeNative --filter Ingress`

**Signed-build / manual evidence:**
unit tests

**Non-goals:**
real AX focus identity (next stories)


### Story 3.5: Serial capture coordinator

As a developer,
I want monotonic request IDs and exactly one terminal outcome,
So that no silent drops.

**Requirements:** CAP-004, G-01, G-02
**ADRs:** ADR-006, ADR-015
**Dependencies:** Story 3.4
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** ingress exists
**When** coordinator is implemented
**Then** 20 synthetic triggers preserve order and all terminate
**And** overflow emits trigger_queue_overflow per missing ID
**And** success feedback cannot fire before persist hook

**Failure / recovery:**
Nonterminal request fails the suite.

**Security / privacy / diagnostics / a11y / i18n / data:**
Diagnostics independent of content transaction.

**Automated verification:**
- `cargo test -p bronze-capture coordinator`

**Signed-build / manual evidence:**
unit/property tests

**Non-goals:**
real AX


### Story 3.6: AX provider with fakes

As a developer,
I want AX-first provider using fixtures,
So that secure fields never leak.

**Requirements:** CAP-005, CAP-006, CAP-007, CAP-009
**ADRs:** ADR-006
**Dependencies:** Story 3.5
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** coordinator exists
**When** AX provider is implemented against fakes
**Then** secure/unknown protection fail closed with zero content in ABI/log/store
**And** whitespace preserved
**And** exclusion runs before content query
**And** empty/zero-width is no_selection without trim

**Failure / recovery:**
Any content on protected_content is S0.

**Security / privacy / diagnostics / a11y / i18n / data:**
Diagnostics have no text/title/URL/hash of secrets.

**Automated verification:**
- `cargo test -p bronze-capture ax`
- `swift test --package-path native/macos/BronzeNative --filter AX`

**Signed-build / manual evidence:**
fixtures; not real Safari/Chrome matrix

**Non-goals:**
synthetic Cmd-C


### Story 3.7: Manual clipboard import

As a developer,
I want explicit Create from Clipboard,
So that users can capture without AX.

**Requirements:** CAP-003, CAP-005
**ADRs:** ADR-006
**Dependencies:** Story 3.5
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** coordinator exists
**When** manual clipboard command is added
**Then** it reads text only after explicit action
**And** synthetic fallback remains off
**And** stale generation fails clipboard_changed

**Failure / recovery:**
Do not restore clipboard. Do not ingest files/images.

**Security / privacy / diagnostics / a11y / i18n / data:**
Shared pasteboard disclosure not required for manual path beyond help later.

**Automated verification:**
- `cargo test -p bronze-capture clipboard_manual`

**Signed-build / manual evidence:**
unit tests

**Non-goals:**
synthetic Cmd-C enablement


### Story 3.8: Content-free diagnostic schema

As a developer,
I want typed diagnostic_events without content fields,
So that CAP-010 is structurally true.

**Requirements:** CAP-010, SEC-006, SUP-002
**ADRs:** ADR-015
**Dependencies:** Stories 1.3 and 3.5
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** diagnostics crate exists
**When** schema is implemented
**Then** type system rejects String payloads for selected text
**And** seeded secret scan of logs passes
**And** diagnostic write failure does not roll back a saved item fake

**Failure / recovery:**
Content type in schema is S0.

**Security / privacy / diagnostics / a11y / i18n / data:**
Support bundle preview comes in a later epic.

**Automated verification:**
- `cargo test -p bronze-diagnostics`

**Signed-build / manual evidence:**
type/unit tests

**Non-goals:**
export UI


### Story 3.9: TCC grant deny revoke evidence

As a operator,
I want signed-build TCC matrix recorded,
So that permissions are not claimed from unit tests.

**Requirements:** SET-003, SET-004, CAP-001, CAP-002
**ADRs:** ADR-005, ADR-011
**Dependencies:** Stories 3.1–3.3
**Blocking gates:** human-only evidence; DG as named

**Acceptance Criteria:**

**Given** a signed local identity build exists
**When** a human runs grant/deny/revoke/relaunch
**Then** results are recorded as blocked-human-validation until executed
**And** this story cannot be marked done by an agent without that evidence file

**Failure / recovery:**
Do not fabricate.

**Security / privacy / diagnostics / a11y / i18n / data:**
No content logs during tests.

**Automated verification:**
- `test ! -f docs/evidence/tcc-matrix.md && echo pending`

**Signed-build / manual evidence:**
blocked-human-validation: never fabricate VoiceOver, Voice Control, Switch Control, TCC, signed capture, notarization, or linguistic QA evidence.

**Non-goals:**
agent-completed TCC

**Human gate:** blocked-human-validation: never fabricate VoiceOver, Voice Control, Switch Control, TCC, signed capture, notarization, or linguistic QA evidence.


### Story 3.10: Signed AX source matrix

As a operator,
I want TextEdit/Safari/Chrome/editor/terminal/secure-field matrix,
So that supported sources are evidence-backed.

**Requirements:** CAP-005, CAP-006, G-01, DG-03
**ADRs:** ADR-006
**Dependencies:** Story 3.6
**Blocking gates:** human-only evidence; DG as named

**Acceptance Criteria:**

**Given** AX provider exists
**When** human runs the mandatory source matrix on a signed build
**Then** each row records provider path and limitation
**And** story stays blocked-human-validation until the matrix file exists

**Failure / recovery:**
Do not count unknown apps toward 99.9%.

**Security / privacy / diagnostics / a11y / i18n / data:**
Secure fixtures must show zero content.

**Automated verification:**
- `test ! -f docs/evidence/ax-matrix.md && echo pending`

**Signed-build / manual evidence:**
blocked-human-validation: never fabricate VoiceOver, Voice Control, Switch Control, TCC, signed capture, notarization, or linguistic QA evidence.

**Non-goals:**
agent-claimed compatibility

**Human gate:** blocked-human-validation: never fabricate VoiceOver, Voice Control, Switch Control, TCC, signed capture, notarization, or linguistic QA evidence.


## Epic 4: Transactional store and portability

SQLite WAL, migrations, undo, backup, and deterministic export without WebView SQL.

### Story 4.1: Domain entities and lifecycle

As a developer,
I want section/item entities with closed lifecycle and content_language,
So that UI cannot invent states.

**Requirements:** QUE-001, QUE-002, QUE-003, I18N-003, DAT-001
**ADRs:** ADR-008, ADR-014
**Dependencies:** Story 1.3
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** bronze-domain exists
**When** entities are implemented
**Then** lifecycle is queued|copied|active|done|skipped|trashed with a transition table
**And** content_language is BCP 47 or und defaulting to und
**And** invalid transitions fail

**Failure / recovery:**
Silent trim/normalize of body fails tests.

**Security / privacy / diagnostics / a11y / i18n / data:**
UTC timestamps. Locale-neutral enums.

**Automated verification:**
- `cargo test -p bronze-domain`

**Signed-build / manual evidence:**
unit tests

**Non-goals:**
SQLite yet


### Story 4.2: Schema v1 and migrations

As a developer,
I want versioned checksummed migrations and WAL,
So that data survives upgrades.

**Requirements:** DAT-001, G-03
**ADRs:** ADR-008; ADR-009 Proposed so locator is an interface
**Dependencies:** Story 4.1
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** entities exist
**When** schema v1 is applied
**Then** tables match docs/08 including command_receipts and diagnostic_events
**And** backup-before-migration hook exists even if backup story is next
**And** WebView has no SQL

**Failure / recovery:**
Failed migration keeps original DB and enters read-only.

**Security / privacy / diagnostics / a11y / i18n / data:**
0600/0700 modes where possible. No iCloud default path.

**Automated verification:**
- `cargo test -p bronze-storage migrate`

**Signed-build / manual evidence:**
migration fixtures

**Non-goals:**
App Group container decision


### Story 4.3: Command receipts and revisions

As a developer,
I want idempotent command IDs and optimistic revisions,
So that retries do not duplicate work.

**Requirements:** DAT-001, QUE-002
**ADRs:** ADR-008
**Dependencies:** Story 4.2
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** schema exists
**When** command API is implemented
**Then** duplicate ID inside window reconstructs prior result
**And** expired ID returns idempotency_expired and does not execute
**And** receipts contain no body

**Failure / recovery:**
Stale revision returns typed conflict.

**Security / privacy / diagnostics / a11y / i18n / data:**
Content-free receipts.

**Automated verification:**
- `cargo test -p bronze-storage receipts`

**Signed-build / manual evidence:**
unit tests

**Non-goals:**
UI conflict dialog


### Story 4.4: Undo trash and purge

As a developer,
I want tombstones, undo inverses, dependency-safe purge,
So that deletes are recoverable.

**Requirements:** QUE-006, DAT-004
**ADRs:** ADR-008
**Dependencies:** Story 4.3
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** items persist
**When** trash/undo/purge are implemented
**Then** purge never cascade-deletes non-trashed descendants
**And** default trash 30 days
**And** forensic erasure is not claimed

**Failure / recovery:**
Empty Trash without confirmation API is forbidden.

**Security / privacy / diagnostics / a11y / i18n / data:**
FTS rows removed on purge when FTS exists.

**Automated verification:**
- `cargo test -p bronze-storage undo`
- `cargo test -p bronze-domain purge`

**Signed-build / manual evidence:**
unit tests

**Non-goals:**
Empty Trash UI


### Story 4.5: Online backup restore

As a developer,
I want SQLite online backup API and verified restore,
So that DAT-002 holds.

**Requirements:** DAT-002
**ADRs:** ADR-008
**Dependencies:** Story 4.2
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** schema exists
**When** backup/restore are implemented
**Then** schedule enum is daily|weekly only
**And** restore snapshots current DB first
**And** integrity failure does not swap

**Failure / recovery:**
File-copy of live WAL is forbidden.

**Security / privacy / diagnostics / a11y / i18n / data:**
Backups inherit 0600. No network.

**Automated verification:**
- `cargo test -p bronze-storage backup`

**Signed-build / manual evidence:**
fault tests

**Non-goals:**
backup UI


### Story 4.6: Deterministic export import

As a developer,
I want JSON archive plus Markdown with contentLanguage,
So that users can leave.

**Requirements:** DAT-003, SET-001, I18N-003
**ADRs:** ADR-008, ADR-014
**Dependencies:** Stories 4.2 and 4.1
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** items persist
**When** export/import run
**Then** manifest is locale-neutral and sorted
**And** preview warns that item bodies may contain secrets
**And** path traversal archives fail
**And** settings export excludes diagnostics/paths/tokens

**Failure / recovery:**
Silent overwrite forbidden.

**Security / privacy / diagnostics / a11y / i18n / data:**
No machine paths. Hostile corpus required.

**Automated verification:**
- `cargo test -p bronze-storage export`

**Signed-build / manual evidence:**
golden/fuzz tests

**Non-goals:**
file picker UI


### Story 4.7: Search placeholder pending ADR-018

As a developer,
I want FTS hook that does not claim locale semantics,
So that QUE-007 is not falsely completed.

**Requirements:** QUE-007, G-06, DG-10
**ADRs:** ADR-018 Proposed
**Dependencies:** Story 4.2
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** items persist
**When** a naive FTS or application filter exists
**Then** story documents ADR-018 as blocking complete locale search
**And** raw body unchanged
**And** query never enters diagnostics

**Failure / recovery:**
Do not mark QUE-007 done.

**Security / privacy / diagnostics / a11y / i18n / data:**
FTS is sensitive data.

**Automated verification:**
- `cargo test -p bronze-storage search_placeholder`

**Signed-build / manual evidence:**
unit tests; tokenizer gate open

**Non-goals:**
final tokenizer


## Epic 5: Native shell and accessible feedback

Status menu, activating panel, focus restore, and native display-preference bridge.

### Story 5.1: Status item and menu

As a user,
I want an accessible status menu with Capture, New Note, Show, Settings, Quit,
So that I can work without a global hook.

**Requirements:** CAP-003, WIN-004, A11Y-001, I18N-001
**ADRs:** ADR-007, ADR-014
**Dependencies:** Stories 1.5, 1.6, 2.3
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** empty Tauri app
**When** status item is added
**Then** menu items have localized accessible names
**And** Capture uses last-external-target snapshot contract even if capture is stubbed
**And** menu works when event tap is off

**Failure / recovery:**
Missing accessible name fails.

**Security / privacy / diagnostics / a11y / i18n / data:**
Native strings from same catalog ownership.

**Automated verification:**
- `cargo test -p bronze-platform-macos status_item`
- `pnpm verify`

**Signed-build / manual evidence:**
unit; VoiceOver is human

**Non-goals:**
real capture success


### Story 5.2: Quick panel window and physical edge

As a user,
I want to summon a key-capable panel on a physical left/right/top edge,
So that I can see the queue.

**Requirements:** WIN-001, WIN-002, WIN-005
**ADRs:** ADR-007
**Dependencies:** Stories 1.5 and 5.1
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** Tauri windows exist
**When** quick panel is implemented
**Then** stored edge is physical not RTL-leading
**And** work area clamping exists
**And** panel is activating
**And** no focus trap

**Failure / recovery:**
Nonactivating NSPanel is forbidden.

**Security / privacy / diagnostics / a11y / i18n / data:**
RTL does not mirror stored edge.

**Automated verification:**
- `cargo test -p bronze-desktop window_edge`
- `pnpm --filter desktop test`

**Signed-build / manual evidence:**
unit; multi-display matrix is human

**Non-goals:**
full queue CRUD


### Story 5.3: Focus restore and capture-only silence

As a user,
I want capture-only success to keep source focus,
So that I am not yanked into Bronze.

**Requirements:** WIN-003, CAP-004, A11Y-006
**ADRs:** ADR-007
**Dependencies:** Stories 3.5 and 5.2
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** panel and coordinator exist
**When** focus policy is implemented
**Then** capture-only does not steal source focus
**And** native announcement API is called for hidden WebView case (can be tested with fake)
**And** Escape restores prior focus where safe

**Failure / recovery:**
Hidden WKWebView live region is not accepted as the only feedback.

**Security / privacy / diagnostics / a11y / i18n / data:**
Announcement text localized.

**Automated verification:**
- `cargo test -p bronze-capture focus_policy`
- `pnpm --filter desktop test`

**Signed-build / manual evidence:**
unit/fake; VoiceOver human

**Non-goals:**
VoiceOver sign-off


### Story 5.4: Native display preference bridge

As a user,
I want Reduce Motion, Reduce Transparency, Increase Contrast, Differentiate Without Color applied live,
So that OS preferences are honored.

**Requirements:** A11Y-003, WIN-004
**ADRs:** ADR-012, ADR-013
**Dependencies:** Stories 1.7 and 2.3
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** native façade exists
**When** NSWorkspace display options are read and observed
**Then** one typed snapshot is published to WebViews
**And** app overrides only strengthen
**And** tokens update without restart

**Failure / recovery:**
Weakening system Reduce Motion is forbidden.

**Security / privacy / diagnostics / a11y / i18n / data:**
No color-only status.

**Automated verification:**
- `cargo test -p bronze-platform-macos display_prefs`
- `pnpm --filter @bronze/ui test`

**Signed-build / manual evidence:**
unit; manual OS toggle is human

**Non-goals:**
full AT matrix


### Story 5.5: Spaces and display matrix

As a operator,
I want manual Spaces/full-screen/multi-display evidence,
So that WIN-002 is not claimed from unit tests.

**Requirements:** WIN-001, WIN-002, DG-05
**ADRs:** ADR-007
**Dependencies:** Story 5.2
**Blocking gates:** human-only evidence; DG as named

**Acceptance Criteria:**

**Given** panel exists
**When** human runs display matrix
**Then** story remains blocked-human-validation until evidence exists

**Failure / recovery:**
Do not fabricate.

**Security / privacy / diagnostics / a11y / i18n / data:**
n/a

**Automated verification:**
- `test ! -f docs/evidence/display-matrix.md && echo pending`

**Signed-build / manual evidence:**
blocked-human-validation: never fabricate VoiceOver, Voice Control, Switch Control, TCC, signed capture, notarization, or linguistic QA evidence.

**Non-goals:**
agent-done display matrix

**Human gate:** blocked-human-validation: never fabricate VoiceOver, Voice Control, Switch Control, TCC, signed capture, notarization, or linguistic QA evidence.


## Epic 6: Queue and library UI

Users can add, order, copy, complete, and search items from semantic UI.

### Story 6.1: Composer add item

As a user,
I want to add a multiline note with Cmd-Enter,
So that I can park a prompt.

**Requirements:** QUE-002, CAP-003, I18N-003, A11Y-002
**ADRs:** ADR-012, ADR-014
**Dependencies:** Stories 1.7, 4.3, 5.2
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** panel and store exist
**When** composer is implemented
**Then** Cmd-Enter adds; Enter during isComposing does not
**And** failure retains draft
**And** content_language defaults to und
**And** strings from catalog

**Failure / recovery:**
Clearing composer before persist is forbidden.

**Security / privacy / diagnostics / a11y / i18n / data:**
IME-safe. Role/name tests. axe on empty/error.

**Automated verification:**
- `pnpm --filter desktop test -- composer`
- `pnpm verify`

**Signed-build / manual evidence:**
component tests; IME human later

**Non-goals:**
Markdown preview fetch


### Story 6.2: Item list lifecycle and menus

As a user,
I want to edit, complete, skip, and trash items from buttons/menus,
So that the queue is usable without drag.

**Requirements:** QUE-002, QUE-003, QUE-006, A11Y-002, A11Y-004, A11Y-006
**ADRs:** ADR-012
**Dependencies:** Story 6.1
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** items exist
**When** cards use semantic list/article and real buttons
**Then** Move Up/Down exist; drag is optional
**And** accessible name contains visible label
**And** keyboard operates all actions

**Failure / recovery:**
Clickable div fails the story.

**Security / privacy / diagnostics / a11y / i18n / data:**
200% text still usable. No hover-only actions.

**Automated verification:**
- `pnpm --filter desktop test -- queue`
- `pnpm --filter desktop test -- a11y`

**Signed-build / manual evidence:**
axe/keyboard tests

**Non-goals:**
VoiceOver sign-off


### Story 6.3: Sections and library window

As a user,
I want active section in the panel and a library window for archive/trash/search,
So that capabilities stay least-privilege.

**Requirements:** QUE-001, QUE-008, WIN-005, SEC-002
**ADRs:** ADR-010, ADR-007
**Dependencies:** Stories 5.2 and 6.2
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** queue UI exists
**When** library window is added
**Then** quick cannot import/export/backup
**And** library can paginate and archive
**And** empty/loading/read-only states exist

**Failure / recovery:**
Capability leak is S0.

**Security / privacy / diagnostics / a11y / i18n / data:**
Per-window commands tested negative.

**Automated verification:**
- `cargo test -p bronze-desktop capabilities`
- `pnpm --filter desktop test -- library`

**Signed-build / manual evidence:**
IPC denial tests

**Non-goals:**
import UI (epic 7)


### Story 6.4: Output profiles and copy

As a user,
I want to copy items with a named profile,
So that pasteboard output is deterministic.

**Requirements:** QUE-004, QUE-005
**ADRs:** ADR-008
**Dependencies:** Stories 4.3 and 6.2
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** items exist
**When** copy command runs
**Then** profile is sole postCopyAction/advancePolicy authority
**And** lifecycle changes only after pasteboard success
**And** default copied+keep
**And** no synthetic paste
**And** exact preview uses hostile sample in settings later; panel preview allowed under queue capability

**Failure / recovery:**
Marking done before pasteboard success is forbidden.

**Security / privacy / diagnostics / a11y / i18n / data:**
Prompt-block delimits untrusted context. No HTML fetch.

**Automated verification:**
- `cargo test -p bronze-domain formatters`
- `cargo test -p bronze-desktop copy`

**Signed-build / manual evidence:**
golden tests

**Non-goals:**
auto-paste


### Story 6.5: Local search UI

As a user,
I want to search items locally,
So that I can find parked text.

**Requirements:** QUE-007, A11Y-002
**ADRs:** ADR-018 Proposed
**Dependencies:** Stories 4.7 and 6.3
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** library exists
**When** search UI is added
**Then** results announce count without putting query in diagnostics
**And** QUE-007 locale tokenizer remains partial until ADR-018

**Failure / recovery:**
Do not claim G-06 search complete.

**Security / privacy / diagnostics / a11y / i18n / data:**
Query stays on device.

**Automated verification:**
- `pnpm --filter desktop test -- search`

**Signed-build / manual evidence:**
component tests

**Non-goals:**
final FTS semantics


## Epic 7: Settings shortcuts and portability UI

Searchable settings, shortcut registration, permission health, backup/export UI, and i18n completion.

### Story 7.1: Settings window schema

As a user,
I want searchable grouped settings with per-field reset,
So that I can change behavior safely.

**Requirements:** SET-001, WIN-005
**ADRs:** ADR-016
**Dependencies:** Stories 1.7 and 5.2
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** settings capability exists
**When** settings window is implemented
**Then** schema matches docs/12 including ShortcutActionId and daily|weekly backup
**And** export preview flags sensitive literals
**And** no credentials/tokens/paths exported

**Failure / recovery:**
Saving invalid shortcut without rollback fails.

**Security / privacy / diagnostics / a11y / i18n / data:**
Keyboard accessible. Catalog strings.

**Automated verification:**
- `pnpm --filter desktop test -- settings`
- `cargo test -p bronze-settings`

**Signed-build / manual evidence:**
component/unit

**Non-goals:**
shortcut recorder (next)


### Story 7.2: Shortcut recorder and registry

As a user,
I want to bind every ShortcutActionId without losing the old chord on failure,
So that triggers are configurable.

**Requirements:** SET-002, CAP-001, CAP-002, I18N-004, A11Y-001
**ADRs:** ADR-016, ADR-005
**Dependencies:** Story 7.1
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** settings exist
**When** recorder is implemented
**Then** all registry actions persist in shortcuts table
**And** standardChord is capture.selection view only
**And** cannot disable chord+menu+manual together
**And** IME/VO chords not swallowed

**Failure / recovery:**
Failed registration retains old binding.

**Security / privacy / diagnostics / a11y / i18n / data:**
Spoken names localized. Test mode skippable.

**Automated verification:**
- `cargo test -p bronze-settings shortcuts`
- `pnpm --filter desktop test -- shortcuts`

**Signed-build / manual evidence:**
unit; layout matrix human

**Non-goals:**
claiming conflict-free globally


### Story 7.3: Permission health center

As a user,
I want distinct permission rows with why, retest, and alternatives,
So that denial is recoverable.

**Requirements:** SET-003, SET-004, CAP-003
**ADRs:** ADR-005
**Dependencies:** Stories 3.1 and 7.1
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** permission enum exists
**When** health UI is implemented
**Then** each capability is independent
**And** Screen Recording shown as Not used
**And** denial leaves manual composer

**Failure / recovery:**
Launch-loop prompting forbidden.

**Security / privacy / diagnostics / a11y / i18n / data:**
No content in self-test.

**Automated verification:**
- `pnpm --filter desktop test -- permissions`

**Signed-build / manual evidence:**
component tests; real TCC human

**Non-goals:**
completing Story 3.9


### Story 7.4: Backup export import UI

As a user,
I want Back Up Now, restore preview, and export/import flows,
So that data is portable.

**Requirements:** DAT-002, DAT-003, SET-001, QUE-008
**ADRs:** ADR-008
**Dependencies:** Stories 4.5, 4.6, 6.3
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** storage APIs exist
**When** library/settings UI calls them
**Then** native picker is Rust-owned
**And** automatic schedule cannot be off
**And** secret-content warning on queue export

**Failure / recovery:**
WebView path strings rejected.

**Security / privacy / diagnostics / a11y / i18n / data:**
Capability split library vs settings.

**Automated verification:**
- `pnpm --filter desktop test -- portability`
- `cargo test -p bronze-desktop portability`

**Signed-build / manual evidence:**
E2E mocked picker

**Non-goals:**
cloud backup


### Story 7.5: Native and WebView catalog parity

As a developer,
I want native menu/InfoPlist strings in the same glossary,
So that i18n is complete for advertised locales.

**Requirements:** I18N-001, I18N-002, I18N-003, I18N-004, G-06
**ADRs:** ADR-014
**Dependencies:** Stories 1.6, 5.1, 7.1
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** catalogs exist
**When** native strings are extracted
**Then** en, en-XA, ar-XB cover WebView and native
**And** RTL smoke for panel chrome
**And** per-item lang/dir on cards

**Failure / recovery:**
Hard-coded user prose fails CI.

**Security / privacy / diagnostics / a11y / i18n / data:**
Pseudo-locales required. Human linguistic QA remains blocked.

**Automated verification:**
- `pnpm --filter @bronze/i18n validate`
- `pnpm --filter desktop test -- i18n`

**Signed-build / manual evidence:**
CI; Arabic/Japanese human later

**Non-goals:**
shipping advertised human locales without QA


## Epic 8: Visual refinement

Bronze visual identity without cloning Copper, still meeting contrast and reflow.

### Story 8.1: Tokenized bronze identity

As a user,
I want warm bronze accents with measured contrast,
So that the app looks like Bronze.

**Requirements:** A11Y-003
**ADRs:** ADR-012
**Dependencies:** Stories 1.7 and 6.2
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** UI primitives exist
**When** tokens from DESIGN.md are applied
**Then** normal text contrast ≥4.5:1 in light/dark in automated sampling
**And** concept PNG is inspiration not pixel spec
**And** no Copper trade dress

**Failure / recovery:**
Color-only status fails.

**Security / privacy / diagnostics / a11y / i18n / data:**
Increase Contrast / Differentiate Without Color variants.

**Automated verification:**
- `pnpm --filter @bronze/ui test -- contrast`

**Signed-build / manual evidence:**
token tests; not a public AA claim

**Non-goals:**
marketing site


### Story 8.2: Reflow two hundred and four hundred

As a user,
I want 200% text resize and 400%/320 CSS px reflow,
So that A11Y-003 is evidenced in WebView.

**Requirements:** A11Y-003
**ADRs:** ADR-013
**Dependencies:** Stories 6.2, 6.3, 7.1
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** core screens exist
**When** layout tests run at 320 CSS px and 200% text
**Then** no control requires two-axis scroll
**And** toolbar overflow labeled
**And** composer reachable

**Failure / recovery:**
Lost functionality fails.

**Security / privacy / diagnostics / a11y / i18n / data:**
en-XA long strings included.

**Automated verification:**
- `pnpm --filter desktop test -- reflow`

**Signed-build / manual evidence:**
layout tests; manual zoom human

**Non-goals:**
WCAG conformance claim


## Epic 9: Packaging and release evidence

Reproducible packaging, SBOM, checksums, and an explicit human-validation backlog. No notarization by the agent.

### Story 9.1: Reproducible debug packaging

As a developer,
I want a documented local package command with checksums,
So that builds are repeatable.

**Requirements:** SEC-005
**ADRs:** ADR-011, ADR-002 Proposed
**Dependencies:** Story 1.8
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** verify passes
**When** packaging script is added
**Then** it records arch as arm64 unless DG-01 says otherwise
**And** SBOM generation stub or cargo/pnpm list captured
**And** no get-task-allow in release config

**Failure / recovery:**
Do not notarize or use Apple Developer credentials.

**Security / privacy / diagnostics / a11y / i18n / data:**
No network updater.

**Automated verification:**
- `pnpm verify`
- `test -f tooling/package-debug.sh`

**Signed-build / manual evidence:**
local script output

**Non-goals:**
notarization, Developer ID, universal2


### Story 9.2: Help diagnostics and known limitations

As a user,
I want local help and redacted diagnostics preview,
So that support is possible offline.

**Requirements:** SUP-001, SUP-002, SEC-006
**ADRs:** ADR-015, ADR-017
**Dependencies:** Stories 3.8 and 7.1
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** settings exist
**When** help/about/diagnostics are implemented
**Then** support bundle is previewed before export
**And** no automatic upload
**And** known limitations include human gates

**Failure / recovery:**
Secret in bundle fails.

**Security / privacy / diagnostics / a11y / i18n / data:**
Accessible HTML help, not image-only.

**Automated verification:**
- `pnpm --filter desktop test -- support`
- `cargo test -p bronze-diagnostics bundle`

**Signed-build / manual evidence:**
unit/component

**Non-goals:**
public ACR


### Story 9.3: Human validation backlog

As a operator,
I want an explicit backlog of human-only evidence,
So that the app is never called fully accessible or notarized by agents.

**Requirements:** G-04, A11Y-005, SEC-005, DG-08
**ADRs:** ADR-013, ADR-011
**Dependencies:** Stories 3.9, 3.10, 5.5
**Blocking gates:** human-only evidence; DG as named

**Acceptance Criteria:**

**Given** implementation stories exist
**When** docs/evidence/HUMAN-GATES.md lists VoiceOver, Voice Control, Switch Control, FKA, Sticky/Slow Keys, TCC, AX matrix, IME/Arabic/Japanese, signing/notarization, independent audit
**Then** each row is pending unless a human filled it
**And** no story in this list is agent-done

**Failure / recovery:**
Fabrication is forbidden.

**Security / privacy / diagnostics / a11y / i18n / data:**
n/a

**Automated verification:**
- `test -f docs/evidence/HUMAN-GATES.md`

**Signed-build / manual evidence:**
blocked-human-validation: never fabricate VoiceOver, Voice Control, Switch Control, TCC, signed capture, notarization, or linguistic QA evidence.

**Non-goals:**
closing DG-08

**Human gate:** blocked-human-validation: never fabricate VoiceOver, Voice Control, Switch Control, TCC, signed capture, notarization, or linguistic QA evidence.


