# Pre-implementation reconciliation

Version: 1.0  
Date: 2026-08-27  
Branch: `bmad/bronze-autonomous`  
Authority order: user prompt → `AGENTS.md` → [PRD](03-prd.md) → accepted ADRs in [18-adrs.md](18-adrs.md) → subsystem specs → research as evidence only.

This record inspects contradictions across the planning pack, names the winning authority, applies the smallest coherent correction, and preserves every requirement ID. Proposed ADRs stay unresolved.

## 1. Verdict

Reconciliation **passed** after the corrections in section 4. Checks in section 6 passed on this commit’s working tree.

Proposed ADRs remain gates, not silent product defaults:

| ADR | Status | Implementation rule |
| --- | --- | --- |
| ADR-002 | Accepted (2026-09-25) | Operator asked for a separate Intel build. macOS 14.0+; named arm64 and x86_64 packages, not universal2. |
| ADR-009 | Proposed | Storage locator is an interface. Development may use Application Support. |
| ADR-018 | Proposed | QUE-007/G-06 incomplete until SEARCH-01 golden corpus exists. |

## 2. Requirement ID inventory

PRD IDs are unchanged:

`G-01`–`G-07`, `CAP-001`–`CAP-010`, `QUE-001`–`QUE-008`, `WIN-001`–`WIN-005`, `DAT-001`–`DAT-004`, `SEC-001`–`SEC-006`, `A11Y-001`–`A11Y-006`, `I18N-001`–`I18N-004`, `SET-001`–`SET-004`, `SUP-001`, `SUP-002`.

Each ID remains in [PRD](03-prd.md) and [traceability](15-backlog-traceability.md).

## 3. Issue register

| ID | Topic | Tension | Authority | Resolution | Affected IDs |
| --- | --- | --- | --- | --- | --- |
| R-01 | Preview/Save vs persist | Copper video looks like toast-then-card; a pre-commit editor would need pending state. | UX spec §2 Quick editor; capture SM in [07](07-macos-capture-reliability.md) §7 and [04](04-functional-spec.md) F-CAP-01 | P0 commits a supported result before success feedback. Quick editor is post-save revision. Pre-commit confirmation is not a P0 setting. | CAP-004, CAP-010, QUE-002, A11Y-006 |
| R-02 | Atomic trigger-time snapshot | Event-tap cannot AX-query; destination/policy must not retarget later. | [07](07-macos-capture-reliability.md) §4.5/§8; ADR-004/005/006 | Ingress snapshots target PID, interned bundle token, activation generation, destination UUID plus accept-capture generation, app-policy/settings revisions, route, and monotonic time. Focused element/window identity is provider-time. No retarget. | CAP-004, CAP-008, CAP-009, QUE-001 |
| R-03 | Diagnostic contract vs schema | Functional spec listed looser field names than `diagnostic_events`. | ADR-015; [06](06-system-architecture.md) §9.2; [08](08-data-model-and-portability.md) §3 | Persisted columns are the contract. Functional spec aligned to those names. Content types remain type-impossible. | CAP-010, SEC-006, SUP-002 |
| R-04 | Shortcut actions vs settings JSON | `SettingsV1` showed only `standardChord`; registry lives in §4. | ADR-016; shortcuts table; SET-002 | Canonical store is `shortcuts` keyed by `ShortcutActionId`. `standardChord` is the `capture.selection` view. Types added to [12](12-settings-and-shortcuts.md). | CAP-001, CAP-002, SET-001, SET-002, I18N-004 |
| R-05 | Post-copy authorities | Copy lifecycle could be read as a global setting. | [06](06-system-architecture.md) §9.1; [04](04-functional-spec.md) F-QUE-05; [12](12-settings-and-shortcuts.md) §9 | Output profile is sole `postCopyAction`/`advancePolicy` authority. `returnToPriorApp` is focus-only, never provenance, never synthetic paste. Built-in default is `copied`+`keep`. | QUE-004, QUE-005 |
| R-06 | Idempotency receipts | Command retry vs new execution. | [06](06-system-architecture.md) §8.3; [08](08-data-model-and-portability.md) §3 | `command_receipts` + `command_receipt_entities`. Duplicate ID inside window reconstructs prior result. Older ID returns `idempotency_expired` and never re-executes. Receipts are content-free. | DAT-001, QUE-002, QUE-006, SEC-006 |
| R-07 | Export may contain secrets | Settings export forbids secrets; item export is user content. | DAT-003; SET-001; [08](08-data-model-and-portability.md) §8; [09](09-security-privacy-threat-model.md) | Settings file (`bronze-settings`) excludes credentials, tokens, diagnostics, and machine paths. The support file is a separate sectioned `bronze-support.txt`: redacted data path only (`/Users/[redacted]`), no credentials or item text. Settings preview names leftover user-entered literals/policies that may themselves be sensitive. Queue `bronze-export` archive necessarily contains item bodies/enabled provenance; preview warns before write. The two formats are not interchangeable. | DAT-003, SET-001, SEC-003, SEC-006 |
| R-08 | RTL vs physical panel edge | Logical CSS vs stored screen edge. | WIN-001; [06](06-system-architecture.md) §11; [11](11-i18n-localization.md) §6 | Persist physical `left\|right\|top`. RTL never mirrors stored side. UI chrome uses logical CSS. | WIN-001, WIN-002, I18N-003 |
| R-09 | Library vs panel capabilities | Overlapping queue UI. | ADR-010; [06](06-system-architecture.md) §10; [05](05-ux-ui-interaction-spec.md) §2 | Quick: active-section lifecycle. Library: paginated history, archive/trash, import/export, backup/restore. Settings: settings/shortcuts/permissions/diagnostics preview. Onboarding: manual note + permission preflight; no library/import/diagnostics authority. | WIN-005, QUE-001, DAT-002, DAT-003, SEC-002 |
| R-10 | Mandatory backup vs manual-only | DAT-002 vs a possible off schedule. | DAT-002; [12](12-settings-and-shortcuts.md) §11 | `backupSchedule` is `daily\|weekly` only. Back Up Now remains. Pre-migration/pre-restore backups are mandatory. No manual-only mode. | DAT-002, SET-001 |
| R-11 | Permission-state enum | Arrow list looked like a required linear machine. | SET-003; [07](07-macos-capture-reliability.md) §3 | Closed enum: `unknown\|not_requested\|denied\|granted_unverified\|healthy\|degraded\|unavailable\|requires_relaunch`. Granted ≠ healthy. | SET-003, SET-004, CAP-010 |
| R-12 | OS/architecture matrix | Hypothesis macOS 15+ vs ADR-002. | ADR-002; DG-01 | Accepted 2026-09-25: macOS 14.0+; split `bronze-macos-arm64.pkg` and `bronze-macos-x86_64.pkg`. Not a silent universal2 blob. | G-01, SEC-005, WIN-002 |
| R-13 | Locale fallback | Jumping `zh-Hant-HK` → `zh` would drop script. | ADR-014; I18N-002; [11](11-i18n-localization.md) §2 | RFC 4647 progressive lookup preserving script/variant before language, then `en`. | I18N-002, G-06 |
| R-14 | Per-item content language | UI locale leaking onto captured text. | I18N-003; DAT-003; ADR-014 | `content_language` canonical BCP 47 or `und`, default `und`, no silent detection, preserved across revision/export/import. | I18N-003, DAT-003, QUE-002 |
| R-15 | 200% plus 400%/320 reflow | Zoom vs text resize conflation. | A11Y-003; [05](05-ux-ui-interaction-spec.md) §10; [10](10-accessibility-conformance-plan.md) §6 | Both required: 200% text resize and 400% zoom/reflow at 320 CSS px without lost content or functionality. | A11Y-003 |
| R-16 | WCAG A/AA inventory | Priority mapping is not a criterion row. | ADR-013; [10](10-accessibility-conformance-plan.md) §2.1; DG-08 | Inventory is a release evidence artifact, not a planning contradiction. Public AA/conformance claims stay blocked until every A/AA row plus independent audit exist. Human AT evidence uses `blocked-human-validation`. | G-04, A11Y-001–A11Y-006 |
| R-17 | Differentiate Without Color | Web `prefers-*` vs native display options. | A11Y-003; [06](06-system-architecture.md) §12; [12](12-settings-and-shortcuts.md) §2 | Native bridge reads Reduce Motion, Reduce Transparency, Increase Contrast, and Differentiate Without Color, observes `accessibilityDisplayOptionsDidChangeNotification`, and publishes one snapshot. App overrides may only strengthen. | A11Y-003, WIN-004 |

## 4. Corrections applied

1. [README.md](../README.md) — index docs 20–21; retire the research-task “no Git” sentence without touching `main`.
2. [12-settings-and-shortcuts.md](12-settings-and-shortcuts.md) — add `ShortcutActionId` and `ShortcutBinding`; state shortcuts-table ownership; forbid off/manual-only backup schedule in schema comments.
3. [07-macos-capture-reliability.md](07-macos-capture-reliability.md) — permission values are a closed enum, not a linear arrow path.
4. [04-functional-spec.md](04-functional-spec.md) — diagnostic fields match `diagnostic_events`.

No requirement ID was added, removed, or renamed.

## 5. Closed enums (implementation contract)

| Enum | Values |
| --- | --- |
| Item lifecycle | `queued \| copied \| active \| done \| skipped \| trashed` |
| Section state | `active \| archived \| trashed` |
| Capture terminal | `saved \| rejected \| failed \| cancelled` |
| Permission | `unknown \| not_requested \| denied \| granted_unverified \| healthy \| degraded \| unavailable \| requires_relaunch` |
| Post-copy action | `unchanged \| copied \| active \| done` |
| Advance policy | `keep \| nextQueued` |
| Clipboard fallback | `manual \| syntheticExperimental \| off` |
| Panel mode | `summon \| pinned \| autoHide` |
| Panel edge | `left \| right \| top \| lastPosition` |
| Backup schedule | `daily \| weekly` |
| Content language | canonical BCP 47 or `und` |

## 6. Checks

Commands and evidence:

```text
npx --yes markdownlint-cli2 "docs/**/*.md" "README.md" "AGENTS.md" "research/**/*.md" "assets/README.md"
python3 tooling/planning-checks.py
```

Expected: zero markdownlint errors on the planning pack; planning-checks reports PASS for local links, requirement parity, enum/schema mentions, and traceability.

Human-only evidence (VoiceOver, Voice Control, Switch Control, signed TCC matrix, notarization, linguistic QA) is **not** claimed here.
