// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "BronzeNative",
    products: [
        .library(name: "BronzeNative", type: .static, targets: ["BronzeNative"]),
    ],
    targets: [
        .target(name: "BronzeNative"),
    ]
)
