# Story 5.2: Quick panel window and physical edge

Status: done

## Story

As a user,
I want to summon a key-capable panel on a physical left/right/top edge,
So that I can see the queue.

**Requirements:** WIN-001, WIN-002, WIN-005
**ADRs:** ADR-007
**Dependencies:** Stories 1.5 and 5.1
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** Tauri windows exist
**When** quick panel is implemented
**Then** stored edge is physical not RTL-leading
**And** work area clamping exists
**And** panel is activating
**And** no focus trap

**Failure / recovery:**
Nonactivating NSPanel is forbidden.

**Security / privacy / diagnostics / a11y / i18n / data:**
RTL does not mirror stored edge.

**Automated verification:**
- `cargo test -p bronze-desktop window_edge`
- `pnpm --filter desktop test`

**Signed-build / manual evidence:**
unit; multi-display matrix is human

**Non-goals:**
full queue CRUD


## Dev Agent Record

### Agent Model Used

Hedgehog authored loop / layer-eng (cursor-grok).

### File List

- `apps/desktop/src-tauri/src/window_edge.rs` — physical left/right/top; RTL does not mirror; work-area clamp; activating; no focus trap
- `apps/desktop/src-tauri/tauri.conf.json` — warm hidden `quick` window
- `apps/desktop/src/index.html` — panel chrome uses logical CSS; not a dialog trap
- `packages/i18n/locales/*/app.json` — `panel.quick.title`

### Notes

- Full queue CRUD is a non-goal. Multi-display matrix is human (5.5). Proposed ADR-002 / ADR-009 / ADR-018 remain unresolved.
