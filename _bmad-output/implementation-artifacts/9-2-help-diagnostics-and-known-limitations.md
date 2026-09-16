# Story 9.2: Help diagnostics and known limitations

Status: done

## Story

As a user,
I want local help and redacted diagnostics preview,
So that support is possible offline.

**Requirements:** SUP-001, SUP-002, SEC-006
**ADRs:** ADR-015, ADR-017
**Dependencies:** Stories 3.8 and 7.1
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** settings exist
**When** help/about/diagnostics are implemented
**Then** support bundle is previewed before export
**And** no automatic upload
**And** known limitations include human gates

**Failure / recovery:**
Secret in bundle fails.

**Security / privacy / diagnostics / a11y / i18n / data:**
Accessible HTML help, not image-only.

**Automated verification:**
- `pnpm --filter desktop test -- support`
- `cargo test -p bronze-diagnostics bundle`

**Signed-build / manual evidence:**
unit/component

**Non-goals:**
public ACR


## Dev Agent Record

### Agent Model Used

Hedgehog authored loop / layer-eng (cursor-grok).

### File List

- `bronze-diagnostics/src/bundle.rs` — preview-before-export, no upload, human gates
- `packages/ui/src/components/help.tsx` — local HTML help
- `apps/desktop/src/help.html` — diagnostics preview

### Notes

- Stories 3.9, 3.10, 5.5, 9.3 stay backlog. Proposed ADR-002 / ADR-009 / ADR-018 remain unresolved. No public ACR claim.
