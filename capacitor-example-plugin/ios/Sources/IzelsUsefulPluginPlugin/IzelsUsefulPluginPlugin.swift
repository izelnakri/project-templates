import Capacitor
import Foundation

@objc public class IzelsUsefulPlugin: NSObject {}

@objc(IzelsUsefulPluginPlugin)
public class IzelsUsefulPluginPlugin: CAPPlugin {
    
    @objc func echoMe(_ call: CAPPluginCall) {
        print("Hello World")
        call.resolve([
            "value": "izel"
        ])
    }

    @objc func showToast(_ call: CAPPluginCall) {
        let message = call.getString("message") ?? "No message"
        DispatchQueue.main.async {
            let alert = UIAlertController(title: "Toast", message: message, preferredStyle: .alert)
            self.bridge?.viewController?.present(alert, animated: true)
            DispatchQueue.main.asyncAfter(deadline: .now() + 2.0) {
                alert.dismiss(animated: true, completion: nil)
            }
        }
        call.resolve()
    }
}
// NOTE: Initially generated one:
//
// import Foundation
// import Capacitor
//
// /**
//  * Please read the Capacitor iOS Plugin Development Guide
//  * here: https://capacitorjs.com/docs/plugins/ios
//  */
// @objc(IzelsUsefulPluginPlugin)
// public class IzelsUsefulPluginPlugin: CAPPlugin, CAPBridgedPlugin {
//     public let identifier = "IzelsUsefulPluginPlugin"
//     public let jsName = "IzelsUsefulPlugin"
//     public let pluginMethods: [CAPPluginMethod] = [
//         CAPPluginMethod(name: "echo", returnType: CAPPluginReturnPromise)
//     ]
//     private let implementation = IzelsUsefulPlugin()
//
//     @objc func echo(_ call: CAPPluginCall) {
//         let value = call.getString("value") ?? ""
//         call.resolve([
//             "value": implementation.echo(value)
//         ])
//     }
// }
