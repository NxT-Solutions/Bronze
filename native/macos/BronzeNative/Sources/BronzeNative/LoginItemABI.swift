import Darwin
import Foundation
import ServiceManagement

private let loginAgentLabel = "app.bronze.desktop.login"
private let launchctlPath = "/bin/launchctl"
private let allowedLoginExecutables: Set<String> = ["bronze-desktop", "Bronze"]

@_silgen_name("bronze_native_login_item_status")
public func bronze_native_login_item_status() -> UInt32 {
    bronzeOnAppKit {
        loginItemStatusCode()
    }
}

@_silgen_name("bronze_native_apply_login_item")
public func bronze_native_apply_login_item(_ enabled: UInt32) -> UInt32 {
    bronzeOnAppKit {
        applyLoginItem(enabled != 0)
    }
}

/// SMAppService.mainApp is only valid for a real .app (not cargo-test or tauri-dev binaries).
private func bundledMainApp() -> Bool {
    let url = Bundle.main.bundleURL
    guard url.pathExtension == "app",
          let id = Bundle.main.bundleIdentifier, !id.isEmpty
    else {
        return false
    }
    return true
}

private func loginItemStatusCode() -> UInt32 {
    scheduleLeftoverDebugLoginAgentRemoval()
    if bundledMainApp() {
        return statusCode(SMAppService.mainApp.status)
    }
    return BRONZE_STATUS_DEGRADED
}

private func statusCode(_ status: SMAppService.Status) -> UInt32 {
    switch status {
    case .enabled:
        return BRONZE_STATUS_OK
    case .notRegistered:
        return BRONZE_STATUS_NOT_FOUND
    case .requiresApproval:
        return BRONZE_STATUS_CANCELLED
    case .notFound:
        return BRONZE_STATUS_DEGRADED
    @unknown default:
        return BRONZE_STATUS_DEGRADED
    }
}

private func applyLoginItem(_ enabled: Bool) -> UInt32 {
    scheduleLeftoverDebugLoginAgentRemoval()
    if bundledMainApp() {
        return applyBundledLoginItem(enabled)
    }
    return BRONZE_STATUS_DEGRADED
}

private func applyBundledLoginItem(_ enabled: Bool) -> UInt32 {
    let service = SMAppService.mainApp
    do {
        if enabled {
            switch service.status {
            case .enabled:
                return BRONZE_STATUS_OK
            case .requiresApproval:
                return BRONZE_STATUS_CANCELLED
            case .notFound:
                return BRONZE_STATUS_DEGRADED
            case .notRegistered:
                try service.register()
                return statusCode(service.status)
            @unknown default:
                return BRONZE_STATUS_DEGRADED
            }
        }
        switch service.status {
        case .notRegistered:
            return BRONZE_STATUS_NOT_FOUND
        case .notFound:
            return BRONZE_STATUS_DEGRADED
        default:
            try service.unregister()
            return statusCode(service.status)
        }
    } catch {
        return BRONZE_STATUS_DEGRADED
    }
}

private func currentExecutableURL() -> URL? {
    if let url = Bundle.main.executableURL {
        return url.resolvingSymlinksInPath()
    }
    let raw = ProcessInfo.processInfo.arguments.first ?? ""
    guard !raw.isEmpty else {
        return nil
    }
    return URL(fileURLWithPath: raw).resolvingSymlinksInPath()
}

private func isOwnedLoginExecutable(_ url: URL) -> Bool {
    let resolved = url.resolvingSymlinksInPath()
    let path = resolved.path
    guard path.hasPrefix("/"),
          !path.contains("/../"),
          FileManager.default.isExecutableFile(atPath: path)
    else {
        return false
    }
    return allowedLoginExecutables.contains(resolved.lastPathComponent)
}

private func loginAgentPlistURL() -> URL? {
    guard let library = FileManager.default.urls(
        for: .libraryDirectory,
        in: .userDomainMask
    ).first else {
        return nil
    }
    return library
        .resolvingSymlinksInPath()
        .appendingPathComponent("LaunchAgents", isDirectory: true)
        .appendingPathComponent("\(loginAgentLabel).plist", isDirectory: false)
}

private func launchdDomain() -> String {
    "gui/\(getuid())"
}

private func runLaunchctl(_ arguments: [String]) -> Int32 {
    let proc = Process()
    proc.executableURL = URL(fileURLWithPath: launchctlPath)
    proc.arguments = arguments
    proc.standardOutput = FileHandle.nullDevice
    proc.standardError = FileHandle.nullDevice
    do {
        try proc.run()
        proc.waitUntilExit()
        return proc.terminationStatus
    } catch {
        return -1
    }
}

private func bootoutLoginAgent() {
    _ = runLaunchctl(["bootout", "\(launchdDomain())/\(loginAgentLabel)"])
}

private func removeLoginAgentIfThisProcessOwnsLogin() {
    guard let exe = currentExecutableURL(),
          isOwnedLoginExecutable(exe),
          let plistURL = loginAgentPlistURL()
    else {
        return
    }
    try? FileManager.default.removeItem(at: plistURL)
}

/// `waitUntilExit` runs the current run loop. On the main thread that re-enters WebKit IPC while LiveSession is still locked.
private func scheduleLeftoverDebugLoginAgentRemoval() {
    enum Once {
        static let token: Void = {
            DispatchQueue.global(qos: .utility).async {
                removeLeftoverDebugLoginAgent()
            }
        }()
    }
    _ = Once.token
}

private func removeLeftoverDebugLoginAgent() {
    bootoutLoginAgent()
    removeLoginAgentIfThisProcessOwnsLogin()
}
