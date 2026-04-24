//! `hone-cli` 所有子命令共用的终端显示 helper。
//!
//! 目标:让 onboard / configure 这类多段 TUI 在一屏扫过时能立刻分辨出
//! 「我现在在第几步、是警告还是成功、哪条是标题哪条是正文」,不靠用户自己
//! 读全屏文字去猜。
//!
//! 所有 helper 都直接 `println!`,没有缓存或 style builder —— 因为 onboard
//! 流程本来就是顺序线性,不需要局部刷新。色彩依赖 `console` crate;在
//! stdout 不是 tty / 用户设了 `NO_COLOR` 时 `console` 会自动退化成明文。

use console::style;

/// 一个大的 section 分隔符,比如 `━━ Step 3/6: Channels ━━━━━━━━━━━━━━━━━`。
/// 用在 onboard 从一个环节切到下一个环节时,让用户立即看到进度。
pub(crate) fn step_header(step: usize, total: usize, title: &str) {
    let prefix = format!("━━ Step {step}/{total}: {title} ");
    let pad_len = 72usize.saturating_sub(prefix.chars().count());
    let line = format!("{}{}", prefix, "━".repeat(pad_len));
    println!();
    println!("{}", style(line).cyan().bold());
}

/// 小标题,例如 channel 的 `Feishu prerequisites`。
/// 比 step_header 一级,用来把 step 内部分成几块。
pub(crate) fn subsection(title: &str) {
    println!();
    println!("{}", style(format!("▸ {title}")).cyan().bold());
}

pub(crate) fn bullet(text: &str) {
    println!("  {} {}", style("·").dim(), text);
}

/// 黄色 `⚠` 警告行。专供「默认行为可能让用户意外」场景,比如 allow_* 白名单默认开放。
pub(crate) fn warn_line(text: &str) {
    println!("  {} {}", style("⚠").yellow().bold(), style(text).yellow());
}

pub(crate) fn ok_line(text: &str) {
    println!("  {} {}", style("✓").green().bold(), text);
}

pub(crate) fn fail_line(text: &str) {
    println!("  {} {}", style("✗").red().bold(), style(text).red());
}

/// 次要信息 (hint / note / 文件路径)。用 dim 让它退到视觉背景。
pub(crate) fn hint_line(text: &str) {
    println!("  {}", style(text).dim());
}

/// 一个居中的 banner (welcome / all done 场景用),两行 border + 一行标题 + subtitle。
pub(crate) fn banner(title: &str, subtitle: &str) {
    let width = 72usize;
    let bar = "━".repeat(width);
    println!();
    println!("{}", style(&bar).cyan());
    println!("{}", style(format!("  {title}")).cyan().bold());
    if !subtitle.is_empty() {
        println!("  {}", style(subtitle).dim());
    }
    println!("{}", style(&bar).cyan());
}

#[cfg(test)]
mod tests {
    //! 这些测试主要用来肉眼验证视觉效果。跑法:
    //! ```sh
    //! FORCE_COLOR=1 cargo test -p hone-cli display::tests -- --nocapture
    //! ```
    //! 在非 tty 下 `console` 会自动退化成明文,ANSI 转义需 `FORCE_COLOR=1` 强制开。

    use super::*;

    #[test]
    fn onboarding_preview_smoke() {
        banner(
            "Hone onboarding",
            "约 3–5 分钟,Ctrl+C 安全退出:mutation 只在最后一步才写盘。",
        );
        hint_line("每个环节都可以跳过,之后再通过 `hone-cli onboard` 补配。");

        step_header(1, 5, "Runner");
        subsection("Multi-Agent (OpenRouter)");
        bullet("前置:一把可用的 OpenRouter API key。");
        bullet("原理:第一段 search 用小模型拉证据,第二段 answer 用主模型总结。");
        ok_line("codex 已检测到可用。");

        step_header(2, 5, "Channels");
        subsection("Feishu prerequisites");
        bullet("需要飞书开放平台应用的 `app_id` 与 `app_secret`。");
        warn_line("Feishu 渠道默认 allow 白名单为空,即所有联系人都能触发 Hone。");
        hint_line("如需限定,onboard 完成后用 `hone-cli configure --section channels`。");

        step_header(3, 5, "Admins");
        hint_line("管理员白名单决定谁能触发 /register-admin /report 等管理指令。");

        step_header(4, 5, "Providers");
        subsection("OpenRouter API keys");
        bullet("LLM 主路由。multi-agent / codex_* / nano_banana 都默认走这里。");
        ok_line("已保存 OpenRouter API keys。");
        fail_line("Token 必须是三段结构(长度=12)。");

        step_header(5, 5, "Apply");
        ok_line("配置已保存,已立即生效(共写入 6 条字段)");
        hint_line("canonical config → /Users/you/.honeclaw/config.yaml");

        banner("Onboarding complete", "下一步:");
        bullet("`hone-cli status`   快速查当前配置");
    }
}
