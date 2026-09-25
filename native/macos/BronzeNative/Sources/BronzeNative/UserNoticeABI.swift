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
        announceNotice(bodyText)
        if let center = bundledNotificationCenter() {
            switch readAuthorizationStatus(center) {
            case .authorized, .provisional:
                postUserNotice(center, title: titleText, body: bodyText)
                return BRONZE_STATUS_OK
            default:
                return BRONZE_STATUS_DEGRADED
            }
        }
        postLegacyNotice(title: titleText, body: bodyText)
        return BRONZE_STATUS_OK
    }
}

@_silgen_name("bronze_native_notification_authorization_status")
public func bronze_native_notification_authorization_status() -> UInt32 {
    bronzeOnAppKit {
        guard let center = bundledNotificationCenter() else {
            return BRONZE_STATUS_DEGRADED
        }
        return statusCode(readAuthorizationStatus(center))
    }
}

@_silgen_name("bronze_native_request_notification_authorization")
public func bronze_native_request_notification_authorization() -> UInt32 {
    bronzeOnAppKitModal {
        guard let center = bundledNotificationCenter() else {
            return BRONZE_STATUS_DEGRADED
        }
        let current = readAuthorizationStatus(center)
        if current == .denied {
            return BRONZE_STATUS_CANCELLED
        }
        if current == .authorized || current == .provisional {
            return BRONZE_STATUS_OK
        }
        let box = AuthStatusBox()
        let lock = DispatchSemaphore(value: 0)
        center.requestAuthorization(options: [.alert]) { granted, _ in
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

/// UNUserNotificationCenter.current() aborts when the process is not a .app
/// (tauri dev / cargo-run use target/debug/bronze-desktop).
@MainActor
private func bundledNotificationCenter() -> UNUserNotificationCenter? {
    let url = Bundle.main.bundleURL
    guard url.pathExtension == "app",
          let id = Bundle.main.bundleIdentifier, !id.isEmpty
    else {
        return nil
    }
    let center = UNUserNotificationCenter.current()
    center.delegate = BronzeNoticeCenter.shared
    return center
}

private func readAuthorizationStatus(_ center: UNUserNotificationCenter) -> UNAuthorizationStatus {
    let box = AuthStatusBox()
    let lock = DispatchSemaphore(value: 0)
    center.getNotificationSettings { settings in
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

private func postUserNotice(
    _ center: UNUserNotificationCenter,
    title: String,
    body: String
) {
    let content = UNMutableNotificationContent()
    content.title = title
    content.body = body
    content.sound = nil
    content.interruptionLevel = .active
    let request = UNNotificationRequest(
        identifier: "bronze.capture.\(UUID().uuidString)",
        content: content,
        trigger: UNTimeIntervalNotificationTrigger(timeInterval: 0.1, repeats: false)
    )
    center.add(request, withCompletionHandler: nil)
}

/// Unbundled cargo-run cannot call UNUserNotificationCenter in-process.
/// A sibling BronzeNotice.app (same logo, UN banners) posts instead so
/// Accessibility / Input Monitoring on this binary stay put.
private func postLegacyNotice(title: String, body: String) {
    guard let app = noticeHelperApp() else {
        return
    }
    // Launch Services (`open -g`) makes the helper responsible for its own
    // UN identity. NSWorkspace.openApplication keeps this binary as parent
    // and usernoted then drops the banner.
    let process = Process()
    process.executableURL = URL(fileURLWithPath: "/usr/bin/open")
    process.arguments = ["-n", "-g", app.path, "--args", title, body]
    try? process.run()
}

private func noticeHelperApp() -> URL? {
    if let support = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first {
        try? FileManager.default.removeItem(
            at: support.appendingPathComponent("Bronze/BronzeNotice.app")
        )
    }
    let exe = Bundle.main.executableURL ?? URL(fileURLWithPath: CommandLine.arguments[0])
    let app = exe.deletingLastPathComponent().appendingPathComponent("BronzeNotice.app")
    let helper = app.appendingPathComponent("Contents/MacOS/BronzeNotice")
    guard FileManager.default.isExecutableFile(atPath: helper.path) else {
        return nil
    }
    return app
}

@MainActor
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
