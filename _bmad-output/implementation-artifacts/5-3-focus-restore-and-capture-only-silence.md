# Story 5.3: Focus restore and capture-only silence

Status: ready-for-dev

## Story

As a user,
I want capture-only success to keep source focus,
So that I am not yanked into Bronze.

**Requirements:** WIN-003, CAP-004, A11Y-006
**ADRs:** ADR-007
**Dependencies:** Stories 3.5 and 5.2
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** panel and coordinator exist
**When** focus policy is implemented
**Then** capture-only does not steal source focus
**And** native announcement API is called for hidden WebView case (can be tested with fake)
**And** Escape restores prior focus where safe

**Failure / recovery:**
Hidden WKWebView live region is not accepted as the only feedback.

**Security / privacy / diagnostics / a11y / i18n / data:**
Announcement text localized.

**Automated verification:**
- `cargo test -p bronze-capture focus_policy`
- `pnpm --filter desktop test`

**Signed-build / manual evidence:**
unit/fake; VoiceOver human

**Non-goals:**
VoiceOver sign-off


## Dev Agent Record

### Agent Model Used

### File List
