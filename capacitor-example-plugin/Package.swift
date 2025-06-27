// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "CapacitorIzelsExamplePlugin",
    platforms: [.iOS(.v14)],
    products: [
        .library(
            name: "CapacitorIzelsExamplePlugin",
            targets: ["IzelsUsefulPluginPlugin"])
    ],
    dependencies: [
        .package(url: "https://github.com/ionic-team/capacitor-swift-pm.git", from: "7.0.0")
    ],
    targets: [
        .target(
            name: "IzelsUsefulPluginPlugin",
            dependencies: [
                .product(name: "Capacitor", package: "capacitor-swift-pm"),
                .product(name: "Cordova", package: "capacitor-swift-pm")
            ],
            path: "ios/Sources/IzelsUsefulPluginPlugin"),
        .testTarget(
            name: "IzelsUsefulPluginPluginTests",
            dependencies: ["IzelsUsefulPluginPlugin"],
            path: "ios/Tests/IzelsUsefulPluginPluginTests")
    ]
)