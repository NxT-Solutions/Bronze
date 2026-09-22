// DoubleTapFSM.swift
// Pure Shift double-tap machine for CAP-002. Timing uses injected monotonic
// nanoseconds (CGEventTimestamp shape). No live tap, AX, AppKit, pasteboard,
// wall clock, key stream, or character logging (ADR-005).
//
// CAP-002, A11Y-001, ADR-005
// - P0 modifier is Shift only (docs/12-settings-and-shortcuts.md §7)
// - Sides from Carbon kVK_Shift / kVK_RightShift; transitions use left | right
// - Default enabled == false; non-timed chord/menu/composer stay independent
// - One valid pair → one trigger on the second release; invalid → zero
// - Reset clears timestamps and sides (wake / disable / settings)

import Carbon.HIToolbox

// MARK: - Side (Carbon mapper)

/// Physical Shift side. FSM transitions never carry raw keycodes.
public enum ModifierSide: Equatable, Sendable {
    case left
    case right

    /// Accepts only Carbon `kVK_Shift` / `kVK_RightShift`. Other modifiers are
    /// not stored; callers send `cancel` with no keycode.
    public init?(carbonKeyCode: Int) {
        switch carbonKeyCode {
        case kVK_Shift:
            self = .left
        case kVK_RightShift:
            self = .right
        default:
            return nil
        }
    }
}

/// CAP-002 side policy (`docs/12-settings-and-shortcuts.md` §7).
public enum ModifierSidePolicy: Equatable, Sendable {
    case either
    case left
    case right
    case same
}

// MARK: - Config

/// Timing and side policy. Defaults match `docs/07-macos-capture-reliability.md` §6.
public struct DoubleTapConfig: Equatable, Sendable {
    public static let gapMsAllowed = 150...900
    public static let maxHoldMsAllowed = 100...1_500
    public static let refractoryMsAllowed = 100...1_500
    public static let debounceMsDefault = 30

    public static let tapCountAllowed = 2...8

    public let enabled: Bool
    public let gapMs: Int
    public let maxHoldMs: Int
    public let debounceMs: Int
    public let refractoryMs: Int
    public let side: ModifierSidePolicy
    public let tapCount: Int

    /// Disabled gesture; gap 250, hold 400, debounce 30, refractory 500, side either.
    public static let defaults = DoubleTapConfig(
        enabled: false,
        gapMs: 250,
        maxHoldMs: 400,
        refractoryMs: 500,
        side: .either
    )!

    /// In-range constructor. Gap must accept 500 ms (EN 301 549 5.8 / A11Y-001).
    public init?(
        enabled: Bool,
        gapMs: Int,
        maxHoldMs: Int,
        refractoryMs: Int,
        side: ModifierSidePolicy,
        debounceMs: Int = DoubleTapConfig.debounceMsDefault,
        tapCount: Int = 2
    ) {
        guard debounceMs >= 0 else { return nil }
        let (_, debounceOverflow) = UInt64(debounceMs).multipliedReportingOverflow(by: 1_000_000)
        guard Self.gapMsAllowed.contains(gapMs),
              Self.maxHoldMsAllowed.contains(maxHoldMs),
              Self.refractoryMsAllowed.contains(refractoryMs),
              Self.tapCountAllowed.contains(tapCount),
              !debounceOverflow
        else {
            return nil
        }
        self.enabled = enabled
        self.gapMs = gapMs
        self.maxHoldMs = maxHoldMs
        self.debounceMs = debounceMs
        self.refractoryMs = refractoryMs
        self.side = side
        self.tapCount = tapCount
    }

    public var gapNs: UInt64 { Self.msToNs(gapMs) }
    public var maxHoldNs: UInt64 { Self.msToNs(maxHoldMs) }
    public var debounceNs: UInt64 { Self.msToNs(debounceMs) }
    public var refractoryNs: UInt64 { Self.msToNs(refractoryMs) }

    public static func msToNs(_ ms: Int) -> UInt64 {
        guard ms >= 0 else { return 0 }
        let (n, overflow) = UInt64(ms).multipliedReportingOverflow(by: 1_000_000)
        return overflow ? UInt64.max : n
    }
}

// MARK: - Events and states

/// `{kind, side?, timeNs}` only. No character, no keycode payload.
public enum DoubleTapEventKind: Equatable, Sendable {
    case down
    case up
    case cancel
    case reset
}

public struct DoubleTapEvent: Equatable, Sendable {
    public var kind: DoubleTapEventKind
    public var side: ModifierSide?
    public var timeNs: UInt64

    public init(kind: DoubleTapEventKind, side: ModifierSide?, timeNs: UInt64) {
        self.kind = kind
        self.side = side
        self.timeNs = timeNs
    }
}

/// `docs/07-macos-capture-reliability.md` §6: Idle → FirstDown → FirstUp →
/// SecondDown → Triggered → Refractory → Idle. Trigger on second up.
public enum DoubleTapState: String, Equatable, Sendable, CaseIterable {
    case idle
    case firstDown
    case firstUp
    case secondDown
    case triggered
    case refractory
}

public struct DoubleTapOutcome: Equatable, Sendable {
    public var triggered: Bool
    public var state: DoubleTapState

    public init(triggered: Bool, state: DoubleTapState) {
        self.triggered = triggered
        self.state = state
    }
}

/// Live candidate only — never a retained key stream.
public struct DoubleTapLiveCandidate: Equatable, Sendable {
    public var firstSide: ModifierSide?
    public var firstDownNs: UInt64?
    public var firstUpNs: UInt64?
    public var secondSide: ModifierSide?
    public var secondDownNs: UInt64?
}

// MARK: - FSM

/// Timestamp-injected Shift double-tap FSM. `handle` / `reset` are tap-free
/// so Story 3.3 can feed events from a listen-only callback later.
public struct DoubleTapFSM: Equatable, Sendable {
    public let config: DoubleTapConfig
    public private(set) var state: DoubleTapState

    private var firstSide: ModifierSide?
    private var firstDownNs: UInt64?
    private var firstUpNs: UInt64?
    private var secondSide: ModifierSide?
    private var secondDownNs: UInt64?
    private var lastKind: DoubleTapEventKind?
    private var lastSide: ModifierSide?
    private var lastTimeNs: UInt64?
    private var refractoryUntilNs: UInt64?
    private var tapsCompleted: Int

    public init(config: DoubleTapConfig = .defaults) {
        self.config = config
        self.state = .idle
        self.tapsCompleted = 0
    }

    /// Current state plus at most the live candidate times/sides. No `[keyCode]`
    /// or `Character` history (CAP-002 / ADR-005).
    public var liveCandidate: DoubleTapLiveCandidate {
        switch state {
        case .firstDown, .firstUp, .secondDown:
            return DoubleTapLiveCandidate(
                firstSide: firstSide,
                firstDownNs: firstDownNs,
                firstUpNs: firstUpNs,
                secondSide: secondSide,
                secondDownNs: secondDownNs
            )
        case .idle, .triggered, .refractory:
            return DoubleTapLiveCandidate(
                firstSide: nil,
                firstDownNs: nil,
                firstUpNs: nil,
                secondSide: nil,
                secondDownNs: nil
            )
        }
    }

    /// Wake / disable / settings / tap-disabled: clear timestamps and sides.
    public mutating func reset() {
        clearToIdle(clearRefractory: true)
    }

    public mutating func handle(_ event: DoubleTapEvent) -> DoubleTapOutcome {
        if event.kind == .reset {
            reset()
            return DoubleTapOutcome(triggered: false, state: .idle)
        }

        guard config.enabled else {
            return DoubleTapOutcome(triggered: false, state: .idle)
        }

        if event.kind == .cancel {
            clearToIdle(clearRefractory: true)
            return DoubleTapOutcome(triggered: false, state: .idle)
        }

        if state == .refractory {
            if let until = refractoryUntilNs, event.timeNs < until {
                // Refractory ignores new candidates (downs and completing ups).
                return DoubleTapOutcome(triggered: false, state: .refractory)
            }
            clearToIdle(clearRefractory: true)
        }

        if let last = lastTimeNs, event.timeNs < last {
            clearToIdle(clearRefractory: true)
            return DoubleTapOutcome(triggered: false, state: .idle)
        }

        guard event.kind == .down || event.kind == .up else {
            clearToIdle(clearRefractory: true)
            return DoubleTapOutcome(triggered: false, state: .idle)
        }

        guard let side = event.side else {
            clearToIdle(clearRefractory: true)
            return DoubleTapOutcome(triggered: false, state: .idle)
        }

        if isDebounceDuplicate(kind: event.kind, side: side, timeNs: event.timeNs) {
            return DoubleTapOutcome(triggered: false, state: state)
        }

        switch (state, event.kind) {
        case (.idle, .down):
            if isAllowedFirstSide(side) {
                enterFirstDown(side: side, timeNs: event.timeNs)
            }
            return DoubleTapOutcome(triggered: false, state: state)

        case (.idle, .up):
            return DoubleTapOutcome(triggered: false, state: .idle)

        case (.firstDown, .up):
            if side == firstSide, intervalOK(from: firstDownNs, to: event.timeNs, limit: config.maxHoldNs) {
                tapsCompleted = 1
                enterFirstUp(timeNs: event.timeNs)
                return DoubleTapOutcome(triggered: false, state: state)
            }
            clearToIdle(clearRefractory: true)
            return DoubleTapOutcome(triggered: false, state: .idle)

        case (.firstDown, .down):
            // Other side = overlap / both-down; same side after debounce = missed release.
            clearToIdle(clearRefractory: true)
            return DoubleTapOutcome(triggered: false, state: .idle)

        case (.firstUp, .down):
            if isAllowedSecondSide(side), intervalOK(from: firstUpNs, to: event.timeNs, limit: config.gapNs) {
                enterSecondDown(side: side, timeNs: event.timeNs)
                return DoubleTapOutcome(triggered: false, state: state)
            }
            clearToIdle(clearRefractory: true)
            return DoubleTapOutcome(triggered: false, state: .idle)

        case (.firstUp, .up):
            clearToIdle(clearRefractory: true)
            return DoubleTapOutcome(triggered: false, state: .idle)

        case (.secondDown, .up):
            if side == secondSide, intervalOK(from: secondDownNs, to: event.timeNs, limit: config.maxHoldNs) {
                tapsCompleted += 1
                if tapsCompleted >= config.tapCount {
                    emitTrigger(at: event.timeNs)
                    return DoubleTapOutcome(triggered: true, state: .refractory)
                }
                enterFirstUp(timeNs: event.timeNs)
                return DoubleTapOutcome(triggered: false, state: state)
            }
            clearToIdle(clearRefractory: true)
            return DoubleTapOutcome(triggered: false, state: .idle)

        case (.secondDown, .down):
            clearToIdle(clearRefractory: true)
            return DoubleTapOutcome(triggered: false, state: .idle)

        case (.triggered, _), (.refractory, _):
            clearToIdle(clearRefractory: true)
            return DoubleTapOutcome(triggered: false, state: .idle)

        default:
            clearToIdle(clearRefractory: true)
            return DoubleTapOutcome(triggered: false, state: .idle)
        }
    }

    // MARK: Transitions

    private mutating func enterFirstDown(side: ModifierSide, timeNs: UInt64) {
        state = .firstDown
        tapsCompleted = 0
        firstSide = side
        firstDownNs = timeNs
        firstUpNs = nil
        secondSide = nil
        secondDownNs = nil
        recordProcessed(kind: .down, side: side, timeNs: timeNs)
    }

    private mutating func enterFirstUp(timeNs: UInt64) {
        state = .firstUp
        firstUpNs = timeNs
        recordProcessed(kind: .up, side: firstSide, timeNs: timeNs)
    }

    private mutating func enterSecondDown(side: ModifierSide, timeNs: UInt64) {
        state = .secondDown
        secondSide = side
        secondDownNs = timeNs
        recordProcessed(kind: .down, side: side, timeNs: timeNs)
    }

    private mutating func emitTrigger(at timeNs: UInt64) {
        // Triggered is instantaneous; observable dwell is Refractory (docs/07 §6).
        state = .triggered
        let (until, overflow) = timeNs.addingReportingOverflow(config.refractoryNs)
        refractoryUntilNs = overflow ? UInt64.max : until
        firstSide = nil
        firstDownNs = nil
        firstUpNs = nil
        secondSide = nil
        secondDownNs = nil
        lastKind = nil
        lastSide = nil
        lastTimeNs = timeNs
        state = .refractory
    }

    private mutating func clearToIdle(clearRefractory: Bool) {
        state = .idle
        firstSide = nil
        firstDownNs = nil
        firstUpNs = nil
        secondSide = nil
        secondDownNs = nil
        lastKind = nil
        lastSide = nil
        lastTimeNs = nil
        tapsCompleted = 0
        if clearRefractory {
            refractoryUntilNs = nil
        }
    }

    private mutating func recordProcessed(kind: DoubleTapEventKind, side: ModifierSide?, timeNs: UInt64) {
        lastKind = kind
        lastSide = side
        lastTimeNs = timeNs
    }

    // MARK: Rules

    private func isDebounceDuplicate(kind: DoubleTapEventKind, side: ModifierSide, timeNs: UInt64) -> Bool {
        guard let lastKind, let lastSide, let lastTimeNs else { return false }
        guard lastKind == kind, lastSide == side, timeNs >= lastTimeNs else { return false }
        return (timeNs - lastTimeNs) <= config.debounceNs
    }

    private func isAllowedFirstSide(_ side: ModifierSide) -> Bool {
        switch config.side {
        case .either, .same:
            return true
        case .left:
            return side == .left
        case .right:
            return side == .right
        }
    }

    private func isAllowedSecondSide(_ side: ModifierSide) -> Bool {
        switch config.side {
        case .either:
            return true
        case .same:
            return side == firstSide
        case .left:
            return side == .left
        case .right:
            return side == .right
        }
    }

    private func intervalOK(from start: UInt64?, to end: UInt64, limit: UInt64) -> Bool {
        guard let start, end >= start else { return false }
        return (end - start) <= limit
    }
}
