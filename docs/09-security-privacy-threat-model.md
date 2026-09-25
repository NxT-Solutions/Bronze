# Security, privacy, and threat model

Status: design-time baseline. Revisit before each new capability and release.

## 1. Security promises

- Captures are deliberate, local, and scoped to selected text.
- No keylogging, screen recording, passive clipboard history, telemetry, account, hosted AI, analytics, or runtime network in P0.
- Protected fields and excluded apps are not captured.
- WebView compromise cannot invoke broad native powers.
- Release identity is stable, signed, notarized, and verifiable.
- User can inspect, export, back up, and delete local data.

Promise wording must match packet capture and code. “No network” means no automatic update check either. Manual release link can open browser; later updater needs opt-in ADR and network ledger.

## 2. Assets

- Selected/user-authored text and provenance.
- Clipboard contents and pasteboard formats.
- Application/window identity and user workflow metadata.
- SQLite database, backups, exports, diagnostics.
- Accessibility/Input Monitoring grants tied to signed app identity.
- Signing certificates, notarization credentials, CI tokens, release artifacts.
- Native bridge, IPC permissions, WebView content and CSP.

## 3. Trust boundaries

```text
macOS apps + AX + event stream + pasteboard
                    │ untrusted/variable
                    ▼
Swift native adapter ── narrow versioned C ABI ── Rust core/store
                                                   │ typed commands
                                                   ▼
                                      Tauri WebView windows (untrusted UI)
                                                   │
                                      import/export filesystem boundary
```

WebView is untrusted presentation even though bundled locally. Captured/imported text is hostile data. Native OS APIs can return malformed, late, or inaccessible data. Files and archives can be malicious.

## 4. Threat actors and exclusions

Consider malicious imported file, compromised WebView/dependency, other local app racing clipboard, hostile/buggy AX target, local user/process with same account, supply-chain attacker, stolen export/backup, and accidental user action.

P0 does not claim defense against administrator/root, kernel compromise, physical access to unlocked Mac, or malicious code running with same user that can already read Application Support. Document these limits.

## 5. Threat register

| ID | Threat | Control | Verification |
| --- | --- | --- | --- |
| T-01 | Global monitor becomes keylogger | mask minimal events; record only modifier FSM/non-Shift cancellation; no keycode/content logs; callback passive | code audit, event-log inspection, privacy review |
| T-02 | Protected field captured | classify role/subrole across bounded ancestor chain before content query; fail closed on unknown protection; app exclusions; no clipboard fallback in protected/unknown context | secure/custom/browser fixture, log/DB/clipboard assertion |
| T-03 | Synthetic copy mutates shared clipboard or races another writer | AX primary; manual fallback default; synthetic path off by default; stable `changeCount` read; no automatic restoration in P0 | concurrent-copy race suite |
| T-03A | Copied text persists in [macOS Clipboard History](https://support.apple.com/en-euro/guide/mac-help/mchl40d5b86b/26/mac/26), [Universal Clipboard](https://support.apple.com/en-us/102430), or third-party manager | pre-enable disclosure; exclusions/protected controls; synthetic fallback off by default; help for clearing OS history and disabling cross-device clipboard | signed-device privacy journey |
| T-04 | WebView invokes arbitrary file read/shell | no shell/fs/http plugins; per-window capabilities; scoped native tokens; canonical paths | capability tests, path fuzzing |
| T-05 | XSS in item/Markdown | constrained dialect only (`*` / `**` / `***`, line-start lists, plus escapes); inbox builds `strong`/`em`/`ul`/`ol`/`li`/text via `createElement`/`createTextNode`; never assign `item.body` as `innerHTML`; HTML tags stay text; copy HTML is the same escaped dialect; source icons accept only `data:image/png;base64,` from Rust; no raw HTML or remote resource; strict CSP | malicious corpus, CSP test |
| T-06 | IPC confused-deputy | per-window command allowlists; Zod/serde schemas; Rust ownership and authorization; revision checks | cross-window denial tests |
| T-07 | Malicious archive traversal/bomb | limits, canonicalization, no symlink, compression ratio, checksum/schema validation, staging | import fuzz/corpus |
| T-08 | DB corruption/migration loss | transactional migrations, online backup, checksums, integrity and recovery mode | fault injection |
| T-09 | Sensitive logs/support bundle | enum/duration logs only; content types impossible in diagnostic schema; preview export | schema/code audit, seeded-secret scan |
| T-10 | Remote exfiltration | zero network capabilities/dependencies at runtime; deny navigation; packet capture gate; item titles persist as portable `compact_title` first; optional refine loads only the offline SHA-256-pinned GGUF allow-list (`SmolLM2-135M-Instruct-Q4_K_M.gguf` SHA-256 `2e8040ceae7815abe0dcb3540b9995eaa1fa0d2ca9e797d0a635ae4433c68c2d`; `SmolLM2-360M-Instruct-Q4_K_M.gguf` SHA-256 `2fa3f013dcdd7b99f9b237717fa0b12d75bbb89984cc1274be1471a465bac9c2`; `qwen2.5-0.5b-instruct-q4_k_m.gguf` SHA-256 `74a4da8c9fdbcd15bd1f6d01d621410d31c6fc00986f5eb687824e7b93d7a9db`) or extractive `compact_title` (no GGUF); Settings `general.titleModel` never fetches; no OS-AI, no hosted AI, no runtime Hub fetch; official app icons come only from local NSWorkspace / installed `.app`; no hosted AI or Private Cloud Compute; opt-in hosted title refine is ADR-022 and stays off by default | offline network test |
| T-11 | Supply-chain compromise | lockfiles, pinned action SHAs, least CI permissions, audit/advisory gates, SBOM, provenance, checksums | release attestation review |
| T-12 | TCC identity drift or spoofed build | stable bundle ID/Team ID/designated requirement/signing; notarization; signed upgrade and recovery tests; no ad-hoc release | clean-VM, upgrade, revoke/regrant matrix |
| T-13 | Native ABI memory error | versioned C ABI, length-delimited values, ownership docs, sanitizers, no unwind across boundary | ABI tests, ASan/TSan where possible |
| T-14 | Oversized selection exhausts memory | size limits before IPC/storage/render; preview truncation without content truncation | boundary/load tests |
| T-15 | Source provenance leaks context | all identity fields off by default; explicit global/per-app opt-in and exclusions; output profile off by default | policy tests |
| T-16 | External link/phishing | no in-WebView navigation; show destination; allow HTTPS; open system browser after confirmation where needed | URL scheme tests |
| T-17 | Second instance corrupts DB | single-instance lock/activation routing; SQLite protection | launch race test |
| T-18 | Secret copied into queue | P0 deliberate capture, exclusions, clear undo/trash; optional local warning remains P1 | usability/security tests |
| T-19 | Captured hostile text becomes prompt injection when copied to AI/editor | preserve raw item; default prompt-block formatter labels/delimits untrusted context separately from instructions; escape delimiter; exact output preview; never auto-paste/execute; no safety guarantee | adversarial formatter corpus and user review journey |

## 6. Tauri controls

Follow [Tauri security model](https://v2.tauri.app/security/), [capabilities](https://v2.tauri.app/learn/security/capabilities-for-windows-and-platforms/), [permissions](https://v2.tauri.app/security/permissions/), [runtime authority](https://v2.tauri.app/security/runtime-authority/), and [CSP guidance](https://v2.tauri.app/security/csp/).

- Separate capabilities for quick panel, library, settings, and onboarding.
- Default deny. Each command permission names resource/action, not generic backend access.
- Disable remote IPC, navigation, window creation, devtools, protocol access, shell, process, global FS, HTTP, SQL frontend plugin.
- CSP starts `default-src 'self'; object-src 'none'; frame-src 'none'; base-uri 'none'; form-action 'none'`; add only exact Tauri/WebView requirements verified against current docs. Avoid `unsafe-eval`; minimize/avoid `unsafe-inline` through bundled CSS/nonces as supported.
- Bundle fonts/icons/assets. No remote images or update feed request in core build.
- IPC payload cap, pagination, command timeout, cancellation, and typed result envelope.
- Window label is not sole authorization; capability grants and backend state both check.

## 7. Native bridge controls

- In-process Swift static library; no helper identity ambiguity.
- C ABI version checked at startup; fixed-width types/tagged enums; explicit allocate/free.
- No Rust panic or Swift exception crosses ABI.
- Thread affinity documented and asserted: event-tap run loop, capture serial queue, AppKit main thread.
- Event callback p99 <1 ms, no allocations/logging/AX/IPC/DB/window operations.
- Pending selected text uses one-shot token and is cleared after save/discard/timeout.
- AX calls bounded; late response discarded against request ID/PID.
- Synthetic keyboard path uses cleanup guard so every key-down has key-up on all errors.

## 8. Filesystem and storage

- DB directory 0700 and files 0600 where supported; use protected container/Application Support per ADR.
- Rust derives owned paths from internal IDs; WebView never supplies raw asset path.
- Native file picker returns operation-scoped token tied to user selection.
- Settings export/import pickers are rust-owned (`NSSavePanel` / `NSOpenPanel` on the AppKit thread, never from the event-tap callback). Commands reject any WebView `requestedPath` (SEC-003). Import caps the file at 1 MiB, requires `bronze-settings` JSON, and strips/rejects credentials, permission tokens, diagnostics, and machine paths (SET-001). The preview DTO sent to Settings lists category/field keys and sensitive leftover keys only — not the raw payload or a filesystem path.
- Exports use atomic new output and do not overwrite without explicit selection.
- Backups inherit same protections; exported files are user-controlled and warning explains they may be unencrypted.
- P0 does not add custom encryption key management. FileVault is recommended for at-rest device protection. App-level encryption requires separate key-recovery/threat-model ADR.

## 9. Privacy data inventory

| Data | Default | Retention | Network |
| --- | --- | --- | --- |
| item content and content-language metadata | local DB | until user/trash/purge policy | never |
| item revisions and undo payload | local DB | bounded 30/7-day defaults; purge cascade | never |
| FTS derived body/provenance | local DB | synchronized with item; rebuilt on purge | never |
| source bundle ID/name | off by default; opt-in | with item/purge cascade | never |
| window title/URL | off | with item if enabled | never |
| diagnostic enums/timing | local | rolling 7 days proposal | never |
| content-free command receipts/entity revisions | local DB | rolling 7 days proposal | never |
| clipboard text read | bounded memory during fallback; system clipboard remains external | immediate Bronze-buffer clear | never by Bronze; OS Universal Clipboard may sync |
| permission state | local/system-derived | current + redacted events | never |
| backups | local | rotation setting; ordinary purge survives until rotation | never |

Privacy page must disclose Lemon Squeezy or website analytics only if Bronze actually uses them; do not inherit Copper wording.

Deletion claims apply only to Bronze-controlled primary data/backups. Time Machine, manual exports, macOS Clipboard History, Universal Clipboard, destination apps, and third-party clipboard managers are separate systems. Factory erase explains these limits and offers official clearing guidance.

## 10. Permissions

- Request Accessibility and Input Monitoring on native start and first capture when not already granted (SET-003, SET-004). Health Retest is an explicit user action that re-requests those same APIs.
- Input Monitoring status separate from Accessibility.
- Denial does not loop or block the manual composer (CAP-003).
- Secure Keyboard Entry is respected, never bypassed, and user is not told to disable it.
- Bronze requests no Screen Recording, microphone, camera, contacts, location, Photos, or broad Automation in P0.
- Permission health tests actual end-to-end capability without collecting arbitrary keys/content.

Apple guidance: [Input Monitoring](https://support.apple.com/guide/mac-help/control-access-to-input-monitoring-on-mac-mchl4cedafb6/mac), [Apple DTS event-tap clarification](https://developer.apple.com/forums/thread/707680), [Secure Keyboard Entry](https://support.apple.com/en-il/guide/terminal/trml109/mac).

## 11. Release and supply chain

- Direct Developer ID distribution; Mac App Store/App Sandbox out of v1.
- Stable reverse-DNS bundle ID, Team ID, Developer ID Application cert, hardened runtime, minimal entitlements.
- No `get-task-allow`, JIT, unsigned executable memory, or disabled library validation without reviewed necessity.
- Sign nested content inside-out, then final app/container. Submit shipped outermost artifact with `notarytool`, staple supported shipped container, and never mutate it afterward; validate Gatekeeper/stapling on clean VM.
- Build universal2 before claiming Intel support; otherwise state Apple Silicon-only support clearly.
- Pin CI actions to commit SHA, minimal read permissions by default, protected release environment, short-lived notarization credentials.
- Produce SHA-256 checksums, CycloneDX/SPDX SBOM, dependency licenses, and provenance/attestation. Verify artifact matches release source as practical.

Primary guidance: [Apple notarization](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution), [Hardened Runtime](https://developer.apple.com/documentation/security/hardened-runtime), [Tauri macOS signing](https://v2.tauri.app/distribute/sign/macos/).

## 12. Security gates

Before beta:

- threat model reviewed against implementation;
- CSP and capabilities asserted in tests;
- IPC/import fuzzing corpus passes;
- seeded secrets absent from logs/support bundle;
- synthetic clipboard race tests pass or feature stays off;
- `cargo audit`, RustSec, package audit, license review, and Tauri patch review clean or risk-accepted;
- network capture shows zero outbound traffic in core flows;
- independent security review of native bridge and release configuration scheduled before 1.0.
