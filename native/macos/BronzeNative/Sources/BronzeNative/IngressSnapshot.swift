// Fixed-size ingress snapshot for the tap thread (story 3.4).
// No titles, URLs, or focused-element fields. Seqlock load fails closed.

import Darwin

public struct BronzeIngressSnapshot: Sendable {
    public var targetPid: Int32
    public var bundleToken: UInt64
    public var activationGeneration: UInt64
    public var destinationUuidHi: UInt64
    public var destinationUuidLo: UInt64
    public var acceptCaptureGeneration: UInt64
    public var policyRevision: UInt64
    public var settingsRevision: UInt64
    public var contextGeneration: UInt64
    public var route: UInt32
    public var monotonicTimeNs: UInt64

    public static let empty = BronzeIngressSnapshot(
        targetPid: 0,
        bundleToken: 0,
        activationGeneration: 0,
        destinationUuidHi: 0,
        destinationUuidLo: 0,
        acceptCaptureGeneration: 0,
        policyRevision: 0,
        settingsRevision: 0,
        contextGeneration: 0,
        route: 0,
        monotonicTimeNs: 0
    )
}

public final class IngressSeqlock: @unchecked Sendable {
    private var seq: UInt64 = 0
    private var slot = BronzeIngressSnapshot.empty
    private var forceOdd = false
    private let lock = UnsafeMutablePointer<os_unfair_lock>.allocate(capacity: 1)

    public init() {
        lock.initialize(to: os_unfair_lock())
    }

    deinit {
        lock.deinitialize(count: 1)
        lock.deallocate()
    }

    public func publish(_ snapshot: BronzeIngressSnapshot) {
        os_unfair_lock_lock(lock)
        seq &+= 1
        slot = snapshot
        seq &+= 1
        os_unfair_lock_unlock(lock)
    }

    public func testBeginInconsistentWrite() {
        os_unfair_lock_lock(lock)
        forceOdd = true
        seq &+= 1
        os_unfair_lock_unlock(lock)
    }

    public func testEndInconsistentWrite() {
        os_unfair_lock_lock(lock)
        forceOdd = false
        if seq & 1 == 1 {
            seq &+= 1
        }
        os_unfair_lock_unlock(lock)
    }

    public func load() -> BronzeIngressSnapshot? {
        os_unfair_lock_lock(lock)
        defer { os_unfair_lock_unlock(lock) }
        if forceOdd || (seq & 1) == 1 {
            return nil
        }
        return slot
    }
}

enum IngressRuntime {
    static let shared = IngressSeqlock()
}
