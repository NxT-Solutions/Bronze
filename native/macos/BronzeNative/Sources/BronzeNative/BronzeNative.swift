// BronzeNative.swift
// Swift implementation of the canonical C ABI types and narrow surface.
// No AppKit, ApplicationServices, CGEventTap, AX, or product behavior.
// All macOS capture stays behind this typed interface (AGENTS.md, ADR-004).
//
// CAP-004, SEC-002, ADR-004
// - versioned C ABI, fixed-width, ptr+len UTF-8 views
// - no NUL-terminated strings or strlen
// - version check fails closed on mismatch
// - Rust panic / Swift error never cross FFI boundary
// - sensitive buffers: no debugDescription containing user content
// - double completion forbidden (documented, tested)

// ABI version/status constants — MUST BE KEPT IN SYNC WITH BronzeNative.h defines (single source of truth is the .h for the C contract).
public let BRONZE_ABI_VERSION: UInt32 = 1
public let BRONZE_STATUS_OK: UInt32                 = 0
public let BRONZE_STATUS_INVALID_UTF8: UInt32       = 1
public let BRONZE_STATUS_DOUBLE_COMPLETION: UInt32  = 2

// Non-owning borrowed UTF-8 view. Exact layout to match C struct in BronzeNative.h for ABI.
// (ptr may be null iff len==0). Used for both Swift API and C ABI export.
public struct bronze_native_utf8_view {
    public let ptr: UnsafePointer<UInt8>?
    public let len: UInt64

    // Explicit public memberwise init so that test target (separate module) can construct
    // views when exercising the ABI surface. Without this, synthesized init can be internal.
    public init(ptr: UnsafePointer<UInt8>?, len: UInt64) {
        self.ptr = ptr
        self.len = len
    }
}

// Returns current ABI version. Consumers must compare and fail closed on != BRONZE_ABI_VERSION.
@_cdecl("bronze_native_abi_version")
public func bronze_native_abi_version() -> UInt32 {
    BRONZE_ABI_VERSION
}

// Strict length-delimited UTF-8 validation. Never relies on cString, strlen, or NUL.
// Uses lossy decode + exact byte roundtrip to detect replacement characters (�)
// without any throwing API that could be mistaken for unwind across boundary.
// Accepts embedded U+0000 (len preserved). Rejects invalid sequences, overlong, etc.
// @_silgen_name used (instead of @_cdecl) because Swift structs are not @objc-representable;
// produces correct C symbol name + C ABI lowering for struct-by-value (layout-compatible).
@_silgen_name("bronze_native_validate_utf8")
public func bronze_native_validate_utf8(_ view: bronze_native_utf8_view) -> UInt32 {
    if view.ptr == nil {
        return view.len == 0 ? BRONZE_STATUS_OK : BRONZE_STATUS_INVALID_UTF8
    }
    if view.len > UInt64(Int.max) {
        return BRONZE_STATUS_INVALID_UTF8
    }
    let count = Int(view.len)
    // Safe: non-nil ptr; count from trusted test input in this story.
    let buffer = UnsafeBufferPointer(start: view.ptr, count: count)
    // NOTE: Array copies are acceptable for this narrow ABI test surface + small test payloads.
    // Real paths (later stories) use explicit owner + free per arch §7.2 "allocation owner provides matching free".
    let bytes = Array(buffer)
    let decoded = String(decoding: bytes, as: UTF8.self)
    let reencoded = Array(decoded.utf8)
    if reencoded.count != count {
        return BRONZE_STATUS_INVALID_UTF8
    }
    return reencoded == bytes ? BRONZE_STATUS_OK : BRONZE_STATUS_INVALID_UTF8
}

// Test helper: exposes the len field for conformance matrix (HAPPY, EMBEDDED_NUL, LARGE, EMPTY).
// Exercises ptr+len contract without any string conversion or ownership transfer.
// @_silgen_name for same C-ABI struct passing reason as validate_utf8.
@_silgen_name("bronze_native_test_view_len")
public func bronze_native_test_view_len(_ view: bronze_native_utf8_view) -> UInt64 {
    view.len
}

// Explicit lifecycle stubs required by arch contract §7.2.
// No-op for this story. Must not unwind. Double call is forbidden (tested via status).
@_cdecl("bronze_native_init")
public func bronze_native_init() {
    // no-op; ownership and resources deferred
}

@_cdecl("bronze_native_shutdown")
public func bronze_native_shutdown() {
    // no-op; completion contracts enforced by callers
}

// No CustomDebugStringConvertible / description that could leak sensitive content from views.
// Views are plain public structs; any future print must be length-only.
