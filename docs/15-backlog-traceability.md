# Backlog and traceability

This matrix converts PRD into executable backlog. Status starts `planned`. Implementation must add concrete test/evidence links; IDs never silently disappear.

## 1. Capture

| ID | Deliverable | Milestone | Primary verification | Status |
| --- | --- | --- | --- | --- |
| CAP-001 | configurable standard global chord capture | M3 | registration rollback; real-app selection matrix | planned |
| CAP-002 | optional configurable modifier double tap | M0/M3 | FSM property tests; 1k trials; false-trigger corpus | planned |
| CAP-003 | menu-bar and manual composer alternatives | M3/M5 | keyboard/VoiceOver route with permissions denied | planned |
| CAP-004 | queued request IDs and terminal outcomes | M3 | 20-request burst; no drop/duplicate; stage events | planned |
| CAP-005 | AX-first selection, bounded clipboard fallback | M0/M3/M4 | native fixtures + app matrix + race tests | planned |
| CAP-006 | protected-field hard block | M3 | seeded secure fixture; zero content in ABI/log/DB/clipboard | planned |
| CAP-007 | exact Unicode/whitespace | M3 | fidelity golden corpus | planned |
| CAP-008 | opt-in safe provenance | M3 | policy combinations and export/output tests | planned |
| CAP-009 | exclusions/per-app provenance | M3/M6 | exclusion before AX query (`excluded_bundle_rejects_before_ax_read`); searchable multi-select picker/settings tests; per-app provenance still planned | planned |
| CAP-010 | redacted staged diagnostics | M3 | schema cannot contain content; failure-path evidence | planned |

## 2. Queue

| ID | Deliverable | Milestone | Primary verification | Status |
| --- | --- | --- | --- | --- |
| QUE-001 | section lifecycle/order/active target | M2/M5 | domain properties; keyboard/UI tests | planned |
| QUE-002 | item CRUD/order/move/select | M2/M5 | revisions/order properties; UI role tests | planned |
| QUE-003 | typed lifecycle | M2/M5 | transition table; announcements/undo | planned |
| QUE-004 | configurable named output profiles and exact preview | M4/M5/M6 | CRUD/bounds/Unicode/Markdown/prompt golden tests | planned |
| QUE-005 | profile-owned lifecycle/advance/return; copied+keep default | M4/M5 | clipboard and lifecycle failure tests | planned |
| QUE-006 | undo/trash/purge safety | M2/M5 | nested dependency/fault tests; focus recovery | planned |
| QUE-007 | local search | M2/M5 | ADR-018 tokenizer golden corpus, locale semantics, 10k benchmark, result announcements | planned |
| QUE-008 | state/recovery UI | M5/M6 | empty/loading/capture-error/read-only/migration state tests | planned |

## 3. Windows and settings

| ID | Deliverable | Milestone | Primary verification | Status |
| --- | --- | --- | --- | --- |
| WIN-001 | summon/hide on correct display/edge | M0/M5 | pointer/front-app/multi-display matrix | planned |
| WIN-002 | safe frame, Spaces/full-screen/scale | M0/M5 | native display matrix | planned |
| WIN-003 | pinned/auto-hide/focus restore | M5/M6 | focus/Space/VoiceOver journeys | planned |
| WIN-004 | Dock/login/material/density/size controls | M6 | launch-at-login: Settings General checkbox, `login_item_status`, SMAppService apply tests (persist on requiresApproval; cargo tests never register). Dock icon, material, density, and size still planned | planned |
| WIN-005 | semantic resizable settings, no focus trap | M5/M6 | keyboard/VoiceOver/zoom | planned |
| SET-001 | searchable/resettable/safely exportable settings | M6 | schema/search/reset plus `bronze-settings` export/import/redaction tests (preview sensitive leftovers; rust-owned panels; no WebView path) | planned |
| SET-002 | accessible conflict-aware shortcut editor | M3/M6 | layouts/collision/registration rollback/AT | planned |
| SET-003 | distinct permission health and self-test | M3/M6 | deny/grant/revoke/stale identity matrix | planned |
| SET-004 | permission explanations/recovery/alternative | M6 | denial onboarding, keyboard/VoiceOver | planned |

## 4. Data and security

| ID | Deliverable | Milestone | Primary verification | Status |
| --- | --- | --- | --- | --- |
| DAT-001 | SQLite + transactional versioned migrations | M2 | all-version upgrade, crash/disk-full/corruption | planned |
| DAT-002 | automatic backup and verified restore | M2/M6 | online backup/restore/fault drill | planned |
| DAT-003 | deterministic JSON/Markdown import/export with content language | M2/M6 | round trip/golden/malicious corpus | planned |
| DAT-004 | scoped recoverable deletion | M2/M5 | tombstone/retention/purge dependency tests | planned |
| SEC-001 | no remote production content, strict CSP | M1/M7 | config assertion and exploit fixture | planned |
| SEC-002 | least-privilege capabilities/typed IPC | M1/M7 | per-window denial and fuzz tests | planned |
| SEC-003 | no arbitrary file/shell/path authority | M1/M7 | traversal/symlink/forbidden command tests | planned |
| SEC-004 | no account/telemetry/AI/network | M6/M7 | packet capture and dependency review | planned |
| SEC-005 | signed/notarized/SBOM/checksums/provenance | M8 | clean VM, signature/Gatekeeper/staple checks | planned |
| SEC-006 | content-free diagnostics/support bundle | M1/M7 | type audit, seeded secret scan, preview | planned |

## 5. Accessibility, i18n, support

| ID | Deliverable | Milestone | Primary verification | Status |
| --- | --- | --- | --- | --- |
| A11Y-001 | keyboard core, no timed gesture dependency | M5/M7 | keyboard + Sticky/Slow Keys journey | planned |
| A11Y-002 | role/name/state/focus/live semantics | M1/M5/M7 | Testing Library/axe/Accessibility Inspector/VO | planned |
| A11Y-003 | 200% text resize; 400%/320 px reflow; display preferences | M1/M6/M7 | rendered/manual matrix | planned |
| A11Y-004 | non-drag reorder | M5 | Move actions + keyboard/VO announcement | planned |
| A11Y-005 | VO/FKA/Voice/Switch Control | M7 | signed-build manual evidence | planned |
| A11Y-006 | identifiable recoverable errors | M5/M7 | failure journeys/live status | planned |
| I18N-001 | complete catalogs/ICU/no concatenation | M1/M6 | extraction/parity/raw-string CI; Settings General switcher applies shipped en/nl/fr/de/es/it catalogs; advertised remain en/en-XA/ar-XB; no public linguistic QA | planned |
| I18N-002 | canonical BCP 47/Intl/script-preserving fallback | M1/M6 | locale/format/fallback tests; `system`/unknown → en; persist tags closed in bronze-settings | planned |
| I18N-003 | RTL/IME/Unicode/long strings/per-item language | M1/M6/M7 | pseudo/Arabic/Japanese/grapheme/content-lang suite; item bodies `lang="und" dir="auto"` | planned |
| I18N-004 | localized semantic shortcuts | M6 | glyph/spoken/layout/recorder tests | planned |
| SUP-001 | accessible help/limitations/privacy/backup | M7 | clause-12/document audit | planned |
| SUP-002 | version/identity/schema/health/diagnostics | M6/M7 | redaction and accessibility tests | planned |

## 6. Goals and release metrics

| Goal | Evidence artifact | Owner | Release status |
| --- | --- | --- | --- |
| G-01 ≥99.9% supported capture, zero drops | signed capture matrix + burst/soak report | native/reliability | pending |
| G-02 latency budgets | benchmark distributions with hardware/OS | performance | pending |
| G-03 recovery | migration/backup/export fault report | storage | pending |
| G-04 accessible core | ACR matrix + audit + AT runs | accessibility | pending |
| G-05 local privacy | packet capture + exclusion/secure tests | security/privacy | pending |
| G-06 global readiness | catalog/RTL/IME/pseudo report | localization | pending |
| G-07 low interruption | moderated local usability study | product/design | pending |

## 7. Decision gates

| Gate | Decision | Needed evidence | Blocks |
| --- | --- | --- | --- |
| DG-01 | minimum macOS and architectures | API/WebKit/support/test capacity | bootstrap/release |
| DG-02 | validate accepted Swift static bridge; supersede only if spike fails | ABI/thread/sanitizer spike | native implementation |
| DG-03 | supported AX sources | real-app matrix and latency | promise/onboarding |
| DG-04 | synthetic clipboard fallback | full-format/race/safety tests | feature default/inclusion |
| DG-05 | utility window behavior | Space/display/focus/VoiceOver spike | panel implementation |
| DG-06 | validate accepted shadcn React Aria foundation; supersede only on demonstrated blocker | keyboard/AT/component ownership spike | UI foundation |
| DG-07 | container path/App Group | signing/container/backup evaluation | storage/release |
| DG-08 | public accessibility claim | reviewed row for every WCAG 2.2 A/AA criterion + native/EN mapping + independent audit | marketing/1.0 |
| DG-09 | updater/network | separate privacy/threat ADR and consent design | any auto-update code |
| DG-10 | locale-aware FTS/tokenizer semantics | query/result golden corpus across Latin, Arabic, CJK, Indic, code | QUE-007/G-06 |

## 8. Definition of done per requirement

Requirement status can change to `done` only when implementation exists, automated tests pass, required signed native/manual evidence exists, docs/help updated, security/a11y/i18n review complete, and trace row links exact evidence. If platform limitation remains, status is `partial` with supported matrix and user-visible fallback—not `done`.
