import Foundation
import ServiceManagement

private let loginAgentLabel = "app.bronze.desktop.login"
private let loginAgentBundleId = "app.bronze.desktop"
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
    if bundledMainApp() {
        return statusCode(SMAppService.mainApp.status)
    }
    return launchAgentIsRegistered() ? BRONZE_STATUS_OK : BRONZE_STATUS_NOT_FOUND
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
    if bundledMainApp() {
        removeLoginAgentIfThisProcessOwnsLogin()
        return applyBundledLoginItem(enabled)
    }
    guard let exe = currentExecutableURL(), isOwnedLoginExecutable(exe) else {
        return BRONZE_STATUS_DEGRADED
    }
    if enabled {
        return writeLoginAgent()
    }
    removeLoginAgentIfThisProcessOwnsLogin()
    return BRONZE_STATUS_NOT_FOUND
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

private func launchAgentIsRegistered() -> Bool {
    guard let plistURL = loginAgentPlistURL(),
          let data = try? Data(contentsOf: plistURL),
          let obj = try? PropertyListSerialization.propertyList(
            from: data,
            options: [],
            format: nil
          ) as? [String: Any],
          obj["Label"] as? String == loginAgentLabel,
          let args = obj["ProgramArguments"] as? [String],
          let first = args.first
    else {
        return false
    }
    guard isOwnedLoginExecutable(URL(fileURLWithPath: first)),
          let exe = currentExecutableURL(),
          isOwnedLoginExecutable(exe)
    else {
        return false
    }
    return URL(fileURLWithPath: first).resolvingSymlinksInPath().path == exe.path
}

private func writeLoginAgent() -> UInt32 {
    guard let exe = currentExecutableURL(),
          isOwnedLoginExecutable(exe),
          let plistURL = loginAgentPlistURL()
    else {
        return BRONZE_STATUS_DEGRADED
    }
    let dir = plistURL.deletingLastPathComponent()
    do {
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        let payload: [String: Any] = [
            "Label": loginAgentLabel,
            "ProgramArguments": [exe.path],
            "RunAtLoad": true,
            "LimitLoadToSessionType": "Aqua",
            "ProcessType": "Interactive",
            "AssociatedBundleIdentifiers": [loginAgentBundleId],
        ]
        let data = try PropertyListSerialization.data(
            fromPropertyList: payload,
            format: .xml,
            options: 0
        )
        try data.write(to: plistURL, options: .atomic)
        return BRONZE_STATUS_OK
    } catch {
        return BRONZE_STATUS_DEGRADED
    }
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
