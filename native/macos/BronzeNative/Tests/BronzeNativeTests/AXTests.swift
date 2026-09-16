import XCTest
@testable import BronzeNative

final class AXTests: XCTestCase {
    func testSecureAndUnknownFailClosedWithZeroContent() {
        let secret = AXFakeNode(
            role: .textField,
            subrole: .secureTextField,
            selection: .text("hunter2-secret")
        )
        let (outcome, text) = axCapture(AXFakeTree(focused: secret))
        XCTAssertEqual(outcome, .protectedContent)
        XCTAssertNil(text)
        XCTAssertEqual(secret.queryCount, 0)

        let web = AXFakeNode(role: .webArea, selection: .text("do-not-read"))
        let (unknown, unknownText) = axCapture(AXFakeTree(focused: web))
        XCTAssertEqual(unknown, .protectionUnknown)
        XCTAssertNil(unknownText)
        XCTAssertEqual(web.queryCount, 0)
    }

    func testExclusionRunsBeforeContentQuery() {
        let node = AXFakeNode(role: .textArea, selection: .text("excluded-body"))
        let (outcome, text) = axCapture(AXFakeTree(focused: node, excluded: true))
        XCTAssertEqual(outcome, .appExcluded)
        XCTAssertNil(text)
        XCTAssertEqual(node.queryCount, 0)
    }

    func testWhitespacePreservedAndEmptyIsNoSelectionWithoutTrim() {
        let raw = "  hello\r\n\t "
        let node = AXFakeNode(role: .textArea, selection: .text(raw))
        let (outcome, text) = axCapture(AXFakeTree(focused: node))
        XCTAssertEqual(outcome, .captured(length: raw.utf8.count))
        XCTAssertEqual(text, raw)

        let empty = AXFakeNode(role: .textField, selection: .empty)
        let (emptyOut, emptyText) = axCapture(AXFakeTree(focused: empty))
        XCTAssertEqual(emptyOut, .noSelection)
        XCTAssertNil(emptyText)

        let spaces = AXFakeNode(role: .textField, selection: .text("   "))
        let (spaceOut, spaceText) = axCapture(AXFakeTree(focused: spaces))
        XCTAssertEqual(spaceOut, .captured(length: 3))
        XCTAssertEqual(spaceText, "   ")
    }
}
