import AppKit
import UserNotifications

@main
enum BronzeNoticeMain {
    static func main() {
        let texts = noticePayload(CommandLine.arguments)
        let app = NSApplication.shared
        app.setActivationPolicy(.accessory)
        let delegate = BronzeNoticeDelegate(
            title: texts?.0 ?? "",
            body: texts?.1 ?? "",
            itemId: texts.flatMap { payload in payload.2 }
        )
        BronzeNoticeDelegate.running = delegate
        app.delegate = delegate
        UNUserNotificationCenter.current().delegate = delegate
        app.run()
    }
}

/// Launch Services may inject `-psn_*` before the title/body pair.
private func noticePayload(_ args: [String]) -> (String, String, String?)? {
    let texts = Array(args.dropFirst().filter { arg in
        !arg.hasPrefix("-psn_") && !arg.hasPrefix("-NS")
    })
    guard texts.count >= 2 else {
        return nil
    }
    let title = texts[0]
    let body = texts[1]
    guard isSafeNoticeText(title, max: 80), isSafeNoticeText(body, max: 200) else {
        return nil
    }
    let itemId = texts.count >= 3 && isSafeItemId(texts[2]) ? texts[2] : nil
    return (title, body, itemId)
}

private func isSafeItemId(_ text: String) -> Bool {
    guard !text.isEmpty, text.count <= 80 else {
        return false
    }
    return text.unicodeScalars.allSatisfy { scalar in
        let value = scalar.value
        return (value >= 48 && value <= 57)
            || (value >= 65 && value <= 90)
            || (value >= 97 && value <= 122)
            || value == 45
            || value == 95
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

private final class BronzeNoticeDelegate: NSObject, NSApplicationDelegate, UNUserNotificationCenterDelegate, @unchecked Sendable {
    nonisolated(unsafe) static var running: BronzeNoticeDelegate?
    private let title: String
    private let body: String
    private let itemId: String?

    init(title: String, body: String, itemId: String?) {
        self.title = title
        self.body = body
        self.itemId = itemId
    }

    func applicationWillFinishLaunching(_: Notification) {
        UNUserNotificationCenter.current().delegate = self
    }

    func applicationDidFinishLaunching(_: Notification) {
        let center = UNUserNotificationCenter.current()
        center.delegate = self
        guard !title.isEmpty, !body.isEmpty else {
            DispatchQueue.main.asyncAfter(deadline: .now() + 2) {
                self.quit()
            }
            return
        }
        DispatchQueue.main.asyncAfter(deadline: .now() + 8) {
            self.quit()
        }
        center.getNotificationSettings { settings in
            let status = settings.authorizationStatus
            DispatchQueue.main.async {
                self.handleSettings(status)
            }
        }
    }

    @MainActor
    private func handleSettings(_ status: UNAuthorizationStatus) {
        let center = UNUserNotificationCenter.current()
        switch status {
        case .authorized, .provisional:
            post(center)
        case .notDetermined:
            presentAsForeground()
            center.requestAuthorization(options: [.alert]) { granted, _ in
                DispatchQueue.main.async {
                    if granted {
                        self.post(UNUserNotificationCenter.current())
                    } else {
                        self.openNotificationSettings()
                        self.quit()
                    }
                }
            }
        default:
            openNotificationSettings()
            quit()
        }
    }

    /// Accessory LSUIElement launches can add a request that usernoted never presents.
    @MainActor
    private func presentAsForeground() {
        NSApp.setActivationPolicy(.regular)
        NSApp.activate(ignoringOtherApps: true)
    }

    nonisolated func userNotificationCenter(
        _ center: UNUserNotificationCenter,
        willPresent notification: UNNotification,
        withCompletionHandler completionHandler: @escaping (UNNotificationPresentationOptions) -> Void
    ) {
        completionHandler([.banner, .list])
        DispatchQueue.main.asyncAfter(deadline: .now() + 2) {
            self.quit()
        }
    }

    nonisolated func userNotificationCenter(
        _ center: UNUserNotificationCenter,
        didReceive response: UNNotificationResponse,
        withCompletionHandler completionHandler: @escaping () -> Void
    ) {
        if let raw = response.notification.request.content.userInfo["bronzeItemId"] as? String,
           isSafeItemId(raw) {
            DistributedNotificationCenter.default().postNotificationName(
                Notification.Name("app.bronze.desktop.notice-activate"),
                object: raw,
                userInfo: nil,
                deliverImmediately: true
            )
        }
        completionHandler()
        DispatchQueue.main.async {
            self.quit()
        }
    }

    @MainActor
    private func post(_ center: UNUserNotificationCenter) {
        let content = UNMutableNotificationContent()
        content.title = title
        content.body = body
        content.sound = nil
        content.interruptionLevel = .active
        if let itemId, isSafeItemId(itemId) {
            content.userInfo = ["bronzeItemId": itemId]
        }
        let request = UNNotificationRequest(
            identifier: "bronze.capture.\(UUID().uuidString)",
            content: content,
            trigger: UNTimeIntervalNotificationTrigger(timeInterval: 0.1, repeats: false)
        )
        center.add(request) { error in
            if error != nil {
                self.quit()
                return
            }
            DispatchQueue.main.asyncAfter(deadline: .now() + 4) {
                self.quit()
            }
        }
    }

    /// Denied cannot show another sheet; Notifications Settings is the enable path.
    @MainActor
    private func openNotificationSettings() {
        guard let url = URL(string: "x-apple.systempreferences:com.apple.Notifications-Settings.extension") else {
            return
        }
        NSWorkspace.shared.open(url)
    }

    private func quit() {
        DispatchQueue.main.async {
            NSApp.terminate(nil)
        }
    }
}
