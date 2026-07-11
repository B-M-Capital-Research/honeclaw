import Foundation

@main
enum NavigationPolicyTests {
    static func main() {
        let firstParty = [
            "https://hone-claw.com/chat",
            "https://www.hone-claw.com/privacy",
        ]
        let rejected = [
            "http://hone-claw.com/chat",
            "https://evil.example/?next=https://hone-claw.com/chat",
            "file:///tmp/private.txt",
            "javascript:alert(1)",
        ]

        for value in firstParty {
            precondition(HONEURLPolicy.isFirstParty(URL(string: value)!), value)
        }
        for value in rejected {
            precondition(!HONEURLPolicy.isFirstParty(URL(string: value)!), value)
        }
        precondition(HONEURLPolicy.canOpenExternally(URL(string: "mailto:bm@hone-claw.com")!))
        precondition(HONEURLPolicy.canOpenExternally(URL(string: "tel:+8613800000000")!))
        precondition(!HONEURLPolicy.canOpenExternally(URL(string: "data:text/plain,no")!))
        print("HONE iOS navigation policy: passed")
    }
}
