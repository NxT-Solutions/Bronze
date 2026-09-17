import AppKit

@_silgen_name("bronze_native_deliver_user_notice")
public func bronze_native_deliver_user_notice(
    _ title: bronze_native_utf8_view,
    _ body: bronze_native_utf8_view
) -> UInt32 {
    if bronze_native_validate_utf8(title) != BRONZE_STATUS_OK {
        return BRONZE_STATUS_INVALID_UTF8
    }
    if bronze_native_validate_utf8(body) != BRONZE_STATUS_OK {
        return BRONZE_STATUS_INVALID_UTF8
    }
    guard let titleText = bronzeUtf8String(title), isSafeNoticeText(titleText, max: 80),
          let bodyText = bronzeUtf8String(body), isSafeNoticeText(bodyText, max: 200)
    else {
        return BRONZE_STATUS_DEGRADED
    }
    return bronzeOnAppKit {
        scheduleUserNotice(title: titleText, body: bodyText)
        announceNotice(bodyText)
        return BRONZE_STATUS_OK
    }
}

private func isSafeNoticeText(_ text: String, max: Int) -> Bool {
    let trimmed = text.trimmingCharacters(in: .whitespacesAndNewlines)
    return !trimmed.isEmpty
        && trimmed.count <= max
        && !trimmed.contains("/")
        && !trimmed.contains("\\")
        && !trimmed.contains("\0")
        && !trimmed.contains("..")
}

private func scheduleUserNotice(title: String, body: String) {
    let notice = NSUserNotification()
    notice.title = title
    notice.informativeText = body
    notice.soundName = nil
    NSUserNotificationCenter.default.deliver(notice)
}

private func announceNotice(_ body: String) {
    let app = NSApplication.shared
    NSAccessibility.post(
        element: app,
        notification: .announcementRequested,
        userInfo: [
            .announcement: body,
            .priority: NSAccessibilityPriorityLevel.medium.rawValue,
        ]
    )
}
