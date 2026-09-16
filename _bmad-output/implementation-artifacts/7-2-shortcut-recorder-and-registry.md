# Story 7.2: Shortcut recorder and registry

Status: done

## Story

As a user,
I want to bind every ShortcutActionId without losing the old chord on failure,
So that triggers are configurable.

**Requirements:** SET-002, CAP-001, CAP-002, I18N-004, A11Y-001
**ADRs:** ADR-016, ADR-005
**Dependencies:** Story 7.1
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** settings exist
**When** recorder is implemented
**Then** all registry actions persist in shortcuts table
**And** standardChord is capture.selection view only
**And** cannot disable chord+menu+manual together
**And** IME/VO chords not swallowed

**Failure / recovery:**
Failed registration retains old binding.

**Security / privacy / diagnostics / a11y / i18n / data:**
Spoken names localized. Test mode skippable.

**Automated verification:**
- `cargo test -p bronze-settings shortcuts`
- `pnpm --filter desktop test -- shortcuts`

**Signed-build / manual evidence:**
unit; layout matrix human

**Non-goals:**
claiming conflict-free globally


## Dev Agent Record

### Agent Model Used

Hedgehog authored loop / layer-eng (cursor-grok).

### File List

- `bronze-settings/src/shortcuts.rs` — registry, failed register retains old, IME/VO not swallowed
- `bronze-storage/src/shortcuts.rs` — persist every ShortcutActionId
- `packages/ui/src/components/shortcut-recorder.tsx` — recorder, skippable test
- `apps/desktop/src/settings.html` — shortcut registry list

### Notes

- Does not claim conflict-free globally. Proposed ADR-002 / ADR-009 / ADR-018 remain unresolved.
