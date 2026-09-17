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
        let wrotePlain = board.setString(plainText, forType: .string)
        let wroteHtml = board.setString(htmlText, forType: .html)
        if wrotePlain && wroteHtml {
            return BRONZE_STATUS_OK
        }
        return BRONZE_STATUS_DEGRADED
    }
}
