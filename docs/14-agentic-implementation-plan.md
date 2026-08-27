# Agentic implementation plan

## 1. Purpose

Plan supports iterative autonomous coding without sacrificing native-risk validation. Agents work small vertical slices, cite requirement IDs, write tests with behavior, and stop at explicit decision gates. UI generation must not race ahead of capture, TCC, storage, and accessibility spikes.

Read [AGENTS.md](../AGENTS.md) first. No Git operations unless user explicitly requests them.

## 2. Work protocol

For each task:

1. **Orient:** read linked PRD IDs, subsystem spec, ADRs, and existing tests.
2. **State contract:** write acceptance cases and failure modes in task note/test names.
3. **Inspect:** search existing symbols/files before creating new abstraction.
4. **Implement narrow slice:** domain → boundary → UI, keeping privileged work outside WebView.
5. **Verify:** smallest tests first, then affected workspace gates; run native/manual evidence when required.
6. **Threat/a11y/i18n review:** inspect data flow, keyboard/focus, strings, RTL, diagnostics.
7. **Document:** update requirement mapping/ADR/source only when behavior or decision changed.
8. **Handoff:** outcome, files, commands, evidence, unresolved risks; never call “done” when native evidence missing.

One task should usually fit one focused agent context and touch one subsystem boundary. Split changes crossing native monitor, capture provider, store, and UI into contract-first sequence.

## 3. Repository blueprint

```text
bronze-app/
  apps/desktop/
    src/                         # React shell/features
    src-tauri/                   # Tauri entry, config, capabilities, bundle
  packages/
    ui/                          # owned shadcn/React Aria components/tokens
    contracts/                   # generated/shared IPC DTOs
    i18n/                        # catalogs, formatters, pseudo-locales
    test-support/                # UI/import/export fixtures
  crates/
    bronze-domain/               # entities, commands, capture state/result
    bronze-capture/              # request state machine/provider policy
    bronze-storage/              # SQLite/migrations/backup/export/import
    bronze-settings/             # typed settings/shortcut policy
    bronze-diagnostics/          # content-free diagnostics/support bundle
    bronze-platform/             # platform-neutral traits
    bronze-platform-macos/       # safe façade over native ABI
  native/macos/BronzeNative/     # Swift Package/static library
  tooling/                       # generation/check/release scripts
  docs/                          # this pack and future evidence
```

Turbo manages JS graph; Cargo workspace Rust; SwiftPM native. Generated contracts must have one source of truth and drift check.

## 4. Milestone 0 — decisions and risk spikes

Goal: falsify risky assumptions before building UI.

### M0-01 Toolchain and minimum OS

- Pin Node/pnpm/Rust/Swift/Xcode/Tauri/shadcn/Biome/Turbo versions.
- Decide macOS minimum using WKWebView security support, API needs, user reach, and signed test capacity; architecture recommends macOS 15+ candidate.
- Decide Apple Silicon development and universal2 release policy.
- Record ADR-002.

Exit: hello app signed with stable local identity runs on minimum/current target; tool version document generated.

### M0-02 Native bridge spike

- Swift static library linked in Tauri/Rust.
- ABI version, string round trip, callback thread assertion, shutdown, error mapping.
- Panic/exception containment and sanitizer test.

Exit: 10k round trips/leak test, clear ownership docs, accepted ADR-004 validated or explicitly superseded with evidence.

### M0-03 TCC/event-tap spike

- Passive flagsChanged tap; Input Monitoring preflight/request after explanation.
- Pure double-Shift FSM with tests; wake/disable recovery.
- Stable signing upgrade test.

Exit: 1,000 trials and false-trigger corpus on at least two OS versions; permissions behavior documented.

### M0-04 AX selection spike

- focused element, secure-field block, selected text, parent/range fallback, bounded timeout.
- Test TextEdit, Safari, Chrome, Cursor/VS Code, Terminal, Preview/PDF, Notes, secure fixture.

Exit: compatibility matrix with exact provider and limitations. Decision gate sets P0 supported sources and default clipboard fallback.

### M0-05 Clipboard spike

- Manual “Create from Clipboard” path.
- Experimental synthetic Cmd-C with modifier-up wait, cleanup guard, stable `changeCount` read, no automatic restoration, and race tests.

Exit: synthetic path remains off by default; remove it if shared-clipboard/privacy behavior cannot meet threat model. Record any ADR-006 change.

### M0-06 Window/focus spike

- normal activating utility window; display safe frame; active Space/full-screen; prior focus restore; menu-bar fallback; VoiceOver focus.

Exit: single/multi-display + Space matrix; confirm no nonactivating NSPanel.

## 5. Milestone 1 — workspace and quality foundation

- **M1-01:** scaffold pnpm/Turbo, Cargo, SwiftPM workspaces; no product behavior.
- **M1-02:** configure Biome, TypeScript strict, rustfmt, Clippy, swift-format, tests.
- **M1-03:** establish typed contracts generation and drift CI.
- **M1-04:** define result/error codes and content-free diagnostic schema.
- **M1-05:** configure Tauri windows/capabilities/CSP default deny.
- **M1-06:** initialize shadcn with accepted React Aria base; add tokens, focus, motion/transparency media queries.
- **M1-07:** initialize i18n catalogs, en-XA/ar-XB, raw-string detector.
- **M1-08:** create requirement-aware CI tasks and local `verify` aggregate.

Exit: empty app/package passes format/lint/type/unit/a11y/i18n/security-config tests; WebView forbidden-command test succeeds.

## 6. Milestone 2 — domain and storage

- **M2-01 DAT-001:** entities, per-item BCP 47/`und` content language, lifecycle transition table, IDs, revisions, command API.
- **M2-02 QUE-001/002:** sections/items CRUD and stable order with property tests.
- **M2-03 DAT-001:** schema v1, migrations/checksums, WAL/foreign keys, single writer.
- **M2-04 QUE-003/006:** lifecycle, tombstone/trash, undo inverses; deep dependency purge safety.
- **M2-05 QUE-007:** decide ADR-018 through tokenizer/case/diacritic/CJK/Arabic/Indic golden corpus; implement FTS index/query and 10k benchmark.
- **M2-06 DAT-002:** online backups, integrity, read-only recovery.
- **M2-07 DAT-003:** deterministic export/import with hostile corpus.
- **M2-08 SEC-006:** diagnostic ring with type-level content exclusion.

Exit: fault-injected migration/backup/export/import suites pass. No WebView SQL/filesystem authority.

## 7. Milestone 3 — native activation and selection

- **M3-01 SET-003/004:** permission state model and native preflight interfaces.
- **M3-02 CAP-002:** production event monitor/FSM/reset; callback budget benchmark.
- **M3-03 CAP-001/SET-002:** registered standard shortcut transaction/conflict test.
- **M3-04 CAP-003:** status menu Capture/New Note/Show/Settings/Quit.
- **M3-05 CAP-004:** serial coordinator, request IDs, cancellation/deadline, terminal result.
- **M3-06 CAP-005/006/007:** AX provider chain, secure block, exact Unicode/whitespace.
- **M3-07 CAP-008/009:** source metadata policy/exclusions before content query.
- **M3-08 CAP-010:** stage diagnostics and health self-test.

Exit: burst queue has zero drop; secure fixture has zero content across ABI/log/store/UI; signed compatibility matrix meets beta threshold.

## 8. Milestone 4 — clipboard and copy

- **M4-01:** manual clipboard capture path, stale-content prevention.
- **M4-02:** synthetic experimental path only if M0 gate passed; race/multi-format/stuck-modifier tests.
- **M4-03 QUE-004:** output-profile CRUD, typed bounded format options, exact preview, pure formatters, and golden/property corpus.
- **M4-04 QUE-005:** pasteboard write, output-profile lifecycle/advance, copied+keep built-in default, copy-and-return, never auto-paste.
- **M4-05:** context-sensitive Cmd-C: editor selection vs focused item.

Exit: concurrent mutation yields stable read or typed failure; Bronze makes no restoration write. Failure changes no item lifecycle; copied text exact.

## 9. Milestone 5 — accessible UI vertical slice

- **M5-01 WIN-001/002:** panel shell/position/safe frame/display policy.
- **M5-02 QUE-001:** section chooser/list semantic structure.
- **M5-03 QUE-002/003:** item card, edit, completion/lifecycle, menus.
- **M5-04:** composer with IME/draft/error behavior.
- **M5-05 QUE-004/005:** selection toolbar/output profile/copy.
- **M5-06 QUE-006:** undo/trash feedback and focus recovery.
- **M5-07 QUE-007:** search/result/empty state.
- **M5-08 A11Y-002/006:** live status, dialogs, focus, error association.
- **M5-09 A11Y-004:** drag plus Move Up/Down/menu/keyboard alternative.

Each component task ships role/name keyboard tests, axe state tests, English/pseudo/RTL render, and reduced-motion/contrast check.

Exit: capture → edit → save → reorder → copy → complete → undo works keyboard and VoiceOver on signed build.

## 10. Milestone 6 — settings, privacy, i18n

- **M6-01 SET-001:** typed searchable settings window and reset scopes.
- **M6-02 SET-002:** accessible shortcut recorder, normalization/conflicts/rollback.
- **M6-03 SET-003/004:** permission health center, self-test, alternatives.
- **M6-04 WIN-003/004:** panel mode, display/edge/size, Dock/login settings.
- **M6-05 CAP-009/SEC-004:** exclusion/provenance/network statement/privacy UI.
- **M6-06 DAT-002/003:** backup/export/import/recovery UI.
- **M6-07 I18N-001–004:** native/WebView catalog completion, script-preserving fallback, per-item content language, RTL/IME/shortcut speech.
- **M6-08 A11Y-003:** 200% text resize, 400%/320 CSS px reflow, contrast/differentiate-without-color/motion/transparency settings and native preference bridge.

Exit: all settings migrate/export/reset; advertised locale coverage 100%; 400%/RTL/IME core UI usable.

## 11. Milestone 7 — hardening and evidence

- **M7-01 SEC-001–003:** production CSP/capability/IPC/path penetration tests.
- **M7-02 SEC-004/006:** zero-network capture and secret-log scan.
- **M7-03:** migration/DB/import fuzz and crash recovery.
- **M7-04:** native ABI sanitizers and event-tap 24-hour soak.
- **M7-05 G-01/G-02:** reliability and latency campaigns.
- **M7-06 A11Y-001–006:** full manual AT matrix, Accessibility Inspector, row for every WCAG 2.2 A/AA criterion, native/EN mapping, conformance draft.
- **M7-07 SUP-001/002:** accessible help, diagnostics, known limitations.
- **M7-08:** external security/accessibility review findings triaged.

Exit: release checklist evidence complete; no S0/S1.

## 12. Milestone 8 — distribution

- **M8-01 SEC-005:** stable bundle ID/Team ID, hardened runtime, minimal entitlements.
- **M8-02:** build architecture accepted by ADR-002 (universal2 only if Intel supported), nested signing, notarization/staple, clean-VM validation.
- **M8-03:** checksums, SBOM, provenance, license notices, privacy/support docs.
- **M8-04:** signed upgrade retains or accurately recovers TCC permissions.
- **M8-05:** small beta, compatibility reports, final performance/accessibility/network regression.

Exit: v1 release definition in PRD met. Manual distribution only; no hidden updater request.

## 13. Parallelization map

Safe parallel work after contracts:

- storage schema/migrations ↔ UI tokens/i18n foundation ↔ native FSM pure tests;
- output formatters ↔ panel layout ↔ backup format;
- security capability tests ↔ accessibility component tests ↔ native source fixtures;
- documentation/help ↔ release tooling ↔ performance harness.

Do not parallel-edit same contract or couple implementation before decision gate. Single owner for native ABI, data schema migration order, settings schema, requirement matrix, and release config.

## 14. Agent task template

```md
# Task: <small outcome>

Requirements: CAP-005, CAP-006
ADRs: ADR-004, ADR-006
Inputs/contracts:
Out of scope:

Acceptance cases:
1.
2.

Failure/security cases:
Accessibility/i18n cases:
Tests/evidence required:
Files expected:
Decision needed, if any:
```

Handoff:

```md
Outcome:
Requirement coverage:
Files changed:
Verification run/result:
Manual/native evidence:
Remaining risk/known limitation:
Docs/ADR updated:
```

## 15. Stop conditions

Agent must stop and raise decision, not guess, when:

- implementation needs new permission/network/cloud/helper process;
- secure-field or clipboard safety cannot meet invariant;
- bundle/signing identity change could reset TCC;
- schema migration could destroy/overwrite data;
- shortcut behavior conflicts with AT/system and no alternative exists;
- source license/trade-dress reuse is unclear;
- public conformance/support claim lacks evidence;
- P0 scope expands into passive clipboard, recording, AI, sync, or new platform.
