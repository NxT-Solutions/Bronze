import AppKit

@_cdecl("bronze_native_frontmost_pid")
public func bronze_native_frontmost_pid() -> Int32 {
    NSWorkspace.shared.frontmostApplication?.processIdentifier ?? 0
}
