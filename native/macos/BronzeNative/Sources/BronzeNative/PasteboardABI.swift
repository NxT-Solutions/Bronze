import AppKit
import Carbon.HIToolbox
import CoreGraphics

private let bronzeSyntheticCopyUserData: Int64 = 0x42524E5A434F5059
private let bronzeSnapshotMagic: UInt32 = 0x42524E53
private let bronzeMarkupMaxBytes = 1 << 20
private let bronzeSnapshotMaxBytes = 2 << 20

@_silgen_name("bronze_native_pasteboard_write")
public func bronze_native_pasteboard_write(
    _ plain: bronze_native_utf8_view,
    _ html: bronze_native_utf8_view
) -> UInt32 {
    if bronze_native_validate_utf8(plain) != BRONZE_STATUS_OK {
        return BRONZE_STATUS_INVALID_UTF8
    }
    if bronze_native_validate_utf8(html) != BRONZE_STATUS_OK {
        return BRONZE_STATUS_INVALID_UTF8
    }
    guard let plainText = bronzeUtf8String(plain), let htmlText = bronzeUtf8String(html) else {
        return BRONZE_STATUS_INVALID_UTF8
    }
    return bronzeOnAppKit {
        let board = NSPasteboard.general
        board.clearContents()
        let item = NSPasteboardItem()
        guard item.setString(plainText, forType: .string) else {
            return board.setString(plainText, forType: .string)
                ? BRONZE_STATUS_OK
                : BRONZE_STATUS_DEGRADED
        }
        if !htmlText.isEmpty, let htmlData = htmlText.data(using: .utf8) {
            item.setData(htmlData, forType: .html)
        }
        if board.writeObjects([item]) {
            return BRONZE_STATUS_OK
        }
        return board.setString(plainText, forType: .string)
            ? BRONZE_STATUS_OK
            : BRONZE_STATUS_DEGRADED
    }
}

@_silgen_name("bronze_native_bounded_copy_read")
public func bronze_native_bounded_copy_read(
    _ target_pid: Int32,
    _ kind: UnsafeMutablePointer<UInt32>?,
    _ post_copy_count: UnsafeMutablePointer<UInt64>?,
    _ payload: UnsafeMutablePointer<bronze_native_utf8_view>?,
    _ snapshot: UnsafeMutablePointer<bronze_native_utf8_view>?
) -> UInt32 {
    guard let kind, let post_copy_count, let payload, let snapshot else {
        return BRONZE_STATUS_NOT_FOUND
    }
    let out = BoundedCopyOut(
        kind: kind,
        post: post_copy_count,
        payload: payload,
        snapshot: snapshot
    )
    return bronzeOnAppKitCopy {
        bronzeBoundedCopyRead(targetPid: target_pid, out: out)
    }
}

@_silgen_name("bronze_native_pasteboard_restore_if_unchanged")
public func bronze_native_pasteboard_restore_if_unchanged(
    _ snapshot: bronze_native_utf8_view,
    _ expected_count: UInt64
) -> UInt32 {
    if snapshot.len > 0 && snapshot.ptr == nil {
        return BRONZE_STATUS_INVALID_UTF8
    }
    let bytes: [UInt8]
    if snapshot.len == 0 {
        bytes = []
    } else {
        let count = Int(snapshot.len)
        bytes = Array(UnsafeBufferPointer(start: snapshot.ptr, count: count))
    }
    return bronzeOnAppKit {
        let board = NSPasteboard.general
        if board.changeCount != Int(expected_count) {
            return BRONZE_STATUS_OK
        }
        if bytes.isEmpty {
            return BRONZE_STATUS_OK
        }
        guard let items = decodeTextualSnapshot(bytes) else {
            return BRONZE_STATUS_OK
        }
        board.clearContents()
        let item = NSPasteboardItem()
        for (uti, data) in items {
            item.setData(data, forType: NSPasteboard.PasteboardType(uti))
        }
        _ = board.writeObjects([item])
        return BRONZE_STATUS_OK
    }
}

private final class BoundedCopyOut: @unchecked Sendable {
    let kind: UnsafeMutablePointer<UInt32>
    let post: UnsafeMutablePointer<UInt64>
    let payload: UnsafeMutablePointer<bronze_native_utf8_view>
    let snapshot: UnsafeMutablePointer<bronze_native_utf8_view>
    init(
        kind: UnsafeMutablePointer<UInt32>,
        post: UnsafeMutablePointer<UInt64>,
        payload: UnsafeMutablePointer<bronze_native_utf8_view>,
        snapshot: UnsafeMutablePointer<bronze_native_utf8_view>
    ) {
        self.kind = kind
        self.post = post
        self.payload = payload
        self.snapshot = snapshot
    }
}

private final class CopyStatusBox: @unchecked Sendable {
    var value: UInt32 = BRONZE_STATUS_DEGRADED
}

private func bronzeOnAppKitCopy(_ work: @escaping @Sendable () -> UInt32) -> UInt32 {
    if Thread.isMainThread {
        return work()
    }
    let box = CopyStatusBox()
    let lock = DispatchSemaphore(value: 0)
    DispatchQueue.main.async {
        box.value = work()
        lock.signal()
    }
    if lock.wait(timeout: .now() + 3.5) == .timedOut {
        return BRONZE_STATUS_DEGRADED
    }
    return box.value
}

private func bronzeBoundedCopyRead(targetPid: Int32, out: BoundedCopyOut) -> UInt32 {
    if !waitPhysicalModifiersUp(until: Date().addingTimeInterval(0.4)) {
        return BRONZE_STATUS_DEGRADED
    }
    let board = NSPasteboard.general
    let snapBytes = encodeTextualSnapshot(board)
    if targetPid > 0 {
        guard let app = NSRunningApplication(processIdentifier: pid_t(targetPid)) else {
            return BRONZE_STATUS_DEGRADED
        }
        if #available(macOS 14.0, *) {
            app.activate()
        } else {
            app.activate(options: [.activateIgnoringOtherApps])
        }
        Thread.sleep(forTimeInterval: 0.03)
    }
    let baseline = board.changeCount
    if !postTaggedCopy() {
        return BRONZE_STATUS_DEGRADED
    }
    guard let generation = waitStableGeneration(
        board,
        baseline: baseline,
        until: Date().addingTimeInterval(0.8)
    ) else {
        return BRONZE_STATUS_DEGRADED
    }
    guard let preferred = readPreferredText(board), preferred.1.utf8.count <= bronzeMarkupMaxBytes else {
        return BRONZE_STATUS_DEGRADED
    }
    out.kind.pointee = preferred.0
    out.post.pointee = UInt64(generation)
    let payloadBytes = Array(preferred.1.utf8)
    let payloadStatus = payloadBytes.withUnsafeBufferPointer { buf in
        bronze_native_utf8_owned_copy(
            bronze_native_utf8_view(ptr: buf.baseAddress, len: UInt64(buf.count)),
            out.payload
        )
    }
    if payloadStatus != BRONZE_STATUS_OK {
        return payloadStatus
    }
    let snap = snapBytes.count > bronzeSnapshotMaxBytes ? [] : snapBytes
    let snapStatus = snap.withUnsafeBufferPointer { buf in
        bronzeOwnedBytesCopy(
            bronze_native_utf8_view(ptr: buf.baseAddress, len: UInt64(buf.count)),
            out.snapshot
        )
    }
    if snapStatus != BRONZE_STATUS_OK {
        _ = bronze_native_utf8_free(out.payload.pointee)
        return snapStatus
    }
    return BRONZE_STATUS_OK
}

private func waitPhysicalModifiersUp(until deadline: Date) -> Bool {
    while Date() < deadline {
        let flags = CGEventSource.flagsState(.hidSystemState)
        if !flags.contains(.maskCommand)
            && !flags.contains(.maskShift)
            && !flags.contains(.maskAlternate)
            && !flags.contains(.maskControl)
        {
            return true
        }
        Thread.sleep(forTimeInterval: 0.01)
    }
    return false
}

private func postTaggedCopy() -> Bool {
    let command = CGKeyCode(kVK_Command)
    let cKey = CGKeyCode(kVK_ANSI_C)
    var commandDown = false
    var cDown = false
    func post(_ key: CGKeyCode, down: Bool, flags: CGEventFlags) -> Bool {
        guard let source = CGEventSource(stateID: .hidSystemState) else {
            return false
        }
        guard let ev = CGEvent(keyboardEventSource: source, virtualKey: key, keyDown: down) else {
            return false
        }
        ev.flags = flags
        ev.setIntegerValueField(.eventSourceUserData, value: bronzeSyntheticCopyUserData)
        ev.post(tap: .cghidEventTap)
        return true
    }
    func cleanup() {
        if cDown {
            _ = post(cKey, down: false, flags: .maskCommand)
            cDown = false
        }
        if commandDown {
            _ = post(command, down: false, flags: [])
            commandDown = false
        }
    }
    guard post(command, down: true, flags: .maskCommand) else {
        return false
    }
    commandDown = true
    guard post(cKey, down: true, flags: .maskCommand) else {
        cleanup()
        return false
    }
    cDown = true
    guard post(cKey, down: false, flags: .maskCommand) else {
        cleanup()
        return false
    }
    cDown = false
    guard post(command, down: false, flags: []) else {
        cleanup()
        return false
    }
    commandDown = false
    return true
}

private func waitStableGeneration(_ board: NSPasteboard, baseline: Int, until deadline: Date) -> Int? {
    var last = board.changeCount
    var stable = 0
    var delay: TimeInterval = 0.008
    while Date() < deadline {
        Thread.sleep(forTimeInterval: delay)
        let now = board.changeCount
        if now > baseline {
            if now == last {
                stable += 1
                if stable >= 2 {
                    return now
                }
            } else {
                stable = 0
                last = now
            }
        }
        delay = min(0.04, delay * 1.4)
    }
    return nil
}

private func readPreferredText(_ board: NSPasteboard) -> (UInt32, String)? {
    if let html = board.string(forType: .html), !html.isEmpty {
        return (BRONZE_PASTEBOARD_KIND_HTML, html)
    }
    if let data = board.data(forType: .html),
       let html = String(data: data, encoding: .utf8),
       !html.isEmpty
    {
        return (BRONZE_PASTEBOARD_KIND_HTML, html)
    }
    if let data = board.data(forType: .rtf) {
        let rtf = String(data: data, encoding: .utf8) ?? String(data: data, encoding: .isoLatin1)
        if let rtf, !rtf.isEmpty {
            return (BRONZE_PASTEBOARD_KIND_RTF, rtf)
        }
    }
    if let plain = board.string(forType: .string), !plain.isEmpty {
        return (BRONZE_PASTEBOARD_KIND_PLAIN, plain)
    }
    return nil
}

private func textualTypes() -> [NSPasteboard.PasteboardType] {
    [
        .string,
        .html,
        .rtf,
        NSPasteboard.PasteboardType("public.utf8-plain-text"),
        NSPasteboard.PasteboardType("public.utf16-plain-text"),
        NSPasteboard.PasteboardType("public.html"),
        NSPasteboard.PasteboardType("public.rtf"),
    ]
}

private func encodeTextualSnapshot(_ board: NSPasteboard) -> [UInt8] {
    var items: [(String, Data)] = []
    var seen = Set<String>()
    for type in textualTypes() {
        let uti = type.rawValue
        if seen.contains(uti) {
            continue
        }
        guard let data = board.data(forType: type), !data.isEmpty else {
            continue
        }
        seen.insert(uti)
        items.append((uti, data))
    }
    var out: [UInt8] = []
    func putU32(_ value: UInt32) {
        var le = value.littleEndian
        withUnsafeBytes(of: &le) { out.append(contentsOf: $0) }
    }
    putU32(bronzeSnapshotMagic)
    putU32(1)
    putU32(UInt32(items.count))
    for (uti, data) in items {
        let name = Array(uti.utf8)
        putU32(UInt32(name.count))
        out.append(contentsOf: name)
        putU32(UInt32(data.count))
        out.append(contentsOf: data)
    }
    return out
}

private func decodeTextualSnapshot(_ bytes: [UInt8]) -> [(String, Data)]? {
    var i = 0
    func takeU32() -> UInt32? {
        guard i + 4 <= bytes.count else {
            return nil
        }
        let value = UInt32(bytes[i])
            | UInt32(bytes[i + 1]) << 8
            | UInt32(bytes[i + 2]) << 16
            | UInt32(bytes[i + 3]) << 24
        i += 4
        return value
    }
    guard takeU32() == bronzeSnapshotMagic, takeU32() == 1, let count = takeU32() else {
        return nil
    }
    var items: [(String, Data)] = []
    for _ in 0..<count {
        guard let nameLen = takeU32(), i + Int(nameLen) <= bytes.count else {
            return nil
        }
        let nameBytes = bytes[i..<i + Int(nameLen)]
        i += Int(nameLen)
        guard let name = String(bytes: nameBytes, encoding: .utf8),
              let dataLen = takeU32(),
              i + Int(dataLen) <= bytes.count
        else {
            return nil
        }
        let data = Data(bytes[i..<i + Int(dataLen)])
        i += Int(dataLen)
        items.append((name, data))
    }
    return items
}
