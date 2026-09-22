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

    func testNonEmptyBodyIsDegraded() {
        let text = "The migration timeout is the real bug in persist."
        text.withCString { ptr in
            let view = bronze_native_utf8_view(ptr: ptr, len: UInt64(text.utf8.count))
            var out = bronze_native_utf8_view(ptr: nil, len: 0)
            XCTAssertEqual(bronze_native_item_title(view, &out), BRONZE_STATUS_DEGRADED)
        }
    }

    func testTitleABISourceBansHostedWeights() {
        let url = URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .appendingPathComponent("Sources/BronzeNative/TitleABI.swift")
        let text = try! String(contentsOf: url, encoding: .utf8)
        for needle in [
            "PrivateCloudCompute",
            "URLSession",
            "openai",
            "llama",
            "gguf",
            "SystemLanguageModel",
            "NLEmbedding",
            "FoundationModels",
            "NaturalLanguage",
            "huggingface",
            "qwen",
            "cactus",
            "needle",
        ] {
            XCTAssertFalse(
                text.lowercased().contains(needle.lowercased()),
                "TitleABI mentions \(needle)"
            )
        }
    }
}
