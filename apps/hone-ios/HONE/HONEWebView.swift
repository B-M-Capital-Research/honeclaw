import SwiftUI
import UIKit
import WebKit

struct HONEWebView: UIViewRepresentable {
    @ObservedObject var model: BrowserModel

    func makeCoordinator() -> Coordinator {
        Coordinator(model: model)
    }

    func makeUIView(context: Context) -> WKWebView {
        let configuration = WKWebViewConfiguration()
        configuration.websiteDataStore = .default()
        configuration.applicationNameForUserAgent = "HONE-iOS"
        configuration.defaultWebpagePreferences.allowsContentJavaScript = true

        let webView = WKWebView(frame: .zero, configuration: configuration)
        webView.navigationDelegate = context.coordinator
        webView.uiDelegate = context.coordinator
        webView.allowsBackForwardNavigationGestures = true
        webView.allowsLinkPreview = true
        webView.scrollView.keyboardDismissMode = .interactive
        webView.isOpaque = false
        webView.backgroundColor = UIColor(red: 0.969, green: 0.957, blue: 0.925, alpha: 1)
        context.coordinator.reload(webView, reloadID: model.reloadID)
        return webView
    }

    func updateUIView(_ webView: WKWebView, context: Context) {
        guard context.coordinator.reloadID != model.reloadID else { return }
        context.coordinator.reload(webView, reloadID: model.reloadID)
    }

    final class Coordinator: NSObject, WKNavigationDelegate, WKUIDelegate {
        private let model: BrowserModel
        fileprivate var reloadID: UUID?

        init(model: BrowserModel) {
            self.model = model
        }

        fileprivate func reload(_ webView: WKWebView, reloadID: UUID) {
            self.reloadID = reloadID
            webView.load(URLRequest(url: HONEURLPolicy.appURL))
        }

        func webView(
            _ webView: WKWebView,
            decidePolicyFor navigationAction: WKNavigationAction,
            decisionHandler: @escaping (WKNavigationActionPolicy) -> Void
        ) {
            guard let url = navigationAction.request.url else {
                decisionHandler(.cancel)
                return
            }

            if HONEURLPolicy.isFirstParty(url) {
                if navigationAction.targetFrame == nil {
                    webView.load(URLRequest(url: url))
                    decisionHandler(.cancel)
                } else {
                    decisionHandler(.allow)
                }
                return
            }

            if HONEURLPolicy.canOpenExternally(url) {
                UIApplication.shared.open(url)
            }
            decisionHandler(.cancel)
        }

        func webView(
            _ webView: WKWebView,
            createWebViewWith configuration: WKWebViewConfiguration,
            for navigationAction: WKNavigationAction,
            windowFeatures: WKWindowFeatures
        ) -> WKWebView? {
            guard let url = navigationAction.request.url else { return nil }
            if HONEURLPolicy.isFirstParty(url) {
                webView.load(URLRequest(url: url))
            } else if HONEURLPolicy.canOpenExternally(url) {
                UIApplication.shared.open(url)
            }
            return nil
        }

        func webView(_ webView: WKWebView, didStartProvisionalNavigation navigation: WKNavigation?) {
            model.didStartLoading()
        }

        func webView(_ webView: WKWebView, didFinish navigation: WKNavigation?) {
            model.didFinishLoading()
        }

        func webView(
            _ webView: WKWebView,
            didFailProvisionalNavigation navigation: WKNavigation?,
            withError error: Error
        ) {
            model.didFailLoading(error)
        }

        func webView(_ webView: WKWebView, didFail navigation: WKNavigation?, withError error: Error) {
            model.didFailLoading(error)
        }

        func webViewWebContentProcessDidTerminate(_ webView: WKWebView) {
            webView.reload()
        }
    }
}
