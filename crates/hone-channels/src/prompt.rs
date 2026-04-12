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
pub const DEFAULT_CRON_TASK_POLICY: &str = "【定时任务 / 心跳任务策略】\n\
- 如用户要求在明确时间执行，请使用常规定时任务（daily / weekly / workday / trading_day / holiday / once）。\n\
- 如用户要求“当某个条件满足时提醒我”，但没有给出具体时刻，例如股价阈值、公告事件、新闻条件、财报条件等，这类任务默认不应伪装成 daily 09:00。\n\
- 对这种无明确时刻的条件型任务，必须先询问用户是否要创建“心跳检测”任务；心跳任务会每 30 分钟检查一次条件。\n\
- 只有在用户明确同意后，才创建 repeat=heartbeat 的任务；heartbeat 任务建议带上 heartbeat 标签。\n\
- 用户询问“我的所有定时任务”时，应把 heartbeat 任务也视为任务列表的一部分一并说明。";
pub const DEFAULT_COMPANY_PROFILE_POLICY: &str = "【公司画像 / 长期跟踪策略】\n\
- 若用户正在系统研究某家公司，且工具 company_profile 可用，应优先检查是否已有画像。\n\
- 若尚无画像，不要自动创建；应先提示用户是否要为该公司建立长期画像。\n\
- 用户确认后，才允许创建画像，并将当前研究结论写入主画像。\n\
- 若画像已存在，后续研究应优先参考已有画像，并在出现实质新增事实时追加事件；只有长期判断变化明显时才回写主画像 section。\n\
- 画像不仅要保留“当前结论”，还应尽量保留“为什么这么判断”、关键证据、来源与本轮研究路径；若当前没有独立 research note 存储层，应把必要的 why / evidence / research trail 写入事件正文。\n\
- 主画像应优先维护 Thesis、关键经营指标、估值框架、风险台账与证伪条件；事件更新应围绕 thesis change log，而不是价格噪音。\n\
- 公司画像服务于长期基本面跟踪，不应用于日内盯盘、高频价格提醒或直接交易指令。";

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
        self.static_system.clone()
    }

    pub fn compose_user_input(&self, user_input: &str) -> String {
        let mut sections = Vec::new();

        if let Some(context) = self
            .conversation_context
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
        {
            sections.push(context.to_string());
        }

        sections.push(format!("【本轮用户输入】\n{}", user_input.trim()));

        if let Some(session_context) =
            Some(self.session_context.trim()).filter(|value| !value.is_empty())
        {
            sections.push(session_context.to_string());
        }

        sections.join("\n\n")
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
        \n如需使用管理员技能，请先调用 skill_tool(skill_name=\"hone_admin\") 获取完整操作指引。"
    )
}

pub fn build_prompt_bundle(
    config: &HoneConfig,
    storage: &SessionStorage,
    channel: &str,
    session_id: &str,
    _prompt_state: &SessionPromptState,
    options: &PromptOptions,
) -> PromptBundle {
    let now = hone_core::beijing_now();
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
    static_system.push_str("\n\n");
    static_system.push_str(DEFAULT_COMPANY_PROFILE_POLICY);

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
        now.format("%Y-%m-%d %H:%M:%S"),
        now.format("%Y-%m-%d"),
        now.format("%Y"),
        session_id
    );

    let conversation_context = storage
        .load_session(session_id)
        .ok()
        .flatten()
        .and_then(|session| {
            hone_memory::latest_compact_summary(&session.messages)
                .map(|message| message.content.trim().to_string())
                .or_else(|| {
                    session
                        .summary
                        .map(|summary| format!("【历史会话总结】\n{}", summary.content.trim()))
                })
        })
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

    #[test]
    fn dynamic_session_context_stays_out_of_system_prompt_prefix() {
        let data_dir = std::env::temp_dir().join(format!(
            "hone-prompt-dynamic-{}-{}",
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
            "feishu",
            "session-demo",
            &prompt_state,
            &PromptOptions::default(),
        );

        assert!(!bundle.system_prompt().contains("【Session 上下文】"));
        assert!(
            bundle
                .compose_user_input("你好")
                .contains("【Session 上下文】")
        );
        assert!(
            bundle
                .compose_user_input("你好")
                .contains("【本轮用户输入】")
        );

        let _ = fs::remove_dir_all(&data_dir);
    }

    #[test]
    fn session_context_is_appended_after_current_turn_input() {
        let data_dir = std::env::temp_dir().join(format!(
            "hone-prompt-session-order-{}-{}",
            std::process::id(),
            hone_core::beijing_now()
                .timestamp_nanos_opt()
                .unwrap_or_default()
        ));
        fs::create_dir_all(&data_dir).expect("session storage dir should init");
        let storage = SessionStorage::new(data_dir.join("sessions"));
        let mut config = HoneConfig::default();
        config.agent.system_prompt = "你是 Hone。".to_string();

        let bundle = build_prompt_bundle(
            &config,
            &storage,
            "feishu",
            "session-demo",
            &SessionPromptState::default(),
            &PromptOptions::default(),
        );

        let composed = bundle.compose_user_input("今天公布的非农数据怎么样");
        let input_pos = composed
            .find("【本轮用户输入】")
            .expect("user input section");
        let session_pos = composed
            .find("【Session 上下文】")
            .expect("session section");

        assert!(input_pos < session_pos);

        let _ = fs::remove_dir_all(&data_dir);
    }

    #[test]
    fn session_context_uses_current_time_instead_of_frozen_prompt_time() {
        let data_dir = std::env::temp_dir().join(format!(
            "hone-prompt-current-time-{}-{}",
            std::process::id(),
            hone_core::beijing_now()
                .timestamp_nanos_opt()
                .unwrap_or_default()
        ));
        fs::create_dir_all(&data_dir).expect("session storage dir should init");
        let storage = SessionStorage::new(data_dir.join("sessions"));
        let mut config = HoneConfig::default();
        config.agent.system_prompt = "你是 Hone。".to_string();
        let prompt_state = SessionPromptState {
            frozen_time_beijing: "2026-03-17T22:01:00+08:00".to_string(),
        };

        let bundle = build_prompt_bundle(
            &config,
            &storage,
            "discord",
            "session-demo",
            &prompt_state,
            &PromptOptions::default(),
        );

        let composed = bundle.compose_user_input("今天公布的非农数据怎么样");
        assert!(!composed.contains("2026-03-17 22:01:00"));
        assert!(composed.contains(&hone_core::beijing_now().format("%Y-%m-%d").to_string()));

        let _ = fs::remove_dir_all(&data_dir);
    }

    #[test]
    fn prompt_options_append_admin_language_and_extra_sections() {
        let data_dir = std::env::temp_dir().join(format!(
            "hone-prompt-options-{}-{}",
            std::process::id(),
            hone_core::beijing_now()
                .timestamp_nanos_opt()
                .unwrap_or_default()
        ));
        fs::create_dir_all(&data_dir).expect("session storage dir should init");
        let storage = SessionStorage::new(data_dir.join("sessions"));
        let mut config = HoneConfig::default();
        config.agent.system_prompt = "你是 Hone。".to_string();

        let bundle = build_prompt_bundle(
            &config,
            &storage,
            "discord",
            "session-demo",
            &SessionPromptState::default(),
            &PromptOptions {
                is_admin: true,
                admin_prompt: Some("【管理员覆写】请先确认影响范围。".to_string()),
                privacy_guard: Some(DEFAULT_GROUP_PRIVACY_GUARD.to_string()),
                model_hint: Some("gpt-5.4".to_string()),
                force_chinese: true,
                extra_sections: vec![
                    "【附加规则】先给结论再展开。".to_string(),
                    "   ".to_string(),
                ],
                include_format_guidance: true,
            },
        );

        let system = bundle.system_prompt();
        assert!(system.contains("【管理员覆写】请先确认影响范围。"));
        assert!(system.contains(DEFAULT_GROUP_PRIVACY_GUARD));
        assert!(system.contains("【基础模型】gpt-5.4。"));
        assert!(system.contains("【语言要求】必须全程以中文回复"));
        assert!(system.contains("【附加规则】先给结论再展开。"));
        assert!(system.contains("【输出格式-Discord】"));

        let _ = fs::remove_dir_all(&data_dir);
    }

    #[test]
    fn prompt_can_skip_channel_format_guidance() {
        let data_dir = std::env::temp_dir().join(format!(
            "hone-prompt-no-format-{}-{}",
            std::process::id(),
            hone_core::beijing_now()
                .timestamp_nanos_opt()
                .unwrap_or_default()
        ));
        fs::create_dir_all(&data_dir).expect("session storage dir should init");
        let storage = SessionStorage::new(data_dir.join("sessions"));
        let mut config = HoneConfig::default();
        config.agent.system_prompt = "你是 Hone。".to_string();

        let bundle = build_prompt_bundle(
            &config,
            &storage,
            "telegram",
            "session-demo",
            &SessionPromptState::default(),
            &PromptOptions {
                include_format_guidance: false,
                ..PromptOptions::default()
            },
        );

        assert!(!bundle.system_prompt().contains("【输出格式-Telegram】"));

        let _ = fs::remove_dir_all(&data_dir);
    }
}
