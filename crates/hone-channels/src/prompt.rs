//! Prompt helpers for channel-specific guidance.

use hone_core::config::HoneConfig;
use hone_memory::SessionStorage;
use hone_memory::session::SessionPromptState;

pub const DEFAULT_GROUP_PRIVACY_GUARD: &str = "【群聊隐私约束】在群聊中禁止要求用户提供持仓、成交价、交易单等敏感信息；如需明细，引导其转为私聊提交。";
pub const DEFAULT_FINANCE_DOMAIN_POLICY: &str = "【领域边界与投研约束】\n\
- 你是金融分析助手，只回答与金融、市场、投资研究、宏观、行业、公司基本面、交易复盘和风险管理相关的问题。\n\
- 如用户问题与金融无关，直接礼貌拒绝，并提醒仅支持金融相关话题。\n\
- 禁止荐股：不要直接告诉用户“买哪只”“卖哪只”“梭哈哪只”或给出未经约束的单一标的推荐。\n\
- 当用户寻求操作建议时，必须改为分析买点、卖点、触发条件、失效条件、仓位与风险，而不是下指令式代客决策。\n\
- 任何涉及操作建议的回复，必须明确提醒：以下内容仅供分析参考，不要未经自己思考和风险评估就直接照做。\n\
- 若用户只是问候、寒暄或试探性打招呼，简短回复即可，不要展开成长篇分析。";

#[derive(Debug, Clone)]
pub struct PromptOptions {
    pub is_admin: bool,
    pub admin_prompt: Option<String>,
    pub privacy_guard: Option<String>,
    pub model_hint: Option<String>,
    pub force_chinese: bool,
    pub extra_sections: Vec<String>,
    pub include_format_guidance: bool,
}

impl Default for PromptOptions {
    fn default() -> Self {
        Self {
            is_admin: false,
            admin_prompt: None,
            privacy_guard: None,
            model_hint: None,
            force_chinese: false,
            extra_sections: Vec::new(),
            include_format_guidance: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PromptBundle {
    pub static_system: String,
    pub session_context: String,
    pub conversation_context: Option<String>,
}

impl PromptBundle {
    pub fn system_prompt(&self) -> String {
        if self.session_context.trim().is_empty() {
            self.static_system.clone()
        } else {
            format!("{}\n\n{}", self.static_system, self.session_context)
        }
    }

    pub fn compose_user_input(&self, user_input: &str) -> String {
        if let Some(context) = self
            .conversation_context
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
        {
            format!("{}\n\n【本轮用户输入】\n{}", context, user_input.trim())
        } else {
            user_input.to_string()
        }
    }
}

pub fn default_admin_prompt(project_root: &str) -> String {
    format!(
        "【管理员权限】\
        \n你正在与管理员用户交互。\
        \n1. 当前渠道运行在隔离沙盒内，默认不能直接浏览项目源码；如需运维动作，优先使用平台已暴露的工具。\
        \n2. 可调用 restart_hone(confirm=\"yes\") 工具重启 Hone（将重新编译并启动）\
        \n3. 如需人工排查源码，仓库根目录仍为：{project_root}\
        \n4. 管理员操作须谨慎，执行重启前应先确认影响范围\
        \n如需使用管理员功能，请先 load_skill(\"hone_admin\") 获取详细操作指引。"
    )
}

pub fn build_prompt_bundle(
    config: &HoneConfig,
    storage: &SessionStorage,
    channel: &str,
    session_id: &str,
    prompt_state: &SessionPromptState,
    options: &PromptOptions,
) -> PromptBundle {
    let frozen = prompt_state.frozen_datetime();
    let mut static_system = config
        .agent
        .system_prompt
        .replace("{{current_time_beijing}}", "")
        .replace("{{current_year}}", "")
        .replace("{{current_date}}", "")
        .replace("{{session_id}}", "")
        .replace("{{hone_version}}", env!("CARGO_PKG_VERSION"));

    static_system.push_str("\n\n");
    static_system.push_str(DEFAULT_FINANCE_DOMAIN_POLICY);

    if options.include_format_guidance {
        let format_guidance = channel_format_guidance(channel);
        if !format_guidance.is_empty() {
            static_system.push_str("\n\n");
            static_system.push_str(format_guidance);
        }
    }

    if let Some(guard) = options
        .privacy_guard
        .as_ref()
        .filter(|s| !s.trim().is_empty())
    {
        static_system.push_str("\n\n");
        static_system.push_str(guard.trim());
    }

    if let Some(model_hint) = options.model_hint.as_ref().filter(|s| !s.trim().is_empty()) {
        static_system.push_str(&format!("\n\n【基础模型】{}。", model_hint.trim()));
    }

    if options.force_chinese {
        static_system.push_str("\n【语言要求】必须全程以中文回复，禁止中英文混排或应答其他语言。");
    }

    if options.is_admin {
        let admin_note = if let Some(custom) = options
            .admin_prompt
            .as_ref()
            .filter(|s| !s.trim().is_empty())
        {
            custom.trim().to_string()
        } else {
            let project_root = std::env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| ".".to_string());
            default_admin_prompt(&project_root)
        };
        static_system.push_str("\n\n");
        static_system.push_str(&admin_note);
    }

    for section in &options.extra_sections {
        if !section.trim().is_empty() {
            static_system.push_str("\n\n");
            static_system.push_str(section.trim());
        }
    }

    let session_context = format!(
        "【Session 上下文】\n当前时间：{} (北京时间)\n当前日期：{}\n当前年份：{}\n会话 ID：{}",
        frozen.format("%Y-%m-%d %H:%M:%S"),
        frozen.format("%Y-%m-%d"),
        frozen.format("%Y"),
        session_id
    );

    let conversation_context = storage
        .load_session(session_id)
        .ok()
        .flatten()
        .and_then(|session| session.summary)
        .map(|summary| format!("【历史会话总结】\n{}", summary.content.trim()))
        .filter(|value| !value.trim().is_empty());

    PromptBundle {
        static_system,
        session_context,
        conversation_context,
    }
}

pub fn channel_format_guidance(channel: &str) -> &'static str {
    match channel {
        "discord" => DISCORD_FORMAT_GUIDANCE,
        "telegram" => TELEGRAM_FORMAT_GUIDANCE,
        "imessage" => IMESSAGE_FORMAT_GUIDANCE,
        "feishu" => FEISHU_FORMAT_GUIDANCE,
        _ => "",
    }
}

const DISCORD_FORMAT_GUIDANCE: &str = "【输出格式-Discord】\n\
- Discord 支持 Markdown：标题（# / ## / ###）、粗体/斜体/下划线/删除线、代码块、引用、列表与 Spoiler。\n\
- 标题仅用 #/##/###；更高层级不在官方指引内，部分客户端也有渲染问题，避免使用。\n\
- 不支持 Markdown 表格；需要表格时改为列表或代码块。\n\
- 社区反馈列表缩进可能失效，避免嵌套列表。";

const TELEGRAM_FORMAT_GUIDANCE: &str = "【输出格式-Telegram】\n\
- 当前使用 HTML 解析（parse_mode=HTML）。\n\
- 必须输出纯 HTML，不要混入任何 Markdown 语法；禁止出现 #、##、###、**粗体**、__粗体__、*斜体*、_斜体_、```代码块```、`行内代码` 这类 Markdown 标记。\n\
- 支持的核心标签：<b>/<strong>、<i>/<em>、<u>/<ins>、<s>/<strike>/<del>、<code>、<pre>、<a href=\"...\">。\n\
- 还可用 <span class=\"tg-spoiler\"> 或 <tg-spoiler> 做 spoiler；<blockquote> / <blockquote expandable> 做引用与可折叠引用。\n\
- 可用 <tg-emoji emoji-id=\"...\"> 和 <tg-time unix=\"...\" format=\"...\"> 做自定义 emoji 与时间展示，但仅在确实有价值时使用。\n\
- 代码块建议写成 <pre><code class=\"language-python\">...</code></pre> 这类嵌套结构；单独 <code> 不支持指定语言。\n\
- 文本中的 <、>、& 必须转义为 &lt;、&gt;、&amp;；除了官方支持标签外，不要使用其他 HTML 标签。\n\
- 未文档化表格支持，避免表格；需要表格时用 <pre>/<code> 生成等宽伪表格，或改为分行列表。\n\
- 如果原本想写 Markdown 标题或列表，请改成 HTML：标题用 <b>...</b>，列表用纯文本项目符号或分行，不要输出 Markdown 列表标记。\n\
- 输出保持简洁，优先用短标题 + 分行列表 + 代码块/引用，避免过长段落。";

const IMESSAGE_FORMAT_GUIDANCE: &str = "【输出格式-iMessage】\n\
- iMessage 文本格式依赖客户端（加粗/斜体/下划线/删除线等），其他设备可能仅显示纯文本。\n\
- 因此输出只用纯文本与清晰换行；不要依赖 Markdown 语法；列表用 1. 2. 3. 表达。";

const FEISHU_FORMAT_GUIDANCE: &str = "【输出格式-飞书】\n\
- 飞书富文本/卡片支持 Markdown 语法扩展，支持链接、图片、表格等元素，但有明确限制。\n\
- 标题仅支持一级与二级；列表不支持缩进；表格有列数与数量上限。\n\
- 当前渠道使用卡片渲染 Markdown，请保持简单：短段落 + 列表；表格尽量扁平，超过限制则改为列表或代码块。";

#[cfg(test)]
mod tests {
    use super::*;
    use hone_core::config::HoneConfig;
    use hone_memory::SessionStorage;
    use hone_memory::session::SessionPromptState;
    use std::fs;

    #[test]
    fn build_prompt_bundle_always_includes_finance_domain_policy() {
        let data_dir = std::env::temp_dir().join(format!(
            "hone-prompt-test-{}-{}",
            std::process::id(),
            hone_core::beijing_now()
                .timestamp_nanos_opt()
                .unwrap_or_default()
        ));
        fs::create_dir_all(&data_dir).expect("session storage dir should init");
        let storage = SessionStorage::new(data_dir.join("sessions"));
        let mut config = HoneConfig::default();
        config.agent.system_prompt = "你是 Hone。".to_string();
        let prompt_state = SessionPromptState::default();

        let bundle = build_prompt_bundle(
            &config,
            &storage,
            "telegram",
            "session-demo",
            &prompt_state,
            &PromptOptions::default(),
        );

        assert!(bundle.system_prompt().contains("【领域边界与投研约束】"));
        assert!(bundle.system_prompt().contains("禁止荐股"));
        assert!(
            bundle
                .system_prompt()
                .contains("不要未经自己思考和风险评估就直接照做")
        );
        assert!(bundle.system_prompt().contains("只回答与金融"));

        let _ = fs::remove_dir_all(&data_dir);
    }

    #[test]
    fn telegram_format_guidance_mentions_supported_html_features() {
        let guidance = channel_format_guidance("telegram");
        assert!(guidance.contains("parse_mode=HTML"));
        assert!(guidance.contains("tg-spoiler"));
        assert!(guidance.contains("blockquote expandable"));
        assert!(guidance.contains("tg-time"));
        assert!(guidance.contains("&lt;、&gt;、&amp;"));
    }
}
