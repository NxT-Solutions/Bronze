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
uint32_t bronze_native_event_tap_drain(uint32_t *kind, uint64_t *sequence);
uint32_t bronze_native_event_tap_fsm_state(void);

// Test hooks: attach current thread as the sole SPSC producer (no live tap).
uint32_t bronze_native_event_tap_test_attach(void);
uint32_t bronze_native_event_tap_test_feed(uint32_t kind, int32_t carbon_key, uint64_t time_ns);
uint32_t bronze_native_event_tap_test_disable(void);
uint32_t bronze_native_event_tap_test_enqueue_from_caller(uint32_t kind);

#ifdef __cplusplus
}
#endif

#endif // BRONZE_NATIVE_H
