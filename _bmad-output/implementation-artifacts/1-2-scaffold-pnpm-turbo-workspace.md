# Story 1.2: Scaffold pnpm Turbo workspace

Status: ready-for-dev

## Story

As a developer,
I want pnpm-workspace and Turbo with empty packages,
So that JS work has a graph before product code.

**Requirements:** SEC-001, I18N-001 (package slots)
**ADRs:** ADR-003, ADR-017
**Dependencies:** Story 1.1
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** pinned package manager
**When** workspace is scaffolded
**Then** `pnpm-workspace.yaml`, `turbo.json`, `packages/ui`, `packages/contracts`, `packages/i18n`, `packages/test-support`, `apps/desktop` exist
**And** no product features
**And** Turbo cache disabled is not required yet because no signing tasks exist

**Failure / recovery:**
If pnpm install needs network, use lockfile creation once then keep lockfile; do not add remote fonts or CDN.

**Security / privacy / diagnostics / a11y / i18n / data:**
packages/ui must not own product state. No raw user strings in TSX.

**Automated verification:**
- `python3 tooling/planning-checks.py`
- `npx --yes markdownlint-cli2 'docs/**/*.md' README.md AGENTS.md`
- `pnpm install`
- `pnpm exec turbo run lint --force || true`

**Signed-build / manual evidence:**
local install log

**Non-goals:**
shadcn components, Tauri commands, capture


## Dev Agent Record

### Agent Model Used

### File List
