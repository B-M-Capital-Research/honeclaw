import Foundation

@MainActor
final class BrowserModel: ObservableObject {
    enum Phase: Equatable {
        case loading
        case ready
        case failed(String)
    }

    @Published private(set) var phase: Phase = .loading
    @Published private(set) var reloadID = UUID()

    func didStartLoading() {
        if phase != .ready {
            phase = .loading
        }
    }

    func didFinishLoading() {
        phase = .ready
    }

    func didFailLoading(_ error: Error) {
        let message = (error as NSError).code == NSURLErrorNotConnectedToInternet
            ? "当前处于离线状态，联网后即可继续"
            : "暂时无法连接 HONE，请稍后重试"
        phase = .failed(message)
    }

    func retry() {
        phase = .loading
        reloadID = UUID()
    }
}
