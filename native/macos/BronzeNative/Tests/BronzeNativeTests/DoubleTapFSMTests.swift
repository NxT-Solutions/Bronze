// DoubleTapFSMTests.swift
// Table and property coverage for the Shift double-tap FSM (CAP-002, A11Y-001, ADR-005).
// Type and method names include `FSM` so `swift test --filter FSM` selects this file only.
//
// I/O rows: VALID_PAIR, DISABLED_DEFAULT, WINDOW_500, HOLD_GAP_BOUNDS,
// INVALID_CANCEL, SIDE_POLICY, REFRACTORY, RESET_WAKE, DEBOUNCE_DUP,
// NO_STREAM, TIME_INJECTED, PROPERTY_ARBITRARY.

import Carbon.HIToolbox
import XCTest
import BronzeNative

final class DoubleTapFSMTests: XCTestCase {

    private let ms: UInt64 = 1_000_000

    // MARK: - Config / mapper (ACs)

    func testFSM_DISABLED_DEFAULT_defaultsAreDisabled() {
        let defaults = DoubleTapConfig.defaults
        XCTAssertFalse(defaults.enabled)
        XCTAssertEqual(defaults.gapMs, 250)
        XCTAssertEqual(defaults.maxHoldMs, 400)
        XCTAssertEqual(defaults.debounceMs, 30)
        XCTAssertEqual(defaults.refractoryMs, 500)
        XCTAssertEqual(defaults.side, .either)
        XCTAssertTrue(DoubleTapConfig.gapMsAllowed.contains(500))
        XCTAssertNotNil(
            DoubleTapConfig(enabled: true, gapMs: 500, maxHoldMs: 400, refractoryMs: 500, side: .either)
        )
        XCTAssertNil(
            DoubleTapConfig(enabled: true, gapMs: 149, maxHoldMs: 400, refractoryMs: 500, side: .either)
        )
        XCTAssertNil(
            DoubleTapConfig(enabled: true, gapMs: 901, maxHoldMs: 400, refractoryMs: 500, side: .either)
        )
        XCTAssertNotNil(
            DoubleTapConfig(
                enabled: true, gapMs: 250, maxHoldMs: 400, refractoryMs: 500, side: .either, debounceMs: 0
            )
        )
        XCTAssertNil(
            DoubleTapConfig(
                enabled: true, gapMs: 250, maxHoldMs: 400, refractoryMs: 500, side: .either, debounceMs: -1
            )
        )
        XCTAssertNil(
            DoubleTapConfig(
                enabled: true, gapMs: 250, maxHoldMs: 400, refractoryMs: 500, side: .either, debounceMs: Int.max
            )
        )
    }

    func testFSM_carbonKeyCodeMapperAcceptsOnlyShift() {
        XCTAssertEqual(ModifierSide(carbonKeyCode: kVK_Shift), .left)
        XCTAssertEqual(ModifierSide(carbonKeyCode: kVK_RightShift), .right)
        XCTAssertNil(ModifierSide(carbonKeyCode: kVK_Control))
        XCTAssertNil(ModifierSide(carbonKeyCode: kVK_RightControl))
        XCTAssertNil(ModifierSide(carbonKeyCode: kVK_Option))
        XCTAssertNil(ModifierSide(carbonKeyCode: kVK_RightOption))
        XCTAssertNil(ModifierSide(carbonKeyCode: kVK_Command))
        XCTAssertNil(ModifierSide(carbonKeyCode: kVK_RightCommand))
        XCTAssertNil(ModifierSide(carbonKeyCode: 0))
    }

    func testFSM_statesMatchDocs07Section6() {
        XCTAssertEqual(
            DoubleTapState.allCases,
            [.idle, .firstDown, .firstUp, .secondDown, .triggered, .refractory]
        )
    }

    func testFSM_doesNotGateNonTimedABIRoutes_A11Y001() {
        XCTAssertEqual(bronze_native_abi_version(), BRONZE_ABI_VERSION)
        bronze_native_init()
        bronze_native_shutdown()
    }

    // MARK: - I/O matrix

    func testFSM_VALID_PAIR_sameAndMixedEmitOneTrigger() {
        let cases: [(String, ModifierSide, ModifierSide)] = [
            ("same-left", .left, .left),
            ("same-right", .right, .right),
            ("mixed-LR", .left, .right),
            ("mixed-RL", .right, .left),
        ]
        for (name, first, second) in cases {
            var fsm = DoubleTapFSM(config: enabledConfig())
            let events = validPair(first: first, second: second)
            let outcomes = events.map { fsm.handle($0) }
            XCTAssertEqual(outcomes.count, 4, name)
            XCTAssertFalse(outcomes[0].triggered, "\(name) first down")
            XCTAssertFalse(outcomes[1].triggered, "\(name) first up")
            XCTAssertFalse(outcomes[2].triggered, "\(name) second down")
            XCTAssertTrue(outcomes[3].triggered, "\(name) second up")
            XCTAssertEqual(outcomes.filter(\.triggered).count, 1, name)
            XCTAssertEqual(fsm.state, .refractory, name)
        }
    }

    func testFSM_DISABLED_DEFAULT_validPairStaysIdle() {
        var fsm = DoubleTapFSM(config: .defaults)
        let triggers = run(validPair(first: .left, second: .left), on: &fsm)
        XCTAssertEqual(triggers, 0)
        XCTAssertEqual(fsm.state, .idle)
        XCTAssertNil(fsm.liveCandidate.firstSide)
        XCTAssertNil(fsm.liveCandidate.firstDownNs)
    }

    func testFSM_TRIPLE_doesNotTriggerOnSecondUp() {
        var fsm = DoubleTapFSM(config: enabledConfig(tapCount: 3))
        XCTAssertEqual(run(validPair(first: .left, second: .left), on: &fsm), 0)
        XCTAssertEqual(fsm.state, .firstUp)
        XCTAssertEqual(
            run([
                ev(.down, .left, DoubleTapConfig.msToNs(200)),
                ev(.up, .left, DoubleTapConfig.msToNs(240)),
            ], on: &fsm),
            1
        )
        XCTAssertEqual(fsm.state, .refractory)
    }

    func testFSM_WINDOW_500_acceptsExactGap() {
        let config = DoubleTapConfig(
            enabled: true, gapMs: 500, maxHoldMs: 400, refractoryMs: 500, side: .either
        )!
        var fsm = DoubleTapFSM(config: config)
        let events = [
            ev(.down, .left, 0),
            ev(.up, .left, 50 * ms),
            ev(.down, .left, 50 * ms + 500 * ms),
            ev(.up, .left, 50 * ms + 500 * ms + 40 * ms),
        ]
        XCTAssertEqual(run(events, on: &fsm), 1)
        XCTAssertEqual(fsm.state, .refractory)
    }

    func testFSM_HOLD_GAP_BOUNDS_equalsAcceptPlusOneCancels() {
        let hold = DoubleTapConfig.msToNs(400)
        let gap = DoubleTapConfig.msToNs(250)

        var equal = DoubleTapFSM(config: enabledConfig())
        let equalEvents = [
            ev(.down, .left, 0),
            ev(.up, .left, hold),
            ev(.down, .left, hold + gap),
            ev(.up, .left, hold + gap + hold),
        ]
        XCTAssertEqual(run(equalEvents, on: &equal), 1)

        var holdPlus = DoubleTapFSM(config: enabledConfig())
        let holdPlusEvents = [
            ev(.down, .left, 0),
            ev(.up, .left, hold + 1),
        ]
        XCTAssertEqual(run(holdPlusEvents, on: &holdPlus), 0)
        XCTAssertEqual(holdPlus.state, .idle)

        var gapPlus = DoubleTapFSM(config: enabledConfig())
        let gapPlusEvents = [
            ev(.down, .left, 0),
            ev(.up, .left, hold),
            ev(.down, .left, hold + gap + 1),
            ev(.up, .left, hold + gap + 1 + 20 * ms),
        ]
        XCTAssertEqual(run(gapPlusEvents, on: &gapPlus), 0)
        XCTAssertEqual(gapPlus.state, .idle)

        var secondHoldPlus = DoubleTapFSM(config: enabledConfig())
        let secondHoldPlusEvents = [
            ev(.down, .left, 0),
            ev(.up, .left, hold),
            ev(.down, .left, hold + gap),
            ev(.up, .left, hold + gap + hold + 1),
        ]
        XCTAssertEqual(run(secondHoldPlusEvents, on: &secondHoldPlus), 0)
        XCTAssertEqual(secondHoldPlus.state, .idle)
    }

    func testFSM_INVALID_CANCEL_nonShiftOtherModOverlapMissedReleaseReversed() {
        let midCandidatePrefix = [
            ev(.down, .left, 0),
            ev(.up, .left, 40 * ms),
        ]

        let cancelRows: [(String, [DoubleTapEvent])] = [
            ("non-Shift", midCandidatePrefix + [ev(.cancel, nil, 50 * ms)]),
            ("other-mod", midCandidatePrefix + [ev(.cancel, nil, 50 * ms)]),
            (
                "overlap",
                [ev(.down, .left, 0), ev(.down, .right, 10 * ms)]
            ),
            (
                "missed-release",
                [ev(.down, .left, 0), ev(.down, .left, 50 * ms)]
            ),
            (
                "reversed",
                [ev(.down, .left, 80 * ms), ev(.up, .left, 40 * ms)]
            ),
            (
                "idle-up",
                [ev(.up, .left, 0)]
            ),
            (
                "crossed-release-first",
                [ev(.down, .left, 0), ev(.up, .right, 40 * ms)]
            ),
            (
                "crossed-release-second",
                [
                    ev(.down, .left, 0),
                    ev(.up, .left, 40 * ms),
                    ev(.down, .left, 80 * ms),
                    ev(.up, .right, 120 * ms),
                ]
            ),
        ]

        for (name, events) in cancelRows {
            var fsm = DoubleTapFSM(config: enabledConfig())
            XCTAssertEqual(run(events, on: &fsm), 0, name)
            XCTAssertEqual(fsm.state, .idle, name)
            XCTAssertNil(fsm.liveCandidate.firstSide, name)
            XCTAssertNil(fsm.liveCandidate.firstDownNs, name)
        }

        var crossedFollow = DoubleTapFSM(config: enabledConfig())
        XCTAssertEqual(
            run(
                [
                    ev(.down, .left, 0),
                    ev(.up, .right, 40 * ms),
                    ev(.down, .right, 80 * ms),
                    ev(.up, .right, 120 * ms),
                ],
                on: &crossedFollow
            ),
            0
        )
        XCTAssertNotEqual(crossedFollow.state, .refractory)
    }

    func testFSM_SIDE_POLICY_sameAndLeftRejectCrossSide() {
        var same = DoubleTapFSM(config: enabledConfig(side: .same))
        XCTAssertEqual(run(validPair(first: .left, second: .right), on: &same), 0)
        XCTAssertEqual(same.state, .idle)

        var leftOnly = DoubleTapFSM(config: enabledConfig(side: .left))
        XCTAssertEqual(run(validPair(first: .right, second: .right), on: &leftOnly), 0)
        XCTAssertEqual(leftOnly.state, .idle)

        var sameOK = DoubleTapFSM(config: enabledConfig(side: .same))
        XCTAssertEqual(run(validPair(first: .left, second: .left), on: &sameOK), 1)

        var leftOK = DoubleTapFSM(config: enabledConfig(side: .left))
        XCTAssertEqual(run(validPair(first: .left, second: .left), on: &leftOK), 1)

        var rightOnly = DoubleTapFSM(config: enabledConfig(side: .right))
        XCTAssertEqual(run(validPair(first: .left, second: .left), on: &rightOnly), 0)
        XCTAssertEqual(rightOnly.state, .idle)

        var leftThenRight = DoubleTapFSM(config: enabledConfig(side: .left))
        XCTAssertEqual(run(validPair(first: .left, second: .right), on: &leftThenRight), 0)
        XCTAssertEqual(leftThenRight.state, .idle)

        var rightThenLeft = DoubleTapFSM(config: enabledConfig(side: .right))
        XCTAssertEqual(run(validPair(first: .right, second: .left), on: &rightThenLeft), 0)
        XCTAssertEqual(rightThenLeft.state, .idle)

        var rightOK = DoubleTapFSM(config: enabledConfig(side: .right))
        XCTAssertEqual(run(validPair(first: .right, second: .right), on: &rightOK), 1)
    }

    func testFSM_REFRACTORY_ignoresImmediateSecondPair() {
        var fsm = DoubleTapFSM(config: enabledConfig())
        var events = validPair(first: .left, second: .left, t0: 0)
        events += validPair(first: .right, second: .right, t0: 160 * ms)
        XCTAssertEqual(run(events, on: &fsm), 1)
        XCTAssertEqual(fsm.state, .refractory)

        var after = DoubleTapFSM(config: enabledConfig())
        var late = validPair(first: .left, second: .left, t0: 0)
        late += validPair(first: .left, second: .left, t0: 150 * ms + 500 * ms)
        XCTAssertEqual(run(late, on: &after), 2)
    }

    func testFSM_RESET_WAKE_thenFreshPair() {
        var fsm = DoubleTapFSM(config: enabledConfig())
        _ = fsm.handle(ev(.down, .left, 0))
        XCTAssertEqual(fsm.state, .firstDown)
        _ = fsm.handle(ev(.reset, nil, 10 * ms))
        XCTAssertEqual(fsm.state, .idle)
        XCTAssertNil(fsm.liveCandidate.firstDownNs)
        XCTAssertEqual(run(validPair(first: .left, second: .right, t0: 20 * ms), on: &fsm), 1)
    }

    func testFSM_DEBOUNCE_DUP_ignoresAndCompletes() {
        var fsm = DoubleTapFSM(config: enabledConfig())
        // Duplicate down at 20 ms must not move last timestamp: an up at 10 ms
        // is still valid against t=0 (hold 10 ms) and must not look reversed.
        let events = [
            ev(.down, .left, 0),
            ev(.down, .left, 20 * ms),
            ev(.up, .left, 10 * ms),
            ev(.down, .left, 80 * ms),
            ev(.up, .left, 120 * ms),
        ]
        XCTAssertEqual(run(events, on: &fsm), 1)

        var cancelAfterWindow = DoubleTapFSM(config: enabledConfig())
        let lateDup = [
            ev(.down, .left, 0),
            ev(.down, .left, 30 * ms + 1),
        ]
        XCTAssertEqual(run(lateDup, on: &cancelAfterWindow), 0)
        XCTAssertEqual(cancelAfterWindow.state, .idle)

        var exactWindow = DoubleTapFSM(config: enabledConfig())
        let exactDup = [
            ev(.down, .left, 0),
            ev(.down, .left, 30 * ms),
            ev(.up, .left, 50 * ms),
            ev(.down, .left, 100 * ms),
            ev(.up, .left, 140 * ms),
        ]
        XCTAssertEqual(run(exactDup, on: &exactWindow), 1)
        XCTAssertEqual(exactWindow.state, .refractory)

        var dupUp = DoubleTapFSM(config: enabledConfig())
        let duplicateUp = [
            ev(.down, .left, 0),
            ev(.up, .left, 40 * ms),
            ev(.up, .left, 55 * ms),
            ev(.down, .left, 80 * ms),
            ev(.up, .left, 120 * ms),
        ]
        XCTAssertEqual(run(duplicateUp, on: &dupUp), 1)
        XCTAssertEqual(dupUp.state, .refractory)
    }

    func testFSM_NO_STREAM_noKeyOrCharacterHistory() {
        var mid = DoubleTapFSM(config: enabledConfig())
        _ = mid.handle(ev(.down, .left, 0))
        XCTAssertEqual(mid.state, .firstDown)
        XCTAssertEqual(mid.liveCandidate.firstSide, .left)
        XCTAssertEqual(mid.liveCandidate.firstDownNs, 0)
        XCTAssertNil(mid.liveCandidate.firstUpNs)
        XCTAssertNil(mid.liveCandidate.secondSide)
        XCTAssertNil(mid.liveCandidate.secondDownNs)
        XCTAssertFalse(mirrorContainsKeyStream(Mirror(reflecting: mid)))

        _ = mid.handle(ev(.up, .left, 40 * ms))
        XCTAssertEqual(mid.state, .firstUp)
        XCTAssertEqual(mid.liveCandidate.firstUpNs, 40 * ms)
        XCTAssertFalse(mirrorContainsKeyStream(Mirror(reflecting: mid)))

        var fsm = DoubleTapFSM(config: enabledConfig())
        _ = run(
            validPair(first: .left, second: .right) + [ev(.cancel, nil, 900 * ms)],
            on: &fsm
        )
        fsm.reset()
        _ = run([ev(.down, .left, 0), ev(.cancel, nil, 5 * ms)], on: &fsm)

        XCTAssertFalse(mirrorContainsKeyStream(Mirror(reflecting: fsm)))
        let candidate = fsm.liveCandidate
        XCTAssertNil(candidate.firstSide)
        XCTAssertNil(candidate.firstDownNs)
        XCTAssertNil(candidate.firstUpNs)
        XCTAssertNil(candidate.secondSide)
        XCTAssertNil(candidate.secondDownNs)
    }

    func testFSM_TIME_INJECTED_identicalOutputsAfterPause() {
        let seq = validPair(first: .left, second: .right)
        var a = DoubleTapFSM(config: enabledConfig())
        let outA = seq.map { a.handle($0) }
        Thread.sleep(forTimeInterval: 0.05)
        var b = DoubleTapFSM(config: enabledConfig())
        let outB = seq.map { b.handle($0) }
        XCTAssertEqual(outA, outB)
        XCTAssertEqual(outA.count, 4)
        XCTAssertFalse(outA[0].triggered)
        XCTAssertFalse(outA[1].triggered)
        XCTAssertFalse(outA[2].triggered)
        XCTAssertTrue(outA[3].triggered)
        XCTAssertEqual(outA.filter(\.triggered).count, 1)
    }

    func testFSM_PROPERTY_ARBITRARY_triggerIffAcceptedPatternNeverStuck() {
        var rng = LCG(state: 0xC0FFEE_CAFE_F00D)
        let policies: [ModifierSidePolicy] = [.either, .same, .left, .right]

        for trial in 0..<240 {
            let config = enabledConfig(side: policies[trial % policies.count])
            let bucket = trial % 5
            let events: [DoubleTapEvent]
            switch bucket {
            case 0:
                events = constructedAcceptedPair(config: config, rng: &rng)
            case 1:
                events = mutatedInvalidPair(config: config, rng: &rng)
            case 2:
                events = generatedSoup(rng: &rng, timeMode: .equal)
            case 3:
                events = generatedSoup(rng: &rng, timeMode: .reversed)
            default:
                events = generatedSoup(rng: &rng, timeMode: trial % 2 == 0 ? .monotonic : .largeGap)
            }

            var fsm = DoubleTapFSM(config: config)
            let firstPass = events.map { fsm.handle($0) }
            let triggers = firstPass.filter(\.triggered).count

            if isExactAcceptedPattern(events, config: config) {
                XCTAssertEqual(triggers, 1, "trial \(trial) accepted pattern must trigger once")
            } else if bucket == 0 || bucket == 1 {
                XCTAssertEqual(triggers, 0, "trial \(trial) non-pattern must not trigger")
            }

            var replay = DoubleTapFSM(config: config)
            let secondPass = events.map { replay.handle($0) }
            XCTAssertEqual(firstPass, secondPass, "trial \(trial) must be deterministic")

            fsm.reset()
            XCTAssertEqual(fsm.state, .idle, "trial \(trial) reset → idle")
            XCTAssertNil(fsm.liveCandidate.firstSide)
            XCTAssertNil(fsm.liveCandidate.firstDownNs)
            XCTAssertNil(fsm.liveCandidate.firstUpNs)
            XCTAssertNil(fsm.liveCandidate.secondSide)
            XCTAssertNil(fsm.liveCandidate.secondDownNs)
        }
    }

    // MARK: - Helpers

    private func enabledConfig(
        gapMs: Int = 250,
        maxHoldMs: Int = 400,
        refractoryMs: Int = 500,
        side: ModifierSidePolicy = .either,
        tapCount: Int = 2
    ) -> DoubleTapConfig {
        DoubleTapConfig(
            enabled: true,
            gapMs: gapMs,
            maxHoldMs: maxHoldMs,
            refractoryMs: refractoryMs,
            side: side,
            tapCount: tapCount
        )!
    }

    private func ev(_ kind: DoubleTapEventKind, _ side: ModifierSide?, _ timeNs: UInt64) -> DoubleTapEvent {
        DoubleTapEvent(kind: kind, side: side, timeNs: timeNs)
    }

    private func validPair(
        first: ModifierSide,
        second: ModifierSide,
        t0: UInt64 = 0
    ) -> [DoubleTapEvent] {
        [
            ev(.down, first, t0),
            ev(.up, first, t0 + 50 * ms),
            ev(.down, second, t0 + 100 * ms),
            ev(.up, second, t0 + 150 * ms),
        ]
    }

    @discardableResult
    private func run(_ events: [DoubleTapEvent], on fsm: inout DoubleTapFSM) -> Int {
        events.reduce(0) { count, event in
            count + (fsm.handle(event).triggered ? 1 : 0)
        }
    }

    private func isExactAcceptedPattern(_ events: [DoubleTapEvent], config: DoubleTapConfig) -> Bool {
        guard config.enabled, events.count == 4 else { return false }
        guard events[0].kind == .down, events[1].kind == .up,
              events[2].kind == .down, events[3].kind == .up
        else { return false }
        guard let s0 = events[0].side, let s1 = events[1].side,
              let s2 = events[2].side, let s3 = events[3].side
        else { return false }
        guard s0 == s1, s2 == s3 else { return false }
        let t = events.map(\.timeNs)
        guard t[1] >= t[0], t[2] >= t[1], t[3] >= t[2] else { return false }
        guard t[1] - t[0] <= config.maxHoldNs else { return false }
        guard t[2] - t[1] <= config.gapNs else { return false }
        guard t[3] - t[2] <= config.maxHoldNs else { return false }
        switch config.side {
        case .either:
            return true
        case .same:
            return s0 == s2
        case .left:
            return s0 == .left && s2 == .left
        case .right:
            return s0 == .right && s2 == .right
        }
    }

    private func allowedSides(_ policy: ModifierSidePolicy) -> [ModifierSide] {
        switch policy {
        case .either, .same:
            return [.left, .right]
        case .left:
            return [.left]
        case .right:
            return [.right]
        }
    }

    private func constructedAcceptedPair(config: DoubleTapConfig, rng: inout LCG) -> [DoubleTapEvent] {
        let firstChoices = allowedSides(config.side)
        let first = firstChoices[rng.int(in: 0...(firstChoices.count - 1))]
        let second: ModifierSide
        switch config.side {
        case .same:
            second = first
        case .either:
            second = rng.int(in: 0...1) == 0 ? .left : .right
        case .left:
            second = .left
        case .right:
            second = .right
        }
        let hold1 = UInt64(rng.int(in: 0...config.maxHoldMs)) * ms
        let gap = UInt64(rng.int(in: 0...config.gapMs)) * ms
        let hold2 = UInt64(rng.int(in: 0...config.maxHoldMs)) * ms
        let t0 = UInt64(rng.int(in: 0...5_000)) * ms
        return [
            ev(.down, first, t0),
            ev(.up, first, t0 + hold1),
            ev(.down, second, t0 + hold1 + gap),
            ev(.up, second, t0 + hold1 + gap + hold2),
        ]
    }

    private func mutatedInvalidPair(config: DoubleTapConfig, rng: inout LCG) -> [DoubleTapEvent] {
        var events = constructedAcceptedPair(config: config, rng: &rng)
        switch rng.int(in: 0...5) {
        case 0:
            events.insert(ev(.cancel, nil, events[1].timeNs &+ 1), at: 2)
        case 1:
            events.insert(ev(.reset, nil, events[1].timeNs &+ 1), at: 2)
        case 2:
            if events[0].timeNs == 0 {
                events.insert(ev(.cancel, nil, 1), at: 1)
            } else {
                events[1].timeNs = events[0].timeNs - 1
            }
        case 3:
            let other: ModifierSide = events[0].side == .left ? .right : .left
            events.insert(ev(.down, other, events[0].timeNs &+ 1), at: 1)
        case 4:
            events[1].timeNs = events[0].timeNs + config.maxHoldNs + 1
        default:
            events[2].timeNs = events[1].timeNs + config.gapNs + 1
        }
        return events
    }

    private enum TimeMode {
        case monotonic
        case equal
        case reversed
        case largeGap
    }

    private func generatedSoup(rng: inout LCG, timeMode: TimeMode) -> [DoubleTapEvent] {
        let count = rng.int(in: 1...10)
        let kinds: [DoubleTapEventKind] = [.down, .up, .cancel, .reset]
        let sides: [ModifierSide?] = [.left, .right, nil]
        var events: [DoubleTapEvent] = []
        let base: UInt64 = 1_000_000_000
        for i in 0..<count {
            let kind = kinds[rng.int(in: 0...(kinds.count - 1))]
            let side: ModifierSide?
            if kind == .down || kind == .up {
                side = sides[rng.int(in: 0...1)]
            } else {
                side = nil
            }
            let time: UInt64
            switch timeMode {
            case .monotonic:
                time = base + UInt64(i) * 10 * ms
            case .equal:
                time = base
            case .reversed:
                time = base + UInt64(20 - i) * 10 * ms
            case .largeGap:
                time = base + UInt64(i) * 2_000 * ms
            }
            events.append(ev(kind, side, time))
        }
        if rng.int(in: 0...3) == 0 {
            events.append(ev(.reset, nil, events.last.map { $0.timeNs &+ 1 } ?? base))
        }
        return events
    }

    private func mirrorContainsKeyStream(_ mirror: Mirror) -> Bool {
        for child in mirror.children {
            if let label = child.label?.lowercased() {
                if label.contains("keycode") || label.contains("character") || label.contains("stream") {
                    return true
                }
            }
            if child.value is [Int] || child.value is [UInt16] || child.value is [UInt32]
                || child.value is [Character] || child.value is [String]
                || child.value is Character || child.value is String
            {
                return true
            }
            let nested = Mirror(reflecting: child.value)
            if !nested.children.isEmpty, mirrorContainsKeyStream(nested) {
                return true
            }
        }
        return false
    }
}

private struct LCG {
    var state: UInt64

    mutating func next() -> UInt64 {
        state = state &* 6_364_136_223_846_793_005 &+ 1
        return state
    }

    mutating func int(in range: ClosedRange<Int>) -> Int {
        let span = UInt64(range.count)
        return range.lowerBound + Int(next() % span)
    }
}
