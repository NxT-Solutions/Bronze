# Validation checklist

Optional. The operator decides ad hoc which rows to run, and whether a
person or an agent runs them. Nothing here is a mandatory human-only
gate.

Do not claim WCAG, VoiceOver, notarization, or platform conformance
without evidence in this directory. Do not fabricate evidence.

| Check | Stories | Evidence |
| --- | --- | --- |
| TCC grant/deny/revoke/relaunch | 3.9 | signed build log |
| AX source matrix | 3.10 | docs/07 §15 matrix |
| Spaces/display/full-screen | 5.5 | WIN-002 matrix |
| VoiceOver, Voice Control, Switch Control, FKA | 9.3 | A11Y-005 |
| Sticky/Slow Keys, layouts | 9.3 | A11Y-001 |
| Arabic/Japanese/Indic IME | 9.3 | I18N-003 |
| Accessibility Inspector | 9.3 | A11Y-002 |
| Independent accessibility audit | 9.3 | DG-08 |
| Developer ID / notarization | 9.3 | SEC-005 |
| WCAG 2.2 A/AA row inventory | 9.3 | docs/10 §2.1; draft `WCAG-22-INVENTORY.md` |

Related: Stories 3.9, 3.10, 5.5, 9.3 in `_bmad-output/planning-artifacts/epics.md`.
