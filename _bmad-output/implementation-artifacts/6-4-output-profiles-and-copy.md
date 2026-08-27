# Story 6.4: Output profiles and copy

Status: ready-for-dev

## Story

As a user,
I want to copy items with a named profile,
So that pasteboard output is deterministic.

**Requirements:** QUE-004, QUE-005
**ADRs:** ADR-008
**Dependencies:** Stories 4.3 and 6.2
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** items exist
**When** copy command runs
**Then** profile is sole postCopyAction/advancePolicy authority
**And** lifecycle changes only after pasteboard success
**And** default copied+keep
**And** no synthetic paste
**And** exact preview uses hostile sample in settings later; panel preview allowed under queue capability

**Failure / recovery:**
Marking done before pasteboard success is forbidden.

**Security / privacy / diagnostics / a11y / i18n / data:**
Prompt-block delimits untrusted context. No HTML fetch.

**Automated verification:**
- `cargo test -p bronze-domain formatters`
- `cargo test -p bronze-desktop copy`

**Signed-build / manual evidence:**
golden tests

**Non-goals:**
auto-paste


## Dev Agent Record

### Agent Model Used

### File List
