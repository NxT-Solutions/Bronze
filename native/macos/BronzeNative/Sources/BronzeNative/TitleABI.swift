import Foundation

@_silgen_name("bronze_native_item_title")
public func bronze_native_item_title(
    _ body: bronze_native_utf8_view,
    _ out: UnsafeMutablePointer<bronze_native_utf8_view>?
) -> UInt32 {
    let status = bronze_native_validate_utf8(body)
    if status != BRONZE_STATUS_OK {
        return status
    }
    guard out != nil else {
        return BRONZE_STATUS_NOT_FOUND
    }
    guard utf8Body(body) != nil else {
        return BRONZE_STATUS_INVALID_UTF8
    }
    return BRONZE_STATUS_DEGRADED
}

private func utf8Body(_ view: bronze_native_utf8_view) -> String? {
    if view.ptr == nil {
        return view.len == 0 ? "" : nil
    }
    if view.len > UInt64(Int.max) {
        return nil
    }
    let count = Int(view.len)
    let buffer = UnsafeBufferPointer(start: view.ptr, count: count)
    let bytes = Array(buffer)
    let decoded = String(decoding: bytes, as: UTF8.self)
    let reencoded = Array(decoded.utf8)
    guard reencoded == bytes else {
        return nil
    }
    return decoded
}
