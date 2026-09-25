import AppKit

@_cdecl("bronze_native_frontmost_pid")
public func bronze_native_frontmost_pid() -> Int32 {
    if Thread.isMainThread {
        return MainActor.assumeIsolated {
            NSWorkspace.shared.frontmostApplication?.processIdentifier ?? 0
        }
    }
    let box = MainPidBox()
    let lock = DispatchSemaphore(value: 0)
    DispatchQueue.main.async {
        box.value = MainActor.assumeIsolated {
            NSWorkspace.shared.frontmostApplication?.processIdentifier ?? 0
        }
        lock.signal()
    }
    _ = lock.wait(timeout: .now() + 2)
    return box.value
}

private final class MainPidBox: @unchecked Sendable {
    var value: Int32 = 0
}
