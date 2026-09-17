import AppKit

@_silgen_name("bronze_native_pasteboard_write")
public func bronze_native_pasteboard_write(
    _ plain: bronze_native_utf8_view,
    _ html: bronze_native_utf8_view
) -> UInt32 {
    if bronze_native_validate_utf8(plain) != BRONZE_STATUS_OK {
        return BRONZE_STATUS_INVALID_UTF8
    }
    if bronze_native_validate_utf8(html) != BRONZE_STATUS_OK {
        return BRONZE_STATUS_INVALID_UTF8
    }
    guard let plainText = bronzeUtf8String(plain), let htmlText = bronzeUtf8String(html) else {
        return BRONZE_STATUS_INVALID_UTF8
    }
    return bronzeOnAppKit {
        let board = NSPasteboard.general
        board.clearContents()
        let item = NSPasteboardItem()
        guard item.setString(plainText, forType: .string) else {
            return board.setString(plainText, forType: .string)
                ? BRONZE_STATUS_OK
                : BRONZE_STATUS_DEGRADED
        }
        if !htmlText.isEmpty, let htmlData = htmlText.data(using: .utf8) {
            item.setData(htmlData, forType: .html)
        }
        if board.writeObjects([item]) {
            return BRONZE_STATUS_OK
        }
        return board.setString(plainText, forType: .string)
            ? BRONZE_STATUS_OK
            : BRONZE_STATUS_DEGRADED
    }
}
