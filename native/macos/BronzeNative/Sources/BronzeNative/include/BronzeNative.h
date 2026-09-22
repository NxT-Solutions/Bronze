// BronzeNative.h
// Canonical C ABI header for Bronze macOS native bridge.
// Introduced by 2-1-versioned-c-abi-types (replaces placeholder from _bmad-output/implementation-artifacts/1-4-scaffold-swift-package.md).
// Sole source of truth for Rust/Swift FFI surface in this story.
//
// CAP-004, SEC-002, ADR-004
// Rules (from docs/06-system-architecture.md:230, docs/18-adrs.md:195):
// - fixed-width integers, C-compatible tagged enums (uint32 discriminant)
// - UTF-8 buffers: pointer + length ONLY; NEVER NUL-terminated or strlen
// - version check fails closed on mismatch
// - no Rust panic / Swift error/unwind crosses FFI
// - double completion forbidden
// - sensitive buffers have no debug description with user content
// - allocation owner provides free (deferred to later stories)

#ifndef BRONZE_NATIVE_H
#define BRONZE_NATIVE_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// ABI version. Query at load; mismatch must take closed path, no further use.
#define BRONZE_ABI_VERSION 1u

// Status codes are explicit uint32 #defines (not enum) for fixed-width guarantee
// across compilers/toolchains. Used as tagged return / out-status.
#define BRONZE_STATUS_OK                 0u
#define BRONZE_STATUS_INVALID_UTF8       1u
#define BRONZE_STATUS_DOUBLE_COMPLETION  2u
#define BRONZE_STATUS_CANCELLED          3u
#define BRONZE_STATUS_NOT_FOUND          4u
#define BRONZE_STATUS_SHUTTING_DOWN      5u
#define BRONZE_STATUS_DEGRADED           6u
#define BRONZE_STATUS_CONTEXT_UNAVAILABLE 7u
// Additional codes may be added in future stories; consumers must treat unknown as failure.

#define BRONZE_EVENT_TAP_IDLE            0u
#define BRONZE_EVENT_TAP_LISTENING       1u
#define BRONZE_EVENT_TAP_DEGRADED        2u

#define BRONZE_TAP_REC_NONE              0u
#define BRONZE_TAP_REC_TRIGGER           1u
#define BRONZE_TAP_REC_RESET             2u
#define BRONZE_TAP_REC_DISABLED          3u

#define BRONZE_TAP_FEED_DOWN             1u
#define BRONZE_TAP_FEED_UP               2u
#define BRONZE_TAP_FEED_CANCEL           3u

// Non-owning UTF-8 view. ptr may be NULL only when len==0.
// Length-delimited: never assumes NUL; embedded 0x00 is valid UTF-8 (U+0000).
// Ownership: borrowed view; caller retains ownership of buffer.
typedef struct bronze_native_utf8_view {
    const uint8_t *ptr;  // length-delimited bytes; not NUL-terminated
    uint64_t       len;  // exact byte length
} bronze_native_utf8_view;

// Returns the ABI version implemented by this build.
uint32_t bronze_native_abi_version(void);

// Strict UTF-8 validation over the exact [ptr, ptr+len) range.
// - Uses length-delimited scan; never calls strlen, cString, or scans for NUL.
// - Rejects invalid sequences (including overlong, surrogate, truncated).
// - Accepts U+0000 embedded (len preserved exactly).
// - Returns BRONZE_STATUS_OK on success, BRONZE_STATUS_INVALID_UTF8 on failure.
// - Never throws / unwinds / panics across boundary.
uint32_t bronze_native_validate_utf8(bronze_native_utf8_view view);

// Test helper: returns the exact len field from the view.
// Exists only to exercise ptr+len round-trip in conformance tests without side effects.
uint64_t bronze_native_test_view_len(bronze_native_utf8_view view);

// Explicit init / shutdown. Init resets ownership tables. Shutdown cancels
// every open probe exactly once (CAP-004). Never unwind across boundary.
void bronze_native_init(void);
void bronze_native_shutdown(void);

// Native-owned copy of a borrowed view. On OK, *out is owned and must be
// released with bronze_native_utf8_free exactly once (arch §7.2).
uint32_t bronze_native_utf8_owned_copy(bronze_native_utf8_view src, bronze_native_utf8_view *out);

// Matching free for a view produced by bronze_native_utf8_owned_copy.
// Double-free returns BRONZE_STATUS_DOUBLE_COMPLETION without a second deallocate.
uint32_t bronze_native_utf8_free(bronze_native_utf8_view view);

// One-shot probe (story 2.4): copies the view, then complete XOR cancel exactly once.
// request_id is generated in Rust. Invalid UTF-8 does not create a probe.
uint32_t bronze_native_probe_begin(uint64_t request_id, bronze_native_utf8_view view);
uint32_t bronze_native_probe_complete(uint64_t request_id);
uint32_t bronze_native_probe_cancel(uint64_t request_id);

// Count of probes still open (owned, not yet complete/cancel). Leak detector.
uint64_t bronze_native_probe_outstanding(void);

// Session listenOnly event tap (story 3.3, CAP-002, ADR-005).
// start never prompts TCC. Denial returns DEGRADED and does not suppress events.
uint32_t bronze_native_event_tap_start(void);
uint32_t bronze_native_event_tap_stop(void);
uint32_t bronze_native_event_tap_health(void);
uint32_t bronze_native_event_tap_set_enabled(uint32_t enabled);
uint32_t bronze_native_event_tap_set_tap_count(uint32_t count);
uint32_t bronze_native_event_tap_drain(uint32_t *kind, uint64_t *sequence);
uint32_t bronze_native_event_tap_fsm_state(void);

// Test hooks: attach current thread as the sole SPSC producer (no live tap).
uint32_t bronze_native_event_tap_test_attach(void);
uint32_t bronze_native_event_tap_test_feed(uint32_t kind, int32_t carbon_key, uint64_t time_ns);
uint32_t bronze_native_event_tap_test_disable(void);
uint32_t bronze_native_event_tap_test_enqueue_from_caller(uint32_t kind);

// Frontmost process, NSWorkspace (not AX). 0 when unavailable.
int32_t bronze_native_frontmost_pid(void);

// On-device item title. On OK, *out is owned and must be freed with
// bronze_native_utf8_free. DEGRADED when the on-device model is unavailable.
// INVALID_UTF8 on a bad body. Never unwinds.
uint32_t bronze_native_item_title(bronze_native_utf8_view body, bronze_native_utf8_view *out);

uint32_t bronze_native_ingress_publish(
    int32_t target_pid,
    uint64_t bundle_token,
    uint64_t activation_generation,
    const uint8_t *destination_uuid,
    uint64_t accept_capture_generation,
    uint64_t policy_revision,
    uint64_t settings_revision,
    uint64_t context_generation,
    uint32_t route,
    uint64_t monotonic_time_ns);
uint32_t bronze_native_ingress_load(
    int32_t *target_pid,
    uint64_t *bundle_token,
    uint64_t *activation_generation,
    uint8_t *destination_uuid,
    uint64_t *accept_capture_generation,
    uint64_t *policy_revision,
    uint64_t *settings_revision,
    uint64_t *context_generation,
    uint32_t *route,
    uint64_t *monotonic_time_ns);
uint32_t bronze_native_ingress_test_begin_inconsistent(void);
uint32_t bronze_native_ingress_test_end_inconsistent(void);

// Writes string + HTML in one clearContents. Never logs body. DEGRADED if either type fails.
uint32_t bronze_native_pasteboard_write(bronze_native_utf8_view plain, bronze_native_utf8_view html);

// On OK, *out is owned UTF-8 (bronze_native_utf8_free). DEGRADED when the PID has no bundle.
uint32_t bronze_native_bundle_id_for_pid(int32_t pid, bronze_native_utf8_view *out);

// PNG octets in the view struct; not UTF-8. Free with bronze_native_utf8_free. Cap 16 KiB.
uint32_t bronze_native_app_icon_png(bronze_native_utf8_view bundle_or_name, bronze_native_utf8_view *out);

// Owned UTF-8 TSV: bundleId TAB displayName per line. Never includes filesystem paths.
uint32_t bronze_native_list_installed_apps(bronze_native_utf8_view *out);

// Rust-owned NSOpenPanel for one .app. Owned UTF-8 TSV: bundleId TAB displayName.
// CANCELLED when the operator dismisses. Never includes filesystem paths.
uint32_t bronze_native_pick_installed_app(bronze_native_utf8_view *out);

// Rust-owned NSSavePanel / NSOpenPanel for Settings JSON. Owned UTF-8 path.
// Path stays in Rust. CANCELLED when the operator dismisses.
uint32_t bronze_native_pick_settings_export_path(bronze_native_utf8_view *out);
uint32_t bronze_native_pick_settings_import_path(bronze_native_utf8_view *out);

// Local Notification Center banner. Title and body are catalog strings only.
// Never include captured selection or paths. DEGRADED when the payload is unsafe.
uint32_t bronze_native_deliver_user_notice(
    bronze_native_utf8_view title,
    bronze_native_utf8_view body);

#ifdef __cplusplus
}
#endif

#endif // BRONZE_NATIVE_H
