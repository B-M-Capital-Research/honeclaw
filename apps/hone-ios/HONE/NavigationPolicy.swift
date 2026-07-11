import Foundation

enum HONEURLPolicy {
    static let appURL = URL(string: "https://hone-claw.com/chat")!

    private static let firstPartyHosts: Set<String> = [
        "hone-claw.com",
        "www.hone-claw.com",
    ]

    static func isFirstParty(_ url: URL) -> Bool {
        url.scheme?.lowercased() == "https"
            && firstPartyHosts.contains(url.host?.lowercased() ?? "")
    }

    static func canOpenExternally(_ url: URL) -> Bool {
        guard let scheme = url.scheme?.lowercased() else { return false }
        return ["http", "https", "mailto", "tel"].contains(scheme)
    }
}
