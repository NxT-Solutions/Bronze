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
// Additional codes may be added in future stories; consumers must treat unknown as failure.

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

// Explicit init / shutdown. No-op in v1 but surface required by arch contract.
// Never unwind across boundary. Documented as must-not-double-complete.
void bronze_native_init(void);
void bronze_native_shutdown(void);

#ifdef __cplusplus
}
#endif

#endif // BRONZE_NATIVE_H
