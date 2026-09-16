// Bounded SPSC for the listen-only tap thread (docs/07 §5.1, CAP-004).
// Exactly one producer: the dedicated tap thread (or the test-attached thread).
// Other routes must not write. Callback-safe: no AX, DB, window, clipboard, log.

import Darwin

public enum EventTapRecordKind: UInt32, Sendable {
    case none = 0
    case trigger = 1
    case reset = 2
    case disabled = 3
}

public struct EventTapRecord: Equatable, Sendable {
    public var kind: EventTapRecordKind
    public var sequence: UInt64
}

public final class EventTapSPSC: @unchecked Sendable {
    public static let capacity = 32

    private var kinds: [UInt32]
    private var seqs: [UInt64]
    private var head = 0
    private var tail = 0
    private var nextSequence: UInt64 = 1
    private var completedSequence: UInt64 = 0
    private var producerTid: UInt = 0
    private let lock = UnsafeMutablePointer<os_unfair_lock>.allocate(capacity: 1)

    public init() {
        kinds = Array(repeating: 0, count: Self.capacity)
        seqs = Array(repeating: 0, count: Self.capacity)
        lock.initialize(to: os_unfair_lock())
    }

    deinit {
        lock.deinitialize(count: 1)
        lock.deallocate()
    }

    public func bindProducer() {
        os_unfair_lock_lock(lock)
        producerTid = UInt(bitPattern: pthread_self())
        os_unfair_lock_unlock(lock)
    }

    public var hasProducer: Bool {
        os_unfair_lock_lock(lock)
        defer { os_unfair_lock_unlock(lock) }
        return producerTid != 0
    }

    /// Returns false if the caller is not the bound producer (no write).
    public func enqueue(_ kind: EventTapRecordKind) -> Bool {
        os_unfair_lock_lock(lock)
        defer { os_unfair_lock_unlock(lock) }
        let tid = UInt(bitPattern: pthread_self())
        if producerTid == 0 || tid != producerTid {
            return false
        }
        let seq = nextSequence
        nextSequence &+= 1
        let nextTail = (tail + 1) % Self.capacity
        if nextTail != head {
            kinds[tail] = kind.rawValue
            seqs[tail] = seq
            tail = nextTail
        }
        completedSequence = seq
        return true
    }

    public func drain() -> EventTapRecord? {
        os_unfair_lock_lock(lock)
        defer { os_unfair_lock_unlock(lock) }
        if head == tail {
            return nil
        }
        let kind = EventTapRecordKind(rawValue: kinds[head]) ?? .none
        let seq = seqs[head]
        head = (head + 1) % Self.capacity
        return EventTapRecord(kind: kind, sequence: seq)
    }

    public func reset() {
        os_unfair_lock_lock(lock)
        head = 0
        tail = 0
        nextSequence = 1
        completedSequence = 0
        producerTid = 0
        os_unfair_lock_unlock(lock)
    }
}
