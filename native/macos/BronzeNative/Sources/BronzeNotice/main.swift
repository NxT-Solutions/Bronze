import AppKit
import UserNotifications

@main
enum BronzeNoticeMain {
    static func main() {
        let args = CommandLine.arguments
        guard args.count >= 3 else {
            return
        }
        let title = args[1]
        let body = args[2]
        guard isSafeNoticeText(title, max: 80), isSafeNoticeText(body, max: 200) else {
            return
        }
        let app = NSApplication.shared
        app.setActivationPolicy(.accessory)
        let delegate = BronzeNoticeDelegate(title: title, body: body)
        BronzeNoticeDelegate.running = delegate
        app.delegate = delegate
        UNUserNotificationCenter.current().delegate = delegate
        app.run()
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

    init(title: String, body: String) {
        self.title = title
        self.body = body
    }

    func applicationDidFinishLaunching(_: Notification) {
        DispatchQueue.main.asyncAfter(deadline: .now() + 25) {
            self.quit()
        }
        let center = UNUserNotificationCenter.current()
        center.delegate = self
        center.getNotificationSettings { settings in
            switch settings.authorizationStatus {
            case .authorized, .provisional:
                self.post(center)
            case .notDetermined:
                center.requestAuthorization(options: [.alert]) { granted, _ in
                    if granted {
                        self.post(center)
                    } else {
                        self.quit()
                    }
                }
            default:
                self.quit()
            }
        }
    }

    func userNotificationCenter(
        _ center: UNUserNotificationCenter,
        willPresent notification: UNNotification,
        withCompletionHandler completionHandler: @escaping (UNNotificationPresentationOptions) -> Void
    ) {
        completionHandler([.banner, .list])
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.5) {
            self.quit()
        }
    }

    private func post(_ center: UNUserNotificationCenter) {
        let content = UNMutableNotificationContent()
        content.title = title
        content.body = body
        content.sound = nil
        let request = UNNotificationRequest(
            identifier: "bronze.capture.\(UUID().uuidString)",
            content: content,
            trigger: nil
        )
        center.add(request) { error in
            if error != nil {
                self.quit()
            }
        }
    }

    private func quit() {
        DispatchQueue.main.async {
            NSApp.terminate(nil)
        }
    }
}
