import AppKit
import UserNotifications

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
        installNoticeDelegate()
        announceNotice(bodyText)
        switch readAuthorizationStatus() {
        case .authorized, .provisional:
            postUserNotice(title: titleText, body: bodyText)
            return BRONZE_STATUS_OK
        default:
            return BRONZE_STATUS_DEGRADED
        }
    }
}

@_silgen_name("bronze_native_notification_authorization_status")
public func bronze_native_notification_authorization_status() -> UInt32 {
    bronzeOnAppKit {
        installNoticeDelegate()
        return statusCode(readAuthorizationStatus())
    }
}

@_silgen_name("bronze_native_request_notification_authorization")
public func bronze_native_request_notification_authorization() -> UInt32 {
    bronzeOnAppKitModal {
        installNoticeDelegate()
        let current = readAuthorizationStatus()
        if current == .denied {
            return BRONZE_STATUS_CANCELLED
        }
        if current == .authorized || current == .provisional {
            return BRONZE_STATUS_OK
        }
        let box = AuthStatusBox()
        let lock = DispatchSemaphore(value: 0)
        UNUserNotificationCenter.current().requestAuthorization(options: [.alert]) { granted, _ in
            box.value = granted ? .authorized : .denied
            lock.signal()
        }
        lock.wait()
        return statusCode(box.value)
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

private func installNoticeDelegate() {
    UNUserNotificationCenter.current().delegate = BronzeNoticeCenter.shared
}

private func readAuthorizationStatus() -> UNAuthorizationStatus {
    let box = AuthStatusBox()
    let lock = DispatchSemaphore(value: 0)
    UNUserNotificationCenter.current().getNotificationSettings { settings in
        box.value = settings.authorizationStatus
        lock.signal()
    }
    _ = lock.wait(timeout: .now() + 2)
    return box.value
}

private func statusCode(_ status: UNAuthorizationStatus) -> UInt32 {
    switch status {
    case .authorized, .provisional:
        return BRONZE_STATUS_OK
    case .denied:
        return BRONZE_STATUS_CANCELLED
    case .notDetermined:
        return BRONZE_STATUS_NOT_FOUND
    @unknown default:
        return BRONZE_STATUS_DEGRADED
    }
}

private func postUserNotice(title: String, body: String) {
    let content = UNMutableNotificationContent()
    content.title = title
    content.body = body
    content.sound = nil
    let request = UNNotificationRequest(
        identifier: "bronze.capture.\(UUID().uuidString)",
        content: content,
        trigger: nil
    )
    UNUserNotificationCenter.current().add(request, withCompletionHandler: nil)
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

private final class AuthStatusBox: @unchecked Sendable {
    var value: UNAuthorizationStatus = .notDetermined
}

private final class BronzeNoticeCenter: NSObject, UNUserNotificationCenterDelegate, @unchecked Sendable {
    nonisolated(unsafe) static let shared = BronzeNoticeCenter()

    func userNotificationCenter(
        _ center: UNUserNotificationCenter,
        willPresent notification: UNNotification,
        withCompletionHandler completionHandler: @escaping (UNNotificationPresentationOptions) -> Void
    ) {
        completionHandler([.banner, .list])
    }
}
