import AppKit

private let iconPngCap = 16 * 1024
private let iconPixels = 16

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

private func pngIcon(for key: String) -> Data? {
    let workspace = NSWorkspace.shared
    var image: NSImage?
    if let url = workspace.urlForApplication(withBundleIdentifier: key) {
        image = workspace.icon(forFile: url.path)
    }
    if image == nil {
        let match = workspace.runningApplications.first(where: {
            $0.bundleIdentifier == key || $0.localizedName == key
        })
        if let url = match?.bundleURL {
            image = workspace.icon(forFile: url.path)
        }
    }
    guard let image else {
        return nil
    }
    return rasterPng(image)
}

private func rasterPng(_ image: NSImage) -> Data? {
    let size = iconPixels
    guard let rep = NSBitmapImageRep(
        bitmapDataPlanes: nil,
        pixelsWide: size,
        pixelsHigh: size,
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
    rep.size = NSSize(width: size, height: size)
    NSGraphicsContext.saveGraphicsState()
    NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: rep)
    image.draw(
        in: NSRect(x: 0, y: 0, width: size, height: size),
        from: .zero,
        operation: .copy,
        fraction: 1
    )
    NSGraphicsContext.restoreGraphicsState()
    return rep.representation(using: .png, properties: [:])
}
