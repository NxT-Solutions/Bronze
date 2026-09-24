// BronzeNative.swift
// Swift implementation of the canonical C ABI types and narrow surface.
// No AppKit, ApplicationServices, CGEventTap, AX, or product behavior.
// All macOS capture stays behind this typed interface (AGENTS.md, ADR-004).
//
// CAP-004, SEC-002, SEC-006, ADR-004
// - versioned C ABI, fixed-width, ptr+len UTF-8 views
// - no NUL-terminated strings or strlen
// - version check fails closed on mismatch
// - Rust panic / Swift error never cross FFI boundary
// - sensitive buffers: no debugDescription containing user content
// - one-shot complete or cancel; double completion forbidden

import Foundation

// ABI version/status constants — MUST BE KEPT IN SYNC WITH BronzeNative.h defines (single source of truth is the .h for the C contract).
public let BRONZE_ABI_VERSION: UInt32 = 1
public let BRONZE_STATUS_OK: UInt32                 = 0
public let BRONZE_STATUS_INVALID_UTF8: UInt32       = 1
public let BRONZE_STATUS_DOUBLE_COMPLETION: UInt32  = 2
public let BRONZE_STATUS_CANCELLED: UInt32          = 3
public let BRONZE_STATUS_NOT_FOUND: UInt32          = 4
public let BRONZE_STATUS_SHUTTING_DOWN: UInt32      = 5
public let BRONZE_STATUS_DEGRADED: UInt32           = 6
public let BRONZE_STATUS_CONTEXT_UNAVAILABLE: UInt32 = 7
public let BRONZE_PASTEBOARD_KIND_NONE: UInt32 = 0
public let BRONZE_PASTEBOARD_KIND_HTML: UInt32 = 1
public let BRONZE_PASTEBOARD_KIND_RTF: UInt32 = 2
public let BRONZE_PASTEBOARD_KIND_PLAIN: UInt32 = 3

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

private enum ProbePhase {
    case open(bronze_native_utf8_view)
    case completed
    case cancelled
}

private struct OwnedKey: Hashable {
    let ptr: UInt
    let len: UInt64
}

/// Process-global ownership tables. Lock is the only shared mutable state.
/// Status codes carry no payload bytes (SEC-006).
private final class NativeOwnership: @unchecked Sendable {
    static let shared = NativeOwnership()
    private let lock = NSLock()
    private var shuttingDown = false
    private var owned = Set<OwnedKey>()
    private var emptyOwned = 0
    private var probes: [UInt64: ProbePhase] = [:]

    func reset() {
        lock.lock()
        defer { lock.unlock() }
        releaseAllOwnedLocked()
        probes.removeAll()
        shuttingDown = false
    }

    func shutdownCancelOpen() {
        lock.lock()
        defer { lock.unlock() }
        shuttingDown = true
        for (id, phase) in probes {
            if case let .open(view) = phase {
                _ = freeLocked(view)
                probes[id] = .cancelled
            }
        }
    }

    func ownedCopy(_ src: bronze_native_utf8_view, _ out: UnsafeMutablePointer<bronze_native_utf8_view>) -> UInt32 {
        let status = bronze_native_validate_utf8(src)
        if status != BRONZE_STATUS_OK {
            return status
        }
        lock.lock()
        defer { lock.unlock() }
        if shuttingDown {
            return BRONZE_STATUS_SHUTTING_DOWN
        }
        return ownedCopyLocked(src, out)
    }

    func ownedRawCopy(_ src: bronze_native_utf8_view, _ out: UnsafeMutablePointer<bronze_native_utf8_view>) -> UInt32 {
        lock.lock()
        defer { lock.unlock() }
        if shuttingDown {
            return BRONZE_STATUS_SHUTTING_DOWN
        }
        return ownedCopyLocked(src, out)
    }

    func free(_ view: bronze_native_utf8_view) -> UInt32 {
        lock.lock()
        defer { lock.unlock() }
        return freeLocked(view)
    }

    func begin(id: UInt64, view: bronze_native_utf8_view) -> UInt32 {
        let status = bronze_native_validate_utf8(view)
        if status != BRONZE_STATUS_OK {
            return status
        }
        lock.lock()
        defer { lock.unlock() }
        if shuttingDown {
            return BRONZE_STATUS_SHUTTING_DOWN
        }
        if probes[id] != nil {
            return BRONZE_STATUS_DOUBLE_COMPLETION
        }
        var ownedView = bronze_native_utf8_view(ptr: nil, len: 0)
        let copyStatus = ownedCopyLocked(view, &ownedView)
        if copyStatus != BRONZE_STATUS_OK {
            return copyStatus
        }
        probes[id] = .open(ownedView)
        return BRONZE_STATUS_OK
    }

    func complete(id: UInt64) -> UInt32 {
        lock.lock()
        defer { lock.unlock() }
        switch probes[id] {
        case let .open(view):
            _ = freeLocked(view)
            probes[id] = .completed
            return BRONZE_STATUS_OK
        case .completed, .cancelled:
            return BRONZE_STATUS_DOUBLE_COMPLETION
        case nil:
            return BRONZE_STATUS_NOT_FOUND
        }
    }

    func cancel(id: UInt64) -> UInt32 {
        lock.lock()
        defer { lock.unlock() }
        switch probes[id] {
        case let .open(view):
            _ = freeLocked(view)
            probes[id] = .cancelled
            return BRONZE_STATUS_CANCELLED
        case .completed, .cancelled:
            return BRONZE_STATUS_DOUBLE_COMPLETION
        case nil:
            return BRONZE_STATUS_NOT_FOUND
        }
    }

    func outstanding() -> UInt64 {
        lock.lock()
        defer { lock.unlock() }
        var count: UInt64 = 0
        for phase in probes.values {
            if case .open = phase {
                count += 1
            }
        }
        return count
    }

    private func ownedCopyLocked(
        _ src: bronze_native_utf8_view,
        _ out: UnsafeMutablePointer<bronze_native_utf8_view>
    ) -> UInt32 {
        if src.len == 0 {
            emptyOwned += 1
            out.pointee = bronze_native_utf8_view(ptr: nil, len: 0)
            return BRONZE_STATUS_OK
        }
        guard let srcPtr = src.ptr else {
            return BRONZE_STATUS_INVALID_UTF8
        }
        let count = Int(src.len)
        let dest = UnsafeMutablePointer<UInt8>.allocate(capacity: count)
        dest.initialize(from: srcPtr, count: count)
        owned.insert(OwnedKey(ptr: UInt(bitPattern: dest), len: src.len))
        out.pointee = bronze_native_utf8_view(ptr: UnsafePointer(dest), len: src.len)
        return BRONZE_STATUS_OK
    }

    private func freeLocked(_ view: bronze_native_utf8_view) -> UInt32 {
        if view.len == 0 {
            if view.ptr != nil {
                return BRONZE_STATUS_NOT_FOUND
            }
            if emptyOwned == 0 {
                return BRONZE_STATUS_DOUBLE_COMPLETION
            }
            emptyOwned -= 1
            return BRONZE_STATUS_OK
        }
        guard let ptr = view.ptr else {
            return BRONZE_STATUS_NOT_FOUND
        }
        let key = OwnedKey(ptr: UInt(bitPattern: ptr), len: view.len)
        guard owned.remove(key) != nil else {
            return BRONZE_STATUS_DOUBLE_COMPLETION
        }
        let raw = UnsafeMutablePointer<UInt8>(mutating: ptr)
        raw.deinitialize(count: Int(view.len))
        raw.deallocate()
        return BRONZE_STATUS_OK
    }

    private func releaseAllOwnedLocked() {
        for (id, phase) in probes {
            if case let .open(view) = phase {
                _ = freeLocked(view)
                probes[id] = .cancelled
            }
        }
        for key in owned {
            if let ptr = UnsafeMutablePointer<UInt8>(bitPattern: key.ptr) {
                ptr.deinitialize(count: Int(key.len))
                ptr.deallocate()
            }
        }
        owned.removeAll()
        emptyOwned = 0
    }
}

@_cdecl("bronze_native_init")
public func bronze_native_init() {
    NativeOwnership.shared.reset()
}

@_cdecl("bronze_native_shutdown")
public func bronze_native_shutdown() {
    NativeOwnership.shared.shutdownCancelOpen()
}

@_silgen_name("bronze_native_utf8_owned_copy")
public func bronze_native_utf8_owned_copy(
    _ src: bronze_native_utf8_view,
    _ out: UnsafeMutablePointer<bronze_native_utf8_view>
) -> UInt32 {
    NativeOwnership.shared.ownedCopy(src, out)
}

@_silgen_name("bronze_native_utf8_free")
public func bronze_native_utf8_free(_ view: bronze_native_utf8_view) -> UInt32 {
    NativeOwnership.shared.free(view)
}

@_silgen_name("bronze_native_probe_begin")
public func bronze_native_probe_begin(_ request_id: UInt64, _ view: bronze_native_utf8_view) -> UInt32 {
    NativeOwnership.shared.begin(id: request_id, view: view)
}

@_cdecl("bronze_native_probe_complete")
public func bronze_native_probe_complete(_ request_id: UInt64) -> UInt32 {
    NativeOwnership.shared.complete(id: request_id)
}

@_cdecl("bronze_native_probe_cancel")
public func bronze_native_probe_cancel(_ request_id: UInt64) -> UInt32 {
    NativeOwnership.shared.cancel(id: request_id)
}

@_cdecl("bronze_native_probe_outstanding")
public func bronze_native_probe_outstanding() -> UInt64 {
    NativeOwnership.shared.outstanding()
}

func bronzeOwnedBytesCopy(
    _ src: bronze_native_utf8_view,
    _ out: UnsafeMutablePointer<bronze_native_utf8_view>
) -> UInt32 {
    NativeOwnership.shared.ownedRawCopy(src, out)
}

final class BronzeOutViewBox: @unchecked Sendable {
    let ptr: UnsafeMutablePointer<bronze_native_utf8_view>
    init(_ ptr: UnsafeMutablePointer<bronze_native_utf8_view>) {
        self.ptr = ptr
    }
}

// AppKit pasteboard and workspace icon APIs require the main thread.
func bronzeOnAppKit(_ work: @escaping @Sendable () -> UInt32) -> UInt32 {
    if Thread.isMainThread {
        return work()
    }
    let box = MainStatusBox()
    let lock = DispatchSemaphore(value: 0)
    DispatchQueue.main.async {
        box.value = work()
        lock.signal()
    }
    if lock.wait(timeout: .now() + 2) == .timedOut {
        return BRONZE_STATUS_DEGRADED
    }
    return box.value
}

/// Modal panels stay open until the operator dismisses them. The 2s hop
/// used by pasteboard and icons must not wrap NSOpenPanel.
func bronzeOnAppKitModal(_ work: @escaping @Sendable () -> UInt32) -> UInt32 {
    if Thread.isMainThread {
        return work()
    }
    let box = MainStatusBox()
    let lock = DispatchSemaphore(value: 0)
    DispatchQueue.main.async {
        box.value = work()
        lock.signal()
    }
    lock.wait()
    return box.value
}

private final class MainStatusBox: @unchecked Sendable {
    var value: UInt32 = BRONZE_STATUS_DEGRADED
}

func bronzeUtf8String(_ view: bronze_native_utf8_view) -> String? {
    if view.ptr == nil {
        return view.len == 0 ? "" : nil
    }
    if view.len > UInt64(Int.max) {
        return nil
    }
    let count = Int(view.len)
    let buffer = UnsafeBufferPointer(start: view.ptr, count: count)
    let bytes = Array(buffer)
    let decoded = String(decoding: bytes, as: UTF8.self)
    let reencoded = Array(decoded.utf8)
    guard reencoded == bytes else {
        return nil
    }
    return decoded
}
