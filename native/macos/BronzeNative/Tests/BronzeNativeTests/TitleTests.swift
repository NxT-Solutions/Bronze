import XCTest
@testable import BronzeNative

final class TitleTests: XCTestCase {
    func testItemTitleInvalidUTF8() {
        let bad: [UInt8] = [0xFF]
        bad.withUnsafeBufferPointer { buf in
            let view = bronze_native_utf8_view(ptr: buf.baseAddress, len: 1)
            var out = bronze_native_utf8_view(ptr: nil, len: 0)
            XCTAssertEqual(bronze_native_item_title(view, &out), BRONZE_STATUS_INVALID_UTF8)
        }
    }

    func testEmptyBodyIsDegraded() {
        let view = bronze_native_utf8_view(ptr: nil, len: 0)
        var out = bronze_native_utf8_view(ptr: nil, len: 0)
        XCTAssertEqual(bronze_native_item_title(view, &out), BRONZE_STATUS_DEGRADED)
    }

    func testCentroidPicksNearestNotFirst() {
        let sentences = ["alpha", "beta", "gamma"]
        let vectors = [
            [1.0, 0.0],
            [0.0, 1.0],
            [0.05, 0.95],
        ]
        XCTAssertEqual(nearestToCentroid(sentences: sentences, vectors: vectors), "gamma")
    }

    func testClampTitleFitsCardLineWithoutEllipsis() {
        let long = String(repeating: "a", count: 80)
        let clamped = clampTitle(long)
        XCTAssertEqual(clamped.count, 40)
        XCTAssertFalse(clamped.contains("…"))
    }

    func testClampTitlePrefersWordBoundary() {
        let long = "Bold heading that keeps going and going and going past the limit"
        let clamped = clampTitle(long)
        XCTAssertLessThanOrEqual(clamped.count, 40)
        XCTAssertFalse(clamped.contains("…"))
        XCTAssertFalse(clamped.hasSuffix(" "))
    }

    func testTitleABISourceBansHostedWeights() {
        let url = URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .appendingPathComponent("Sources/BronzeNative/TitleABI.swift")
        let text = try! String(contentsOf: url, encoding: .utf8)
        for needle in ["PrivateCloudCompute", "URLSession", "openai", "llama", "gguf"] {
            XCTAssertFalse(
                text.lowercased().contains(needle.lowercased()),
                "TitleABI mentions \(needle)"
            )
        }
    }
}
