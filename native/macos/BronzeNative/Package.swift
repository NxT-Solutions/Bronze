// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "BronzeNative",
    platforms: [.macOS(.v13)],
    products: [
        .library(name: "BronzeNative", type: .static, targets: ["BronzeNative"]),
    ],
    targets: [
        .target(
            name: "BronzeNative",
            publicHeadersPath: "include",
            linkerSettings: [
                .linkedFramework("NaturalLanguage"),
                .linkedFramework("AppKit"),
                .linkedFramework("UserNotifications"),
            ]
        ),
        .testTarget(
            name: "BronzeNativeTests",
            dependencies: ["BronzeNative"]
        ),
    ]
)
