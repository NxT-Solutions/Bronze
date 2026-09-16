// EventTap* suite for story 3.3 (CAP-002, CAP-004, ADR-005).
// Fake/inject only — no TCC prompt, no physical 1k trials.

import Carbon.HIToolbox
import CoreGraphics
import XCTest
@testable import BronzeNative

private final class ForeignWrite: @unchecked Sendable {
    var value = true
}

final class EventTapTests: XCTestCase {
    override func tearDown() {
        EventTapRuntime.shared.stop()
        super.tearDown()
    }

    func testEventTapCallbackAlwaysReturnsIncomingEvent() {
        let engine = EventTapEngine()
        let event = CGEvent(keyboardEventSource: nil, virtualKey: 0, keyDown: true)!
        let out = engine.passThrough(event)
        XCTAssertTrue(out === event, "listenOnly callback must never suppress or replace the event")
    }

    func testEventTapSPSCIsSingleProducer() {
        let engine = EventTapEngine()
        engine.attachTestProducer()
        XCTAssertTrue(engine.enqueueFromCaller(.reset))
        let box = ForeignWrite()
        let foreign = expectation(description: "foreign enqueue")
        Thread.detachNewThread {
            box.value = engine.enqueueFromCaller(.trigger)
            foreign.fulfill()
        }
        wait(for: [foreign], timeout: 2)
        XCTAssertFalse(box.value, "SPSC must reject a second producer")
        XCTAssertEqual(engine.spsc.drain()?.kind, .reset)
        XCTAssertNil(engine.spsc.drain())
    }

    func testEventTapDisableResetsFSM() {
        let engine = EventTapEngine()
        engine.attachTestProducer()
        engine.setGestureEnabled(true)
        XCTAssertTrue(engine.feedTest(kind: 1, carbonKey: Int32(kVK_Shift), timeNs: 1_000))
        XCTAssertEqual(engine.fsm.state, .firstDown)
        XCTAssertTrue(engine.simulateTapDisabled())
        XCTAssertEqual(engine.fsm.state, .idle)
        XCTAssertEqual(engine.spsc.drain()?.kind, .disabled)
    }

    func testEventTapDeniedDegradesWithoutPromptOrSuppression() {
        let engine = EventTapEngine()
        let health = engine.startLive()
        XCTAssertTrue(health == .listening || health == .degraded)
        let event = CGEvent(keyboardEventSource: nil, virtualKey: 0, keyDown: true)!
        XCTAssertTrue(engine.passThrough(event) === event)
        engine.stop()
        XCTAssertEqual(engine.health, .idle)
    }

    func testEventTapTriggerEnqueuesOnce() {
        let engine = EventTapEngine()
        engine.attachTestProducer()
        engine.setGestureEnabled(true)
        let gap: UInt64 = 250_000_000
        XCTAssertTrue(engine.feedTest(kind: 1, carbonKey: Int32(kVK_Shift), timeNs: 1_000))
        XCTAssertTrue(engine.feedTest(kind: 2, carbonKey: Int32(kVK_Shift), timeNs: 2_000))
        XCTAssertTrue(engine.feedTest(kind: 1, carbonKey: Int32(kVK_Shift), timeNs: 2_000 + gap))
        XCTAssertTrue(engine.feedTest(kind: 2, carbonKey: Int32(kVK_Shift), timeNs: 2_000 + gap + 1_000))
        XCTAssertEqual(engine.spsc.drain()?.kind, .trigger)
        XCTAssertNil(engine.spsc.drain())
    }
}
