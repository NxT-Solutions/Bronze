# Research methodology

Cutoff: 2026-08-27.

## Evidence collected

- Inspected Copper landing, privacy, and terms pages in rendered browser.
- Acquired landing-page video through browser page assets; inspected metadata and frame sequence at two-second and half-second intervals.
- Audited Cooper repository source, releases, issues, CI, history, permissions, data model, UI semantics, and Apache-2.0 license through public GitHub surfaces.
- Compared direct and adjacent products using official product, pricing, documentation, App Store, and repository pages.
- Read official Tauri security and distribution guidance.
- Read Apple documentation and Apple DTS guidance for event taps, Input Monitoring, Accessibility, VoiceOver evaluation, and macOS security controls.
- Read WCAG 2.2 and rendered/visually inspected relevant pages of ETSI EN 301 549 V3.2.1, especially clauses 5, 11, and 12.
- Queried current package registries for a dated implementation baseline.
- Searched public agent-skill catalog and checked candidate repository health.

## Evidence grading

| Grade | Meaning | Examples |
| --- | --- | --- |
| A | Normative or primary technical source | W3C standard, ETSI standard, Apple docs, Tauri docs, repository source |
| B | First-party product claim | pricing, privacy, feature page, App Store listing |
| C | Third-party report | Reddit post, review, issue report |
| D | Inference | behavior inferred from code or combined sources; labeled as inference |

Prices and version numbers are snapshots, not enduring facts. Recheck before purchase, implementation bootstrap, or release.

## Constraints

- Reddit rendered a human-verification challenge in the in-app browser. Repository and public post surfaces supported only a marketing-source assessment; no CAPTCHA bypass was attempted.
- Video has no captions or text track. Sequence analysis therefore combines visual frames and on-screen text; no hidden narration was assumed.
- Accessibility conformance cannot be established from designs or source alone. This pack defines verification, not certification.
