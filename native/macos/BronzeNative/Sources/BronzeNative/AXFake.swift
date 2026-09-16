// Fixture AX provider (story 3.6). No live ApplicationServices queries.

public enum AXRole: Sendable {
    case application
    case window
    case group
    case scrollArea
    case textField
    case textArea
    case staticText
    case secureTextField
    case webArea
    case unknown
}

public enum AXSubrole: Sendable {
    case secureTextField
    case other
}

public enum AXProtection: Sendable {
    case protected
    case allowedText
    case neutralContainer
    case unknownContentBearing
}

public enum AXFakeSelection: Sendable {
    case missing
    case empty
    case text(String)
}

public enum AXOutcome: Sendable, Equatable {
    case captured(length: Int)
    case noSelection
    case protectedContent
    case protectionUnknown
    case appExcluded
    case accessibilityDenied
    case focusedElementMissing
}

public final class AXFakeNode: @unchecked Sendable {
    public let role: AXRole
    public let subrole: AXSubrole?
    public let selection: AXFakeSelection
    public let parent: AXFakeNode?
    private var queries: UInt32 = 0

    public init(
        role: AXRole,
        subrole: AXSubrole? = nil,
        selection: AXFakeSelection = .missing,
        parent: AXFakeNode? = nil
    ) {
        self.role = role
        self.subrole = subrole
        self.selection = selection
        self.parent = parent
    }

    public var queryCount: UInt32 { queries }

    fileprivate func querySelection() -> AXFakeSelection {
        queries += 1
        return selection
    }
}

public struct AXFakeTree: Sendable {
    public var focused: AXFakeNode?
    public var excluded: Bool
    public var accessibilityGranted: Bool

    public init(
        focused: AXFakeNode?,
        excluded: Bool = false,
        accessibilityGranted: Bool = true
    ) {
        self.focused = focused
        self.excluded = excluded
        self.accessibilityGranted = accessibilityGranted
    }
}

public func axClassify(role: AXRole, subrole: AXSubrole?) -> AXProtection {
    if role == .secureTextField || subrole == .secureTextField {
        return .protected
    }
    switch role {
    case .application, .window, .group, .scrollArea:
        return .neutralContainer
    case .textField, .textArea, .staticText:
        return .allowedText
    case .webArea, .unknown, .secureTextField:
        return .unknownContentBearing
    }
}

public func axCapture(_ tree: AXFakeTree) -> (AXOutcome, String?) {
    guard tree.accessibilityGranted else {
        return (.accessibilityDenied, nil)
    }
    guard !tree.excluded else {
        return (.appExcluded, nil)
    }
    guard var node = tree.focused else {
        return (.focusedElementMissing, nil)
    }
    var steps = 0
    while steps < 16 {
        steps += 1
        switch axClassify(role: node.role, subrole: node.subrole) {
        case .protected:
            return (.protectedContent, nil)
        case .unknownContentBearing:
            return (.protectionUnknown, nil)
        case .neutralContainer:
            guard let parent = node.parent else { return (.noSelection, nil) }
            node = parent
        case .allowedText:
            switch node.querySelection() {
            case .missing, .empty:
                guard let parent = node.parent else { return (.noSelection, nil) }
                node = parent
            case .text(let text):
                if text.isEmpty {
                    guard let parent = node.parent else { return (.noSelection, nil) }
                    node = parent
                } else {
                    return (.captured(length: text.utf8.count), text)
                }
            }
        }
    }
    return (.noSelection, nil)
}
