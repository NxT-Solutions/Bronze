import XCTest
@testable import BronzeNative

final class IngressTests: XCTestCase {
    func testIngressSnapshotHasNoFocusOrContentFields() {
        let snap = BronzeIngressSnapshot.empty
        let mirror = Mirror(reflecting: snap)
        let names = mirror.children.compactMap(\.label)
        for forbidden in ["focused", "title", "url", "axElement", "windowTitle"] {
            XCTAssertFalse(names.contains(where: { $0.localizedCaseInsensitiveContains(forbidden) }))
        }
        XCTAssertTrue(names.contains("targetPid"))
        XCTAssertTrue(names.contains("bundleToken"))
        XCTAssertTrue(names.contains("activationGeneration"))
        XCTAssertTrue(names.contains("destinationUuidHi"))
        XCTAssertTrue(names.contains("acceptCaptureGeneration"))
        XCTAssertTrue(names.contains("route"))
        XCTAssertTrue(names.contains("monotonicTimeNs"))
    }

    func testIngressInconsistentReadIsUnavailable() {
        let lock = IngressSeqlock()
        lock.publish(.empty)
        XCTAssertNotNil(lock.load())
        lock.testBeginInconsistentWrite()
        XCTAssertNil(lock.load())
        lock.testEndInconsistentWrite()
        XCTAssertNotNil(lock.load())
    }
}
