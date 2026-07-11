import SwiftUI

struct RootView: View {
    @StateObject private var browser = BrowserModel()

    var body: some View {
        ZStack {
            HONEWebView(model: browser)
                .opacity(browser.phase == .ready ? 1 : 0)

            switch browser.phase {
            case .loading:
                BrandLaunchView()
                    .transition(.opacity)
            case let .failed(message):
                OfflineView(message: message, retry: browser.retry)
                    .transition(.opacity)
            case .ready:
                EmptyView()
            }
        }
        .background(Color.honeCanvas)
        .animation(.easeOut(duration: 0.28), value: browser.phase)
    }
}

private struct BrandLaunchView: View {
    @State private var appeared = false

    var body: some View {
        ZStack {
            HONEBackground()
            VStack(spacing: 0) {
                Spacer()
                HONEBrandLockup()
                    .scaleEffect(appeared ? 1 : 0.94)
                    .opacity(appeared ? 1 : 0)
                Text("让重要的信息，在需要时抵达。")
                    .font(.system(size: 24, weight: .semibold, design: .serif))
                    .foregroundStyle(Color.honeInk)
                    .padding(.top, 28)
                Text("正在安全连接 hone-claw.com")
                    .font(.system(size: 12, weight: .medium))
                    .foregroundStyle(Color.honeMuted)
                    .padding(.top, 12)
                ProgressView()
                    .tint(Color.honeOrange)
                    .padding(.top, 24)
                Spacer()
                Text("登录状态仅保存在这台设备的系统 WebKit 数据中")
                    .font(.system(size: 10, weight: .medium))
                    .foregroundStyle(Color.honeMuted.opacity(0.8))
                    .padding(.bottom, 22)
            }
            .padding(.horizontal, 28)
        }
        .onAppear {
            withAnimation(.spring(response: 0.7, dampingFraction: 0.84)) {
                appeared = true
            }
        }
    }
}

private struct OfflineView: View {
    let message: String
    let retry: () -> Void

    var body: some View {
        ZStack {
            HONEBackground()
            VStack(spacing: 0) {
                HONEBrandLockup()
                Text("暂时没有连接成功")
                    .font(.system(size: 24, weight: .semibold, design: .serif))
                    .foregroundStyle(Color.honeInk)
                    .padding(.top, 30)
                Text(message)
                    .font(.system(size: 13, weight: .regular))
                    .foregroundStyle(Color.honeMuted)
                    .multilineTextAlignment(.center)
                    .padding(.top, 10)
                Button(action: retry) {
                    HStack(spacing: 10) {
                        Text("重新连接")
                        Image(systemName: "arrow.clockwise")
                    }
                    .font(.system(size: 14, weight: .bold))
                    .foregroundStyle(.white)
                    .frame(maxWidth: .infinity)
                    .frame(height: 50)
                    .background(Color.honeInk, in: RoundedRectangle(cornerRadius: 16))
                }
                .buttonStyle(.plain)
                .padding(.top, 26)
                .frame(maxWidth: 280)
            }
            .padding(.horizontal, 28)
        }
    }
}

private struct HONEBrandLockup: View {
    var body: some View {
        VStack(spacing: 14) {
            Image("HoneMark")
                .resizable()
                .frame(width: 92, height: 92)
                .shadow(color: Color.black.opacity(0.12), radius: 18, y: 10)
            Text("HONE")
                .font(.system(size: 20, weight: .heavy, design: .rounded))
                .tracking(5.5)
                .foregroundStyle(Color.honeInk)
                .padding(.leading, 5.5)
        }
        .accessibilityElement(children: .ignore)
        .accessibilityLabel("HONE")
    }
}

private struct HONEBackground: View {
    var body: some View {
        ZStack {
            Color.honeCanvas
            RadialGradient(
                colors: [Color.honeOrange.opacity(0.22), .clear],
                center: .topTrailing,
                startRadius: 10,
                endRadius: 360
            )
            RadialGradient(
                colors: [Color.honeSlate.opacity(0.16), .clear],
                center: .bottomLeading,
                startRadius: 10,
                endRadius: 330
            )
        }
        .ignoresSafeArea()
    }
}

private extension Color {
    static let honeCanvas = Color(red: 0.969, green: 0.957, blue: 0.925)
    static let honeInk = Color(red: 0.125, green: 0.157, blue: 0.173)
    static let honeMuted = Color(red: 0.42, green: 0.46, blue: 0.48)
    static let honeOrange = Color(red: 0.93, green: 0.36, blue: 0.04)
    static let honeSlate = Color(red: 0.25, green: 0.36, blue: 0.40)
}
