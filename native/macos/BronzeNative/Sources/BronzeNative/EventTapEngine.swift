// Session listenOnly tap engine (story 3.3, CAP-002, CAP-004, ADR-005).
// Callback path: flagsChanged / keyDown → FSM → SPSC. Always pass the event
// through. No accessibility queries, workspace lookups, DB, window,
// clipboard, string allocation, or logging.

import ApplicationServices
import Carbon.HIToolbox
import CoreGraphics
import Darwin
import Foundation

public enum EventTapHealth: UInt32, Sendable {
    case idle = 0
    case listening = 1
    case degraded = 2
}

public final class EventTapEngine: @unchecked Sendable {
    public let spsc = EventTapSPSC()
    public private(set) var health: EventTapHealth = .idle
    public private(set) var fsm: DoubleTapFSM
    private var pressedLeft = false
    private var pressedRight = false
    private var tap: CFMachPort?
    private var source: CFRunLoopSource?
    private var runLoop: CFRunLoop?
    private var thread: Thread?
    private let ready = DispatchSemaphore(value: 0)
    private var gestureEnabled = false

    public init() {
        fsm = DoubleTapFSM(config: .defaults)
    }

    public var fsmStateRaw: UInt32 {
        switch fsm.state {
        case .idle: return 0
        case .firstDown: return 1
        case .firstUp: return 2
        case .secondDown: return 3
        case .triggered: return 4
        case .refractory: return 5
        }
    }

    public func setGestureEnabled(_ enabled: Bool) {
        gestureEnabled = enabled
        applyConfig()
    }

    /// Bind this thread as the sole SPSC producer. No live tap (unit tests).
    public func attachTestProducer() {
        stopLiveTap()
        spsc.reset()
        applyConfig()
        spsc.bindProducer()
        health = .idle
    }

    /// Install a session listenOnly tap on a dedicated run loop. Never prompts.
    public func startLive() -> EventTapHealth {
        stopLiveTap()
        spsc.reset()
        applyConfig()
        let mask: CGEventMask =
            (1 << CGEventType.flagsChanged.rawValue)
                | (1 << CGEventType.keyDown.rawValue)
                | (1 << CGEventType.tapDisabledByTimeout.rawValue)
                | (1 << CGEventType.tapDisabledByUserInput.rawValue)
        let unmanaged = Unmanaged.passUnretained(self)
        guard let port = CGEvent.tapCreate(
            tap: .cgSessionEventTap,
            place: .tailAppendEventTap,
            options: .listenOnly,
            eventsOfInterest: mask,
            callback: EventTapEngine.callback,
            userInfo: unmanaged.toOpaque()
        ) else {
            health = .degraded
            spsc.bindProducer()
            return health
        }
        tap = port
        source = CFMachPortCreateRunLoopSource(kCFAllocatorDefault, port, 0)
        let thread = Thread { [weak self] in
            self?.runTapLoop()
        }
        thread.name = "bronze.event-tap"
        self.thread = thread
        thread.start()
        _ = ready.wait(timeout: .now() + 2)
        health = .listening
        return health
    }

    public func stop() {
        stopLiveTap()
        fsm.reset()
        clearPressed()
        spsc.reset()
        health = .idle
    }

    /// Same path as a CG callback, without a CGEvent. Always "pass through".
    public func feedTest(kind: UInt32, carbonKey: Int32, timeNs: UInt64) -> Bool {
        switch kind {
        case 3:
            return applyOutcome(fsm.handle(DoubleTapEvent(kind: .cancel, side: nil, timeNs: timeNs)))
        case 1, 2:
            let eventKind: DoubleTapEventKind = kind == 1 ? .down : .up
            let side = ModifierSide(carbonKeyCode: Int(carbonKey))
            if side == nil, eventKind != .cancel {
                return applyOutcome(fsm.handle(DoubleTapEvent(kind: .cancel, side: nil, timeNs: timeNs)))
            }
            return applyOutcome(fsm.handle(DoubleTapEvent(kind: eventKind, side: side, timeNs: timeNs)))
        default:
            return false
        }
    }

    public func simulateTapDisabled() -> Bool {
        fsm.reset()
        clearPressed()
        return spsc.enqueue(.disabled)
    }

    /// Attempt an enqueue from the calling thread. Foreign threads must fail.
    public func enqueueFromCaller(_ kind: EventTapRecordKind) -> Bool {
        spsc.enqueue(kind)
    }

    /// Pass-through proof for tests: incoming event identity is unchanged.
    public func passThrough(_ event: CGEvent) -> CGEvent {
        event
    }

    private func applyConfig() {
        let config = DoubleTapConfig(
            enabled: gestureEnabled,
            gapMs: 250,
            maxHoldMs: 400,
            refractoryMs: 500,
            side: .either
        ) ?? .defaults
        fsm = DoubleTapFSM(config: config)
        fsm.reset()
        clearPressed()
    }

    private func clearPressed() {
        pressedLeft = false
        pressedRight = false
    }

    private func applyOutcome(_ outcome: DoubleTapOutcome) -> Bool {
        if outcome.triggered {
            return spsc.enqueue(.trigger)
        }
        return true
    }

    private func runTapLoop() {
        let rl = CFRunLoopGetCurrent()
        runLoop = rl
        if let source {
            CFRunLoopAddSource(rl, source, .commonModes)
        }
        spsc.bindProducer()
        ready.signal()
        CFRunLoopRun()
    }

    private func stopLiveTap() {
        if let rl = runLoop {
            CFRunLoopStop(rl)
        }
        if let source, let rl = runLoop {
            CFRunLoopRemoveSource(rl, source, .commonModes)
        }
        if let tap {
            CGEvent.tapEnable(tap: tap, enable: false)
        }
        tap = nil
        source = nil
        runLoop = nil
        thread = nil
    }

    private func handleLive(type: CGEventType, event: CGEvent) {
        let timeNs = event.timestamp
        switch type {
        case .tapDisabledByTimeout, .tapDisabledByUserInput:
            fsm.reset()
            clearPressed()
            _ = spsc.enqueue(.disabled)
            if let tap {
                CGEvent.tapEnable(tap: tap, enable: true)
            }
        case .flagsChanged:
            let key = Int(event.getIntegerValueField(.keyboardEventKeycode))
            if let side = ModifierSide(carbonKeyCode: key) {
                let down: Bool
                switch side {
                case .left:
                    down = !pressedLeft
                    pressedLeft = down
                case .right:
                    down = !pressedRight
                    pressedRight = down
                }
                _ = applyOutcome(fsm.handle(DoubleTapEvent(
                    kind: down ? .down : .up,
                    side: side,
                    timeNs: timeNs
                )))
            } else {
                _ = applyOutcome(fsm.handle(DoubleTapEvent(kind: .cancel, side: nil, timeNs: timeNs)))
            }
        case .keyDown:
            let key = Int(event.getIntegerValueField(.keyboardEventKeycode))
            if ModifierSide(carbonKeyCode: key) == nil {
                _ = applyOutcome(fsm.handle(DoubleTapEvent(kind: .cancel, side: nil, timeNs: timeNs)))
            }
        default:
            break
        }
    }

    private static let callback: CGEventTapCallBack = { _, type, event, refcon in
        if let refcon {
            let engine = Unmanaged<EventTapEngine>.fromOpaque(refcon).takeUnretainedValue()
            engine.handleLive(type: type, event: event)
        }
        return Unmanaged.passUnretained(event)
    }
}

enum EventTapRuntime {
    static let shared = EventTapEngine()
}
