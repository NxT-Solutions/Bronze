# Story 7.5: Native and WebView catalog parity

Status: done

## Story

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


## Dev Agent Record

### Agent Model Used

Hedgehog authored loop / layer-eng (cursor-grok).

### File List

- `apps/desktop/src-tauri/src/catalog.rs` — native/InfoPlist glossary, advertised locales, RTL chrome
- `bronze-domain/src/entities.rs` — per-item `card_lang` / `card_dir="auto"`
- `packages/i18n/src/catalog.ts` — same keys for WebView
- `packages/ui/src/components/item-list.tsx` — per-item lang/dir on cards
- `packages/ui/src/components/panel-chrome.tsx` — RTL smoke, physical edge

### Notes

- Human linguistic QA remains backlog (stories 3.9, 3.10). Proposed ADR-002 / ADR-009 / ADR-018 remain unresolved.
