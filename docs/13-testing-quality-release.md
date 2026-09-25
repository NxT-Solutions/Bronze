# Testing, quality, and release

## 1. Quality strategy

Capture reliability, native permissions, clipboard safety, data recovery, accessibility, and release identity dominate risk. Test pyramid alone is insufficient; combine pure deterministic state-machine tests, native fixtures, signed app/VM matrix, fault injection, long soak, assistive-technology journeys, and clean-VM release verification.

No test-free prototype merges into production path. Compilation/package success is not product validation.

## 2. Toolchain

Proposed baseline, locked at bootstrap:

- TypeScript/React: Biome, `tsc --noEmit`, Vitest, Testing Library, axe-core, Playwright or Tauri WebDriver/WebdriverIO.
- Rust: rustfmt, Clippy `-D warnings`, unit/integration/property tests, `cargo test` or nextest, cargo-audit/RustSec, cargo-deny.
- Swift: swift-format, XCTest/Swift Testing, Thread/Address Sanitizer test builds where compatible.
- SQLite: migration fixtures, property/fault tests, integrity and backup round trips.
- Packaging: codesign, `spctl`, `stapler validate`, notarization log, SBOM/checksum/provenance scripts.
- Network: packet/proxy capture on clean machine proving zero unexpected connections.

Tauri WebDriver supports WebView automation on macOS but does not replace signed native TCC/AX testing. [Tauri testing docs](https://v2.tauri.app/develop/tests/webdriver/).

## 3. CI task graph

```text
format-check
  ├─ lint-ts ─ typecheck ─ unit-ui ─ axe-component
  ├─ fmt-rust ─ clippy ─ unit-rust ─ migration-tests
  ├─ fmt-swift ─ unit-swift-fsm ─ native-abi-tests
  └─ i18n-validate ─ pseudo-locale-build
                         │
                 build-debug-signed
                         │
                  integration-mocked
                         │
                package-release-candidate
                         │
     signed-mac-matrix + a11y-manual + soak + security
                         │
               notarize/verify/release gate
```

Turbo cache: pure JS generation/test tasks only. Native signing, notarization, permission, E2E, timing, network, and release tasks `cache: false`. CI actions are pinned to commit SHA. Workflow permissions are contents read, plus actions write so a visual failure can upload diff images.

Pull-request CI is `.github/workflows/ci.yml`. It runs `quality-js` (Biome, typecheck, JS tests, i18n validate), `quality-docs` (`planning-checks.py`), `quality-rust` (fmt, Clippy, portable crate tests on Ubuntu), `quality-macos` (Clippy and tests except the Swift-linked `bronze-desktop` crate), and `quality-app` (`cargo test -p bronze-desktop` on macos-15). `.github/workflows/visual.yml` compares committed WebView screenshots for Queue, Settings, Library, and Help. `quality-js` waits on that job, so a mismatch fails the required `CI` check. Diff images upload as the `visual-diffs` artifact. The `CI` job fails if any of those fail. That workflow does not notarize, does not claim WCAG, and does not replace `pnpm verify` on a Mac.

## 4. Unit tests

### Trigger FSM

Property/table tests for timestamps and flag sequences:

- valid same/either/left/right double taps;
- boundary equals/minus/plus gap/hold/debounce;
- key repeat, dirty non-Shift, extra modifiers, overlap, missed release;
- monotonic clock wrap assumptions;
- cancel/reset on timeout, disable, wake, session, device, permission, setting;
- refractory behavior; no duplicate trigger;
- arbitrary generated event sequences never trigger without exact accepted pattern.
- Event-tap SPSC ingress model tests enforce single producer, publish payload before release-store watermark, consume after acquire-load, preserve source sequence/request-ID reconstruction through wrap assumptions, and emit one terminal overflow receipt for every dropped ID. Separate chord/menu routes cannot write SPSC and deterministic cross-route merge order is tested.

### Domain

- item lifecycle transition table and invalid transitions;
- copy profile formatting/golden Unicode;
- output-profile CRUD/default replacement, typed option bounds, template-like text remaining literal, and locale-change immutability;
- order insert/move/rebalance invariants;
- undo inverse commands and purge dependency safety;
- section active/archive/trash constraints;
- settings validation/migration and conflict normalization.
- canonical BCP 47 validation, RFC 4647-style script/variant fallback, and `und` item-language round trips.

### UI

- role/name/state queries and keyboard operations;
- composer IME behavior, draft preservation, errors;
- dialog/popover/menu focus; live status dedupe;
- search/selection/reorder/copy/complete/undo states;
- RTL, long strings, zoom classes, reduced motion/contrast.

## 5. Native integration fixtures

Build small signed fixture apps or test windows exposing:

- standard editable AX text with selection;
- read-only selected text;
- nested focus element with selection on parent;
- selected-range parameterized fallback;
- secure text field;
- delayed `kAXErrorCannotComplete` and timeout;
- target process exit/focus change;
- original target selection changes while queued, target changes away and back, request-age ceiling, policy revision, and destination accept-capture generation changes;
- custom controls with absent/malformed protection metadata; classifier must fail closed to `protection_unknown` without reading selection;
- pasteboard text/RTF/HTML/image/file/custom UTI/multiple items/lazy provider;
- concurrent pasteboard mutation;
- key synthesis failure after each down event.

Fixtures make failure deterministic. Real-app matrix then checks ecological validity.

## 6. macOS compatibility matrix

Minimum matrix is decided after OS-support spike; candidate macOS 15+ and current stable/beta policy. Cover:

- each supported macOS major/latest patch;
- Apple Silicon and Intel as separate named packages (`bronze-macos-arm64.pkg`, `bronze-macos-x86_64.pkg`); not a silent universal2 blob (ADR-002 Accepted);
- clean TCC, granted, denied, revoked mid-session, stale identity/update;
- TextEdit, Notes, Safari, Chrome, Firefox, VS Code/Cursor, Slack/Discord-like Electron, Terminal/iTerm, PDF Preview/browser, Office-like app where licensed;
- normal, read-only, canvas/custom, secure fields;
- QWERTY, AZERTY, QWERTZ, Dvorak/Colemak; built-in/external keyboard;
- Sticky Keys, Slow Keys, VoiceOver modifiers;
- single/multiple display, scale factors, notch, Dock/menu positions, Spaces/full-screen;
- sleep/wake, fast user switch/session lock, keyboard reconnect;
- Secure Keyboard Entry and remote desktop where feasible;
- 0–2 second copy latency and rapid repeated capture.

For each source record AX direct, range fallback, manual clipboard, synthetic experimental, exact fidelity, latency, focus result, and known limitation. Do not label unsupported case as flaky.

## 7. Reliability campaigns

- 1,000 valid double-tap trials per representative app/configuration: exactly one trigger each.
- 100,000 generated/recorded ordinary Shift sequences: zero false triggers.
- Rapid burst 20 capture requests: 20 terminal results, stable order, no drop.
- 24-hour idle/active event-tap soak: no disabled tap, leak, duplicate, CPU breach.
- 10,000-item store: startup/query/mutation/UI keyboard budgets.
- 1,000 clipboard race iterations: stable-generation reads or typed failure, no restoration write, no stuck modifiers. Do not claim atomic winner semantics from `changeCount`.
- Clipboard History/Universal Clipboard privacy journey and per-bundle fallback denial; wrong-content rate must be zero or synthetic fallback remains unsupported for that source.
- crash injection at each DB migration/export/backup phase.
- signed upgrade across RC versions: permissions remain valid or recovery is accurate.

Store raw test evidence locally in CI artifacts without user content.

## 8. Accessibility testing

Automated on every change:

- role/name-first component tests;
- axe on stable UI states;
- semantic lint and contrast tokens;
- keyboard journeys;
- en-XA/ar-XB and zoom screenshots/interaction.

Manual per release candidate:

- VoiceOver, Full Keyboard Access, Voice Control, Switch Control;
- focus/announcement/native status/window behavior;
- 200% text resize, 400%/320 CSS px reflow, dark/light, Increase Contrast, Differentiate Without Color, Reduce Motion, and Reduce Transparency;
- Sticky/Slow Keys and non-timing route;
- translated/RTL/IME journeys.
- live UI-locale change persists `general.locale`, reapplies WebView catalog and `html lang`/`dir`, keeps item bodies `lang="und" dir="auto"`, and leaves native app-menu labels until relaunch;
- item bodies tagged `ja`, `ar`, mixed-script tag, and `und` receive correct per-content language/direction exposure;
- authored hover/focus content passes dismissible/hoverable/persistent behavior; visible label remains in accessible name;
- completed evidence row exists for every WCAG 2.2 A/AA criterion, including reasoned N/A rows.

Capture-only success while source app keeps focus must use/test native accessibility announcement or persistent native status; hidden WebView live region is not accepted as evidence.

Evidence template and standards mapping live in [accessibility plan](10-accessibility-conformance-plan.md).

## 9. Security tests

- CSP production assertion and attempt inline/eval/remote frame/navigation.
- Enumerate commands allowed per window; invoke forbidden commands from compromised UI fixture.
- IPC fuzz lengths/enums/IDs/revisions and stale tokens.
- Path traversal, symlink, archive bomb, checksum, malicious JSON/Markdown/URL corpus.
- Native ABI fuzz/property tests; sanitizer runs.
- Seed canary secrets in selection/clipboard/title/URL; assert absent from logs, crash output, diagnostics/export unless expected item content.
- Offline packet capture through onboarding, capture, search, backup/import/export, About/help.
- dependency advisory/license audit and SBOM diff.
- notarized clean-VM install and Gatekeeper assessment.

## 10. Performance budgets

| Metric | Gate |
| --- | --- |
| event callback p99 | <1 ms |
| idle CPU | <0.5% on reference hardware |
| trigger acknowledgment p95 | <100 ms |
| trigger → AX result p95 | <200 ms target, <300 ms product gate |
| trigger → usable warm editor/panel p95 | <300 ms |
| copy write p95 | <50 ms |
| DB mutation p95 | <50 ms on reference hardware |
| 10k-item search p95 | <100 ms |
| steady RSS target | <200 MiB, investigate regression |
| capture queue drops | 0 |

Benchmarks pin hardware/OS/power state and report distributions, not single best run.

## 11. Defect severity

- **S0:** content/security/privacy loss, protected-field capture, release compromise, destructive migration. Blocks all release.
- **S1:** frequent missed/duplicate capture, inaccessible core journey, clipboard corruption, app crash, unrecoverable permissions. Blocks release.
- **S2:** recoverable feature failure, notable layout/translation/compatibility issue. Beta exception needs owner/date/workaround.
- **S3:** cosmetic/low-impact. May ship with documented plan.

Flaky test is defect. Quarantine requires issue, owner, reason, scope, expiry, and alternative evidence.

## 12. Release checklist

1. Requirement/traceability and ADR review complete.
2. All automated gates green; zero unowned quarantine.
3. S0/S1 zero; S2 exceptions approved/documented.
4. Capture and app matrices meet success thresholds.
5. Migration from all supported schemas and restore drill pass.
6. A11y matrix/known limitations reviewed; independent audit status clear.
7. i18n catalogs complete for advertised locales (`en`, `en-XA`, `ar-XB`); shipped nl/fr/de/es/it/ru/uk/hr/sl/da/sv/nb/fi/tr catalogs are not a public QA claim; RTL/IME passes.
8. Threat model, dependency audit, CSP/capabilities, network test pass.
9. Performance and 24-hour soak pass.
10. Split arm64 and x86_64 packages selected by ADR-002. Notarization is not claimed without `docs/evidence/`.
11. Clean VM install, first-run permissions, update, rollback/recovery pass.
12. Checksums, SBOM, provenance, license notices, privacy/help/support published.
13. Release notes state supported OS/apps and known capture limitations honestly.

## 13. Version/support policy

- Semantic versioning for app and export schema separately.
- Database migrations forward; rollback uses backup, not reverse SQL assumption.
- Security patch updates prioritized; minimum macOS/WebKit reviewed each release.
- Release rings: internal signed → small local beta → release candidate → stable.
- P0 has no auto-updater and no background network. A user-initiated GitHub latest-release check is ADR-023 Proposed (Settings button, no silent download, no Sparkle, no Tauri updater plugin). Homebrew installs are told to `brew upgrade --cask bronze`.

## 14. Local debug packaging (SEC-005)

Documented command: `tooling/package-debug.sh`.

- Records the cargo target architecture (`arm64` or `x86_64`). Release artifacts are the two named packages in ADR-002.
- Writes SHA-256 checksums and an SBOM stub (`cargo metadata` / `pnpm list`, not notarized CycloneDX).
- Release entitlements must not include `get-task-allow`.
- Does not notarize, use Apple Developer credentials, or open a network updater.
