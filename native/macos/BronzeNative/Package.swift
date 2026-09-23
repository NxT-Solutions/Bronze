// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "BronzeNative",
    platforms: [.macOS(.v13)],
    products: [
        .library(name: "BronzeNative", type: .static, targets: ["BronzeNative"]),
        .executable(name: "BronzeNotice", targets: ["BronzeNotice"]),
    ],
    targets: [
        .target(
            name: "BronzeNative",
            publicHeadersPath: "include",
            linkerSettings: [
                .linkedFramework("AppKit"),
                .linkedFramework("UserNotifications"),
                .linkedFramework("ServiceManagement"),
            ]
        ),
        .executableTarget(
            name: "BronzeNotice",
            linkerSettings: [
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
