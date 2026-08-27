# Story 1.6: i18n catalogs and pseudo-locales

Status: ready-for-dev

## Story

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


## Dev Agent Record

### Agent Model Used

### File List
