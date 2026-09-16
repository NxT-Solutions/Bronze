# Story 5.4: Native display preference bridge

Status: done

## Story

As a user,
I want Reduce Motion, Reduce Transparency, Increase Contrast, Differentiate Without Color applied live,
So that OS preferences are honored.

**Requirements:** A11Y-003, WIN-004
**ADRs:** ADR-012, ADR-013
**Dependencies:** Stories 1.7 and 2.3
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** native façade exists
**When** NSWorkspace display options are read and observed
**Then** one typed snapshot is published to WebViews
**And** app overrides only strengthen
**And** tokens update without restart

**Failure / recovery:**
Weakening system Reduce Motion is forbidden.

**Security / privacy / diagnostics / a11y / i18n / data:**
No color-only status.

**Automated verification:**
- `cargo test -p bronze-platform-macos display_prefs`
- `pnpm --filter @bronze/ui test`

**Signed-build / manual evidence:**
unit; manual OS toggle is human

**Non-goals:**
full AT matrix


## Dev Agent Record

### Agent Model Used

Hedgehog authored loop / layer-eng (cursor-grok).

### File List

- `bronze-platform-macos/src/display_prefs.rs` — typed snapshot, strengthen-only OR merge, FakeDisplayHost
- `packages/ui/src/lib/apply-display-snapshot.ts` — live data-* attributes on the WebView root
- `packages/ui/src/styles/globals.css` — snapshot tokens strengthen Reduce Motion / Transparency / Contrast / Differentiate Without Color without restart

### Notes

- Manual OS toggle remains human. Proposed ADR-002 / ADR-009 / ADR-018 remain unresolved.
