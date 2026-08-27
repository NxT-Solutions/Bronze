# Story 3.4: Atomic ingress snapshot

Status: ready-for-dev

## Story

As a developer,
I want CaptureIngressContext seqlock snapshot,
So that queued work cannot retarget.

**Requirements:** CAP-004, CAP-008, CAP-009, QUE-001
**ADRs:** ADR-005, ADR-006
**Dependencies:** Stories 2.2 and 3.3
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** native façade exists
**When** ingress context is implemented
**Then** snapshot includes target PID, bundle token, activation generation, destination UUID, accept-capture generation, policy/settings revisions, route, monotonic time
**And** inconsistent read returns context_unavailable
**And** focused element is not in event-tap snapshot

**Failure / recovery:**
Never substitute current frontmost app.

**Security / privacy / diagnostics / a11y / i18n / data:**
No titles/URLs in ingress.

**Automated verification:**
- `cargo test -p bronze-capture ingress`
- `swift test --package-path native/macos/BronzeNative --filter Ingress`

**Signed-build / manual evidence:**
unit tests

**Non-goals:**
real AX focus identity (next stories)


## Dev Agent Record

### Agent Model Used

### File List
