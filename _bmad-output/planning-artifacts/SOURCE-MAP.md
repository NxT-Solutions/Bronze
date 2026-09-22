# SOURCE-MAP

Maps BMAD artifacts, epics, and stories to Bronze source files, requirement IDs, and ADRs.

| BMAD artifact | Bronze source | Requirements | ADRs |
| --- | --- | --- | --- |
| `prds/prd-bronze-app-2026-08-27/prd.md` | `docs/03-prd.md`, `docs/00-executive-summary.md`, `research/glossary.md` | all P0 IDs | ADR-001 |
| `prds/.../addendum.md` | `docs/06`, `docs/09`, `docs/18` | — | all |
| `ux-designs/ux-bronze-app-2026-08-27/DESIGN.md` | `docs/05`, `assets/README.md` | A11Y-003 | ADR-012 |
| `ux-designs/.../EXPERIENCE.md` | `docs/05`, `docs/10`, `docs/11`, `docs/12` | WIN/QUE/A11Y/I18N/SET | ADR-007, ADR-012–016 |
| `architecture/architecture-bronze-app-2026-08-27/ARCHITECTURE-SPINE.md` | `docs/06`, `docs/18`, `docs/07`, `docs/08`, `docs/09`, `docs/21` | CAP/QUE/WIN/DAT/SEC | ADR-001–017 accepted; 002/009/018 deferred; ADR-019 Proposed (compact_title first, optional hash-pinned SmolLM2 refine) |
| `epics.md` | `docs/14`, `docs/15`, `docs/21` | coverage map in epics.md | as per story |
| `AGENTS.md` `bmad:context` | `AGENTS.md`, `docs/20` | process | ADR-003 |
| `docs/21-preimplementation-reconciliation.md` | planning pack | all listed R-01–R-17 | accepted beat stale |

## Epic map

| Epic | Intent | Primary IDs |
| --- | --- | --- |
| 1 Toolchain | M0-01 / M1 | SEC-001–004 slots, I18N catalogs |
| 2 Swift bridge | M0-02 | CAP-004, ADR-004 |
| 3 Capture | M0-03–05 / M3 | CAP-001–010 |
| 4 Store | M2 | DAT-001–004, QUE domain |
| 5 Shell | M0-06 / M3 menu | WIN-001–005 |
| 6 Queue UI | M4–M5 | QUE-001–008 |
| 7 Settings | M6 | SET/I18N/DAT UI |
| 8 Visual | M5 polish | A11Y-003 |
| 9 Packaging | M7–M8 minus notarization | SEC-005–006, SUP, human gates |

## Story files

Machine-actionable stories live in `_bmad-output/implementation-artifacts/<epic>-<n>-<slug>.md`. Human-only stories 3.9, 3.10, 5.5, 9.3 remain epic-only until an operator records evidence under `docs/evidence/`.
