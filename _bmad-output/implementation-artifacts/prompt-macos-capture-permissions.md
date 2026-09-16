# Change-work: Prompt macOS capture permissions

Status: done

## Intent

Prompt for Accessibility and Input Monitoring on first native need and let permission-health retest re-request those OS prompts.

**Requirements:** SET-003, SET-004, CAP-003, CAP-010
**ADRs:** ADR-001 (no Screen Recording), ADR-005 (listen-only tap). Proposed ADR-002 / ADR-009 / ADR-018 remain unresolved.

**Acceptance:**

- Native start and first capture request Accessibility (`AXIsProcessTrustedWithOptions` with prompt) and Input Monitoring (`CGRequestListenEventAccess`) when not already granted. Never loop-prompt on launch.
- Health Retest always attempts those request APIs. If the OS will not re-prompt, System Settings is fallback after the attempt.
- Tests inject a fake request hook. Production source is not preflight-only. Denial leaves the manual composer.
- Screen Recording is never requested and stays Not used.

## When prompts fire

| Reason | When | Request if already granted? |
| --- | --- | --- |
| Native start | `start_native_or_die` after ABI check, before `event_tap_start` | no |
| First capture | `on_capture_requested` once per process | no |
| Health retest | Settings Retest → `retest_used_permissions` | always attempt |

Requests run off the event-tap callback. Creating a listen-only tap may also surface Input Monitoring; that is not a Screen Recording request.

## Dev Agent Record

### File List

- `bronze-platform-macos/src/permission.rs` — `PermissionRequestHost`, `prompt_used_permissions`
- `bronze-settings/src/permission.rs` — `from_request_granted`
- `bronze-settings/src/health.rs` — used-permission Settings URLs only
- `apps/desktop/src-tauri/src/capture_permissions.rs` — start / first-capture / retest / open Settings
- `apps/desktop/src-tauri/src/lib.rs` — `prompt_on_native_start`, `on_capture_requested`
- `packages/ui/src/lib/permission-health.ts` — retest invoke; never Screen Recording
- `apps/desktop/src/permission-health.mjs` — injectable invoke; reveal Settings after denied retest
- `docs/07-macos-capture-reliability.md` — permission service requests on start / first capture / retest
- `docs/06-system-architecture.md` — startup step 8 requests used permissions
- `docs/09-security-privacy-threat-model.md` — request on first native need; composer after denial
- `README.md` — how to see the OS dialogs (`pnpm --filter desktop tauri dev`, no Corepack)

### Notes

- Story 3.9 / 3.10 / 5.5 / 9.3 stay backlog.
- First-capture hook is Rust `on_capture_requested` (status/app-menu Capture). It persists AX selection when permitted; the WebView is not the capture trigger.
- Input Monitoring often prompts once per TCC identity. Accessibility may re-prompt on some OS versions. A `tauri dev` rebuild can receive a new TCC identity.
