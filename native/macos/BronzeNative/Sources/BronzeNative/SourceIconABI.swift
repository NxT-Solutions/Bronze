import AppKit

private let iconPngCap = 16 * 1024
private let iconPointSize = 16
private let iconPixels = 32

@_silgen_name("bronze_native_bundle_id_for_pid")
public func bronze_native_bundle_id_for_pid(
    _ pid: Int32,
    _ out: UnsafeMutablePointer<bronze_native_utf8_view>?
) -> UInt32 {
    guard let out else {
        return BRONZE_STATUS_NOT_FOUND
    }
    guard pid > 0 else {
        return BRONZE_STATUS_DEGRADED
    }
    let outBox = BronzeOutViewBox(out)
    return bronzeOnAppKit {
        let running = NSRunningApplication(processIdentifier: pid)
            ?? NSWorkspace.shared.runningApplications.first(where: { $0.processIdentifier == pid })
        guard let bundle = running?.bundleIdentifier, !bundle.isEmpty else {
            return BRONZE_STATUS_DEGRADED
        }
        let bytes = Array(bundle.utf8)
        return bytes.withUnsafeBufferPointer { buf in
            let src = bronze_native_utf8_view(ptr: buf.baseAddress, len: UInt64(bytes.count))
            return bronze_native_utf8_owned_copy(src, outBox.ptr)
        }
    }
}

@_silgen_name("bronze_native_app_icon_png")
public func bronze_native_app_icon_png(
    _ bundle_or_name: bronze_native_utf8_view,
    _ out: UnsafeMutablePointer<bronze_native_utf8_view>?
) -> UInt32 {
    if bronze_native_validate_utf8(bundle_or_name) != BRONZE_STATUS_OK {
        return BRONZE_STATUS_INVALID_UTF8
    }
    guard let out else {
        return BRONZE_STATUS_NOT_FOUND
    }
    guard let key = bronzeUtf8String(bundle_or_name), !key.isEmpty else {
        return BRONZE_STATUS_DEGRADED
    }
    let outBox = BronzeOutViewBox(out)
    return bronzeOnAppKit {
        guard let png = pngIcon(for: key), !png.isEmpty, png.count <= iconPngCap else {
            return BRONZE_STATUS_DEGRADED
        }
        return png.withUnsafeBytes { raw in
            let ptr = raw.bindMemory(to: UInt8.self).baseAddress
            let src = bronze_native_utf8_view(ptr: ptr, len: UInt64(png.count))
            return bronzeOwnedBytesCopy(src, outBox.ptr)
        }
    }
}

@MainActor
private func pngIcon(for key: String) -> Data? {
    guard let url = officialAppURL(for: key) else {
        return nil
    }
    return rasterPng(NSWorkspace.shared.icon(forFile: url.path))
}

@MainActor
private func officialAppURL(for key: String) -> URL? {
    let workspace = NSWorkspace.shared
    if let url = workspace.urlForApplication(withBundleIdentifier: key) {
        return url
    }
    if let match = workspace.runningApplications.first(where: { runningApp($0, matches: key) }) {
        return match.bundleURL
    }
    return installedAppURL(matching: key)
}

private func runningApp(_ app: NSRunningApplication, matches key: String) -> Bool {
    if let bundle = app.bundleIdentifier, bundle.caseInsensitiveCompare(key) == .orderedSame {
        return true
    }
    if let name = app.localizedName, name.caseInsensitiveCompare(key) == .orderedSame {
        return true
    }
    if let stem = app.bundleURL?.deletingPathExtension().lastPathComponent,
       stem.caseInsensitiveCompare(key) == .orderedSame
    {
        return true
    }
    return false
}

private func installedAppURL(matching key: String) -> URL? {
    let fm = FileManager.default
    let roots = [
        "/Applications",
        "/System/Applications",
        "/System/Applications/Utilities",
        fm.homeDirectoryForCurrentUser.appendingPathComponent("Applications").path,
    ]
    for root in roots {
        let direct = URL(fileURLWithPath: root, isDirectory: true)
            .appendingPathComponent("\(key).app")
        if fm.fileExists(atPath: direct.path) {
            return direct
        }
    }
    for root in roots {
        let dir = URL(fileURLWithPath: root, isDirectory: true)
        guard let children = try? fm.contentsOfDirectory(
            at: dir,
            includingPropertiesForKeys: [.isDirectoryKey],
            options: [.skipsHiddenFiles]
        ) else {
            continue
        }
        for child in children where child.pathExtension.caseInsensitiveCompare("app") == .orderedSame {
            if child.deletingPathExtension().lastPathComponent.caseInsensitiveCompare(key) == .orderedSame {
                return child
            }
        }
    }
    return nil
}

private func rasterPng(_ image: NSImage) -> Data? {
    if let png = rasterPng(image, pixels: iconPixels), png.count <= iconPngCap {
        return png
    }
    return rasterPng(image, pixels: iconPointSize)
}

private let listPayloadCap = 512 * 1024

@_silgen_name("bronze_native_list_installed_apps")
public func bronze_native_list_installed_apps(
    _ out: UnsafeMutablePointer<bronze_native_utf8_view>?
) -> UInt32 {
    guard let out else {
        return BRONZE_STATUS_NOT_FOUND
    }
    let outBox = BronzeOutViewBox(out)
    return bronzeOnAppKit {
        let payload = installedAppListPayload()
        let bytes = Array(payload.utf8)
        guard bytes.count <= listPayloadCap else {
            return BRONZE_STATUS_DEGRADED
        }
        return bytes.withUnsafeBufferPointer { buf in
            let src = bronze_native_utf8_view(ptr: buf.baseAddress, len: UInt64(bytes.count))
            return bronze_native_utf8_owned_copy(src, outBox.ptr)
        }
    }
}

private func installedAppListPayload() -> String {
    var seen = Set<String>()
    var rows: [(String, String)] = []
    let fm = FileManager.default
    let roots = [
        "/Applications",
        "/System/Applications",
        "/System/Applications/Utilities",
        fm.homeDirectoryForCurrentUser.appendingPathComponent("Applications").path,
    ]
    for root in roots {
        let dir = URL(fileURLWithPath: root, isDirectory: true)
        guard let children = try? fm.contentsOfDirectory(
            at: dir,
            includingPropertiesForKeys: [.isDirectoryKey],
            options: [.skipsHiddenFiles]
        ) else {
            continue
        }
        for child in children where child.pathExtension.caseInsensitiveCompare("app") == .orderedSame {
            guard let row = installedAppRow(from: child), seen.insert(row.0).inserted else {
                continue
            }
            rows.append(row)
        }
    }
    rows.sort { lhs, rhs in
        lhs.1.localizedCaseInsensitiveCompare(rhs.1) == .orderedAscending
    }
    return rows.map { "\($0.0)\t\($0.1)" }.joined(separator: "\n")
}

@_silgen_name("bronze_native_pick_installed_app")
public func bronze_native_pick_installed_app(
    _ out: UnsafeMutablePointer<bronze_native_utf8_view>?
) -> UInt32 {
    guard let out else {
        return BRONZE_STATUS_NOT_FOUND
    }
    let outBox = BronzeOutViewBox(out)
    return bronzeOnAppKitModal {
        let panel = NSOpenPanel()
        panel.canChooseFiles = true
        panel.canChooseDirectories = false
        panel.allowsMultipleSelection = false
        panel.treatsFilePackagesAsDirectories = false
        panel.allowedFileTypes = ["app"]
        panel.directoryURL = URL(fileURLWithPath: "/Applications", isDirectory: true)
        let response = panel.runModal()
        guard response == .OK else {
            return BRONZE_STATUS_CANCELLED
        }
        guard let url = panel.url, let row = installedAppRow(from: url) else {
            return BRONZE_STATUS_DEGRADED
        }
        let payload = "\(row.0)\t\(row.1)"
        let bytes = Array(payload.utf8)
        guard bytes.count <= listPayloadCap else {
            return BRONZE_STATUS_DEGRADED
        }
        return bytes.withUnsafeBufferPointer { buf in
            let src = bronze_native_utf8_view(ptr: buf.baseAddress, len: UInt64(bytes.count))
            return bronze_native_utf8_owned_copy(src, outBox.ptr)
        }
    }
}

private let settingsPathCap = 4096

@_silgen_name("bronze_native_pick_settings_export_path")
public func bronze_native_pick_settings_export_path(
    _ out: UnsafeMutablePointer<bronze_native_utf8_view>?
) -> UInt32 {
    guard let out else {
        return BRONZE_STATUS_NOT_FOUND
    }
    let outBox = BronzeOutViewBox(out)
    return bronzeOnAppKitModal {
        let panel = NSSavePanel()
        panel.canCreateDirectories = true
        panel.allowedFileTypes = ["json"]
        panel.nameFieldStringValue = "bronze-settings.json"
        let response = panel.runModal()
        guard response == .OK else {
            return BRONZE_STATUS_CANCELLED
        }
        return copyPickedSettingsPath(panel.url, to: outBox.ptr)
    }
}

@_silgen_name("bronze_native_pick_support_export_path")
public func bronze_native_pick_support_export_path(
    _ out: UnsafeMutablePointer<bronze_native_utf8_view>?
) -> UInt32 {
    guard let out else {
        return BRONZE_STATUS_NOT_FOUND
    }
    let outBox = BronzeOutViewBox(out)
    return bronzeOnAppKitModal {
        let panel = NSSavePanel()
        panel.canCreateDirectories = true
        panel.allowedFileTypes = ["txt"]
        panel.nameFieldStringValue = "bronze-support.txt"
        let response = panel.runModal()
        guard response == .OK else {
            return BRONZE_STATUS_CANCELLED
        }
        return copyPickedSupportPath(panel.url, to: outBox.ptr)
    }
}

@_silgen_name("bronze_native_pick_settings_import_path")
public func bronze_native_pick_settings_import_path(
    _ out: UnsafeMutablePointer<bronze_native_utf8_view>?
) -> UInt32 {
    guard let out else {
        return BRONZE_STATUS_NOT_FOUND
    }
    let outBox = BronzeOutViewBox(out)
    return bronzeOnAppKitModal {
        let panel = NSOpenPanel()
        panel.canChooseFiles = true
        panel.canChooseDirectories = false
        panel.allowsMultipleSelection = false
        panel.allowedFileTypes = ["json"]
        let response = panel.runModal()
        guard response == .OK else {
            return BRONZE_STATUS_CANCELLED
        }
        return copyPickedSettingsPath(panel.url, to: outBox.ptr)
    }
}

private func copyPickedSupportPath(
    _ url: URL?,
    to out: UnsafeMutablePointer<bronze_native_utf8_view>
) -> UInt32 {
    guard let url, url.isFileURL else {
        return BRONZE_STATUS_DEGRADED
    }
    let path = url.path
    guard !path.isEmpty, !path.contains("\0"), path.lowercased().hasSuffix(".txt") else {
        return BRONZE_STATUS_DEGRADED
    }
    let bytes = Array(path.utf8)
    guard bytes.count <= settingsPathCap else {
        return BRONZE_STATUS_DEGRADED
    }
    return bytes.withUnsafeBufferPointer { buf in
        let src = bronze_native_utf8_view(ptr: buf.baseAddress, len: UInt64(bytes.count))
        return bronze_native_utf8_owned_copy(src, out)
    }
}

private func copyPickedSettingsPath(
    _ url: URL?,
    to out: UnsafeMutablePointer<bronze_native_utf8_view>
) -> UInt32 {
    guard let url, url.isFileURL else {
        return BRONZE_STATUS_DEGRADED
    }
    let path = url.path
    guard !path.isEmpty, !path.contains("\0"), path.hasSuffix(".json") || path.lowercased().hasSuffix(".json") else {
        return BRONZE_STATUS_DEGRADED
    }
    let bytes = Array(path.utf8)
    guard bytes.count <= settingsPathCap else {
        return BRONZE_STATUS_DEGRADED
    }
    return bytes.withUnsafeBufferPointer { buf in
        let src = bronze_native_utf8_view(ptr: buf.baseAddress, len: UInt64(bytes.count))
        return bronze_native_utf8_owned_copy(src, out)
    }
}

private func installedAppRow(from url: URL) -> (String, String)? {
    guard url.pathExtension.caseInsensitiveCompare("app") == .orderedSame,
          let bundle = Bundle(url: url),
          let id = bundle.bundleIdentifier,
          !id.isEmpty,
          !id.contains("/"),
          !id.contains("\\"),
          !id.contains("..")
    else {
        return nil
    }
    let name = (bundle.localizedInfoDictionary?["CFBundleDisplayName"] as? String)
        ?? (bundle.infoDictionary?["CFBundleDisplayName"] as? String)
        ?? (bundle.localizedInfoDictionary?["CFBundleName"] as? String)
        ?? (bundle.infoDictionary?["CFBundleName"] as? String)
        ?? url.deletingPathExtension().lastPathComponent
    let cleanName = name
        .replacingOccurrences(of: "\t", with: " ")
        .replacingOccurrences(of: "\n", with: " ")
        .replacingOccurrences(of: "\r", with: " ")
        .trimmingCharacters(in: .whitespacesAndNewlines)
    guard !cleanName.isEmpty, !cleanName.hasPrefix("/"), !cleanName.hasPrefix("~"),
          !cleanName.contains("\\")
    else {
        return nil
    }
    return (id, cleanName)
}

private func rasterPng(_ image: NSImage, pixels: Int) -> Data? {
    let points = CGFloat(iconPointSize)
    let drawable = (image.copy() as? NSImage) ?? image
    drawable.size = NSSize(width: points, height: points)
    guard let rep = NSBitmapImageRep(
        bitmapDataPlanes: nil,
        pixelsWide: pixels,
        pixelsHigh: pixels,
        bitsPerSample: 8,
        samplesPerPixel: 4,
        hasAlpha: true,
        isPlanar: false,
        colorSpaceName: .deviceRGB,
        bytesPerRow: 0,
        bitsPerPixel: 0
    ) else {
        return nil
    }
    rep.size = NSSize(width: points, height: points)
    NSGraphicsContext.saveGraphicsState()
    guard let ctx = NSGraphicsContext(bitmapImageRep: rep) else {
        NSGraphicsContext.restoreGraphicsState()
        return nil
    }
    NSGraphicsContext.current = ctx
    ctx.imageInterpolation = .high
    ctx.cgContext.clear(CGRect(x: 0, y: 0, width: pixels, height: pixels))
    let dest = NSRect(x: 0, y: 0, width: points, height: points)
    if let best = drawable.bestRepresentation(for: dest, context: ctx, hints: [
        .interpolation: NSImageInterpolation.high,
    ]) {
        best.draw(in: dest)
    } else {
        drawable.draw(
            in: dest,
            from: .zero,
            operation: .sourceOver,
            fraction: 1
        )
    }
    NSGraphicsContext.restoreGraphicsState()
    return rep.representation(using: .png, properties: [:])
}
