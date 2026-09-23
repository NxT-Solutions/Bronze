import Foundation
import ServiceManagement

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
    guard bundledMainApp() else {
        return BRONZE_STATUS_DEGRADED
    }
    return statusCode(SMAppService.mainApp.status)
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
    guard bundledMainApp() else {
        return BRONZE_STATUS_DEGRADED
    }
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
