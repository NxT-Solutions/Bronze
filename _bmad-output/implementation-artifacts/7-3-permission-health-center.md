# Story 7.3: Permission health center

Status: done

## Story

As a user,
I want distinct permission rows with why, retest, and alternatives,
So that denial is recoverable.

**Requirements:** SET-003, SET-004, CAP-003
**ADRs:** ADR-005
**Dependencies:** Stories 3.1 and 7.1
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** permission enum exists
**When** health UI is implemented
**Then** each capability is independent
**And** Screen Recording shown as Not used
**And** denial leaves manual composer

**Failure / recovery:**
Launch-loop prompting forbidden.

**Security / privacy / diagnostics / a11y / i18n / data:**
No content in self-test.

**Automated verification:**
- `pnpm --filter desktop test -- permissions`

**Signed-build / manual evidence:**
component tests; real TCC human

**Non-goals:**
completing Story 3.9


## Dev Agent Record

### Agent Model Used

Hedgehog authored loop / layer-eng (cursor-grok).

### File List

- `bronze-settings/src/health.rs` — independent rows, Screen Recording Not used, composer after denial; Input Monitoring / Accessibility System Settings URLs only
- `packages/ui/src/components/permission-health.tsx` — why/retest/alternatives; Open System Settings after a denied retest
- `packages/ui/src/lib/permission-health.ts` — `retest_used_permissions` then reveal Settings; never Screen Recording
- `apps/desktop/src/settings.html` — permission health list
- `apps/desktop/src/permission-health.mjs` — injectable invoke; no auto-open Settings

### Notes

- Story 3.9 stays backlog. Proposed ADR-002 / ADR-009 / ADR-018 remain unresolved.
- Retest calls `retest_used_permissions` (always request used permissions). If the OS will not re-prompt, `settings.permission.openSystemSettings` is revealed after that attempt. Screen Recording stays Not used and is never requested.
- AX / Input Monitoring denial still leaves the manual composer. Status-menu Capture uses the same used-permission prompt path.
