use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Component, Path, PathBuf};

use chrono::{DateTime, NaiveDate, Utc};

use super::types::{default_profile_status, default_thesis_impact};
use super::{
    AppendEventInput, CompanyProfileEventDocument, IndustryTemplate, ProfileEventMetadata,
    ProfileMetadata, TrackingConfig,
};

pub(super) fn parse_frontmatter(content: &str) -> Result<(String, String), String> {
    if !content.starts_with("---\n") {
        return Err("缺少 frontmatter".to_string());
    }
    let remainder = &content[4..];
    let Some(end) = remainder.find("\n---\n") else {
        return Err("frontmatter 未正确结束".to_string());
    };
    let frontmatter = remainder[..end].to_string();
    let body = remainder[end + 5..].to_string();
    Ok((frontmatter, body))
}

pub(super) fn parse_profile_markdown(
    content: &str,
) -> Result<(ProfileMetadata, Vec<(String, String)>), String> {
    let (frontmatter, body) = parse_frontmatter(content)?;
    let metadata: ProfileMetadata = serde_yaml::from_str(&frontmatter)
        .map_err(|err| format!("解析画像 frontmatter 失败: {err}"))?;
    let (sections, _) = parse_profile_sections(&body);
    Ok((metadata, sections))
}

pub(super) fn parse_profile_metadata_relaxed(
    profile_id: &str,
    content: &str,
    updated_at: Option<String>,
) -> Result<ProfileMetadata, String> {
    if let Ok((metadata, _)) = parse_profile_markdown(content) {
        return Ok(metadata);
    }

    Ok(infer_profile_metadata(
        profile_id,
        &strip_frontmatter_for_transfer(content),
        updated_at,
    ))
}

pub(super) fn parse_event_markdown(
    id: &str,
    filename: &str,
    content: &str,
) -> Result<CompanyProfileEventDocument, String> {
    let (frontmatter, body) = parse_frontmatter(content)?;
    let metadata: ProfileEventMetadata = serde_yaml::from_str(&frontmatter)
        .map_err(|err| format!("解析事件 frontmatter 失败: {err}"))?;
    let title = extract_title_from_markdown(&body, "未命名事件");
    Ok(CompanyProfileEventDocument {
        id: id.to_string(),
        filename: filename.to_string(),
        title,
        metadata,
        markdown: content.to_string(),
    })
}

pub(super) fn parse_event_markdown_relaxed(
    id: &str,
    filename: &str,
    content: &str,
    updated_at: Option<String>,
) -> Result<CompanyProfileEventDocument, String> {
    if let Ok(document) = parse_event_markdown(id, filename, content) {
        return Ok(document);
    }

    let body = strip_frontmatter_for_transfer(content);
    Ok(CompanyProfileEventDocument {
        id: id.to_string(),
        filename: filename.to_string(),
        title: extract_title_from_markdown(&body, id),
        metadata: infer_event_metadata(filename, updated_at),
        markdown: content.to_string(),
    })
}

pub(super) fn parse_profile_markdown_for_transfer(
    profile_id: &str,
    content: &str,
    updated_at: Option<String>,
) -> Result<(ProfileMetadata, String), String> {
    if let Ok((metadata, _)) = parse_profile_markdown(content) {
        return Ok((metadata, content.to_string()));
    }

    let body = strip_frontmatter_for_transfer(content);
    let metadata = infer_profile_metadata(profile_id, &body, updated_at);
    let markdown = render_profile_markdown(&metadata, &[], &body)
        .map_err(|err| format!("生成导出画像 frontmatter 失败: {err}"))?;
    Ok((metadata, markdown))
}

pub(super) fn parse_event_markdown_for_transfer(
    id: &str,
    filename: &str,
    content: &str,
    updated_at: Option<String>,
) -> Result<CompanyProfileEventDocument, String> {
    if let Ok(document) = parse_event_markdown(id, filename, content) {
        return Ok(document);
    }

    let body = strip_frontmatter_for_transfer(content);
    let metadata = infer_event_metadata(filename, updated_at);
    let frontmatter = serde_yaml::to_string(&metadata)
        .map_err(|err| format!("生成导出事件 frontmatter 失败: {err}"))?;
    let markdown = format!("---\n{}---\n\n{}", frontmatter, body.trim());
    Ok(CompanyProfileEventDocument {
        id: id.to_string(),
        filename: filename.to_string(),
        title: extract_title_from_markdown(&body, id),
        metadata,
        markdown,
    })
}

fn strip_frontmatter_for_transfer(content: &str) -> String {
    parse_frontmatter(content)
        .map(|(_, body)| body)
        .unwrap_or_else(|_| content.to_string())
}

fn infer_stock_code_from_profile_id(profile_id: &str) -> String {
    let normalized = normalize_stock_code(profile_id);
    if !normalized.is_empty() && normalized == profile_id.trim() {
        normalized
    } else {
        String::new()
    }
}

fn infer_profile_metadata(
    profile_id: &str,
    body: &str,
    updated_at: Option<String>,
) -> ProfileMetadata {
    let updated_at = updated_at.unwrap_or_else(|| Utc::now().to_rfc3339());
    ProfileMetadata {
        company_name: extract_title_from_markdown(body, profile_id),
        stock_code: infer_stock_code_from_profile_id(profile_id),
        aliases: Vec::new(),
        sector: String::new(),
        industry_template: IndustryTemplate::General,
        status: default_profile_status(),
        tracking: TrackingConfig::default(),
        created_at: updated_at.clone(),
        updated_at,
        last_reviewed_at: None,
    }
}

fn infer_event_type_from_filename(filename: &str) -> String {
    let stem = filename.trim_end_matches(".md");
    let candidate = if stem.len() > 11 && stem.as_bytes().get(10) == Some(&b'-') {
        stem[11..].split('-').next().unwrap_or("update")
    } else {
        "update"
    };
    let sanitized = sanitize_id(candidate);
    if sanitized.is_empty() {
        "update".to_string()
    } else {
        sanitized
    }
}

fn infer_event_metadata(filename: &str, updated_at: Option<String>) -> ProfileEventMetadata {
    let updated_at = updated_at.unwrap_or_else(|| Utc::now().to_rfc3339());
    ProfileEventMetadata {
        event_type: infer_event_type_from_filename(filename),
        occurred_at: infer_occurred_at_from_filename(filename, &updated_at),
        captured_at: updated_at,
        thesis_impact: default_thesis_impact(),
        changed_sections: Vec::new(),
        refs: Vec::new(),
    }
}

fn infer_occurred_at_from_filename(filename: &str, fallback: &str) -> String {
    let stem = filename.trim_end_matches(".md");
    if stem.len() >= 10 {
        let candidate = &stem[..10];
        if NaiveDate::parse_from_str(candidate, "%Y-%m-%d").is_ok() {
            return format!("{candidate}T00:00:00Z");
        }
    }
    fallback.to_string()
}

pub(super) fn extract_title_from_markdown(body: &str, fallback: &str) -> String {
    for line in body.lines() {
        if let Some(rest) = line.strip_prefix("# ") {
            return rest.trim().to_string();
        }
    }
    fallback.to_string()
}

pub(super) fn file_modified_at_rfc3339(path: &Path) -> Option<String> {
    let modified = fs::metadata(path).ok()?.modified().ok()?;
    let updated_at: DateTime<Utc> = modified.into();
    Some(updated_at.to_rfc3339())
}

pub(super) fn render_profile_markdown(
    metadata: &ProfileMetadata,
    _sections: &[(String, String)],
    body: &str,
) -> Result<String, serde_yaml::Error> {
    let frontmatter = serde_yaml::to_string(metadata)?;
    Ok(format!("---\n{}---\n\n{}", frontmatter, body.trim()))
}

pub(super) fn render_event_markdown(
    title: &str,
    metadata: &ProfileEventMetadata,
    input: &AppendEventInput,
) -> String {
    let frontmatter =
        serde_yaml::to_string(metadata).unwrap_or_else(|_| "event_type: unknown\n".to_string());
    format!(
        "---\n{}---\n\n# {}\n\n## 发生了什么\n{}\n\n## 为什么重要\n{}\n\n## 影响哪些画像 section\n{}\n\n## 对 thesis 的影响\n{}\n\n## 证据与来源\n{}\n\n## 本轮研究路径\n{}\n\n## 需要继续跟踪什么\n{}\n",
        frontmatter,
        title.trim(),
        fallback_markdown(&input.what_happened),
        fallback_markdown(&input.why_it_matters),
        render_list_or_placeholder(&input.changed_sections, "暂无"),
        fallback_markdown(&input.thesis_effect),
        render_evidence_markdown(&input.evidence, &input.refs),
        fallback_markdown(&input.research_log),
        fallback_markdown(&input.follow_up),
    )
}

pub(super) fn create_profile_body(sections: &[(String, String)]) -> String {
    sections
        .iter()
        .map(|(title, content)| format!("## {}\n{}\n", title, content.trim()))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn fallback_markdown(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        "暂无".to_string()
    } else {
        trimmed.to_string()
    }
}

fn render_list_or_placeholder(values: &[String], placeholder: &str) -> String {
    if values.is_empty() {
        placeholder.to_string()
    } else {
        values
            .iter()
            .map(|value| format!("- {}", value.trim()))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn render_evidence_markdown(evidence: &str, refs: &[String]) -> String {
    let mut blocks = Vec::new();
    if !evidence.trim().is_empty() {
        blocks.push(evidence.trim().to_string());
    }
    if !refs.is_empty() {
        blocks.push(
            refs.iter()
                .map(|value| format!("- {}", value.trim()))
                .collect::<Vec<_>>()
                .join("\n"),
        );
    }
    if blocks.is_empty() {
        "暂无".to_string()
    } else {
        blocks.join("\n\n")
    }
}

pub(super) fn parse_profile_sections(content: &str) -> (Vec<(String, String)>, Vec<String>) {
    let body = if content.starts_with("---\n") {
        parse_frontmatter(content)
            .map(|(_, body)| body)
            .unwrap_or_else(|_| content.to_string())
    } else {
        content.to_string()
    };

    let mut sections = Vec::new();
    let mut extra_lines = Vec::new();
    let mut current_title: Option<String> = None;
    let mut current_lines: Vec<String> = Vec::new();

    for line in body.lines() {
        if let Some(rest) = line.strip_prefix("## ") {
            if let Some(title) = current_title.take() {
                sections.push((title, current_lines.join("\n").trim().to_string()));
                current_lines.clear();
            }
            current_title = Some(rest.trim().to_string());
        } else if current_title.is_some() {
            current_lines.push(line.to_string());
        } else if !line.trim().is_empty() {
            extra_lines.push(line.to_string());
        }
    }

    if let Some(title) = current_title.take() {
        sections.push((title, current_lines.join("\n").trim().to_string()));
    }

    (sections, extra_lines)
}

pub(super) fn build_initial_sections(
    template: &IndustryTemplate,
    overrides: &BTreeMap<String, String>,
) -> Vec<(String, String)> {
    let mut sections = base_profile_sections(template);
    let mut index = HashMap::new();
    for (position, (title, _)) in sections.iter().enumerate() {
        index.insert(title.clone(), position);
    }

    for (title, content) in overrides {
        let normalized_title = title.trim().to_string();
        if normalized_title.is_empty() {
            continue;
        }
        if let Some(position) = index.get(&normalized_title).copied() {
            sections[position].1 = content.trim().to_string();
        } else {
            sections.push((normalized_title.clone(), content.trim().to_string()));
            index.insert(normalized_title, sections.len() - 1);
        }
    }
    sections
}

pub(super) fn base_profile_sections(template: &IndustryTemplate) -> Vec<(String, String)> {
    let mut sections = vec![
        (
            "投资主张".to_string(),
            "待补充：这家公司当前最核心的长期判断、为何值得跟踪，以及现阶段最重要的一句话结论。".to_string(),
        ),
        (
            "Thesis".to_string(),
            "待补充：当前多空要点、判断为什么成立、最关键的 3-5 个观察变量，以及什么事实会证伪或改写 thesis。".to_string(),
        ),
        (
            "商业模式".to_string(),
            "待补充：公司如何赚钱、收入结构、成本结构、单位经济与周期性特征。".to_string(),
        ),
        (
            "行业与竞争格局".to_string(),
            "待补充：行业空间、竞争者、替代品、上下游议价权、进入壁垒与监管环境。".to_string(),
        ),
        (
            "护城河".to_string(),
            "待补充：品牌、网络效应、切换成本、规模优势、成本优势、渠道控制或牌照壁垒，并标注 moat 趋势。".to_string(),
        ),
        (
            "管理层与治理".to_string(),
            "待补充：创始人/高管团队、激励机制、资本配置记录、治理质量与对外沟通可信度。".to_string(),
        ),
        (
            "财务质量".to_string(),
            "待补充：增长质量、利润率、ROIC、现金流、负债结构、再投资效率与会计质量。".to_string(),
        ),
        (
            "资本配置".to_string(),
            "待补充：分红、回购、并购、研发、产能投资、去杠杆等动作是否提升长期每股价值。".to_string(),
        ),
        (
            "关键经营指标".to_string(),
            template_operating_metrics_markdown(template),
        ),
        (
            "估值框架".to_string(),
            "待补充：估值方法、关键假设、敏感性、可比对象和当前估值区间。".to_string(),
        ),
        (
            "风险台账".to_string(),
            "待补充：监管、技术替代、客户集中、库存、地缘政治、融资、治理失误或财务失真等风险，并单列 disconfirming evidence。".to_string(),
        ),
        (
            "关键跟踪清单".to_string(),
            template_tracking_markdown(template),
        ),
        (
            "未决问题".to_string(),
            "待补充：当前还未验证、但会显著影响 thesis 的问题列表。".to_string(),
        ),
        (
            "行业模板附录".to_string(),
            template_appendix_markdown(template),
        ),
    ];

    if matches!(template, IndustryTemplate::General) {
        sections.retain(|(title, _)| title != "行业模板附录");
    }
    sections
}

fn template_tracking_markdown(template: &IndustryTemplate) -> String {
    match template {
        IndustryTemplate::General => {
            "- 季度至少 review 一次\n- 财报/业绩会后必更\n- 重大事件（管理层、监管、资本配置、行业格局变化）触发更新\n- 估值进入关键区间时重看 thesis / 赔率 / 风险回报".to_string()
        }
        IndustryTemplate::Saas => {
            "- 财报后核对 ARR / RPO / NRR / 留存 / deferred revenue 的方向是否改变 thesis\n- 观察 seat expansion、产品渗透与销售效率是否改善\n- 指引变化时同步检查估值假设与可持续增长判断".to_string()
        }
        IndustryTemplate::SemiconductorHardware => {
            "- 跟踪 ASP、良率、产能利用率、库存周期与 capex\n- 设计 win / 产品 mix 变化若影响中期盈利能力，应更新 thesis\n- 行业景气和客户备货节奏变化时，重看估值框架和风险台账".to_string()
        }
        IndustryTemplate::Consumer => {
            "- 跟踪同店、复购率、客单价、渠道库存、促销强度\n- 品牌溢价与新品表现若出现拐点，应检查护城河与管理层判断\n- 观察库存/折扣是否正在侵蚀长期盈利质量".to_string()
        }
        IndustryTemplate::IndustrialDefense => {
            "- 跟踪订单、积压订单、book-to-bill、交付节奏、产能利用率\n- 大客户签约/流失、项目延误、预算变化应写入事件并重看 thesis\n- 若订单质量或兑现节奏恶化，更新风险台账与估值假设".to_string()
        }
        IndustryTemplate::Financials => {
            "- 跟踪净息差、不良、拨备、资本充足率、负债成本\n- 若风险成本、资产质量或资本压力变化，应更新财务质量与 thesis\n- 利率环境或监管变化后，重看估值框架和核心风险".to_string()
        }
    }
}

fn template_operating_metrics_markdown(template: &IndustryTemplate) -> String {
    match template {
        IndustryTemplate::General => {
            "待补充：列出这家公司真正决定长期判断的 3-7 个经营指标，并说明每个指标为什么重要。"
                .to_string()
        }
        IndustryTemplate::Saas => {
            "- ARR\n- RPO / cRPO\n- NRR / 客户留存\n- seat expansion / 产品渗透\n- deferred revenue"
                .to_string()
        }
        IndustryTemplate::SemiconductorHardware => {
            "- ASP\n- 良率\n- 产能与 capex\n- 库存天数 / 渠道库存\n- 设计 win 与产品 mix"
                .to_string()
        }
        IndustryTemplate::Consumer => {
            "- 同店销售\n- 复购率\n- 客单价\n- 渠道库存\n- 品牌溢价与促销强度".to_string()
        }
        IndustryTemplate::IndustrialDefense => {
            "- 新签订单\n- 积压订单\n- book-to-bill\n- 交付节奏\n- 产能利用率".to_string()
        }
        IndustryTemplate::Financials => {
            "- 净息差\n- 不良率\n- 拨备覆盖率\n- 资本充足率\n- 负债成本".to_string()
        }
    }
}

fn template_appendix_markdown(template: &IndustryTemplate) -> String {
    match template {
        IndustryTemplate::General => String::new(),
        IndustryTemplate::Saas => {
            "本模板重点关注 SaaS 公司常见核心变量：ARR、RPO、NRR、留存、产品渗透、deferred revenue。".to_string()
        }
        IndustryTemplate::SemiconductorHardware => {
            "本模板重点关注半导体/硬件公司常见核心变量：ASP、良率、产能、库存、设计 win、capex。".to_string()
        }
        IndustryTemplate::Consumer => {
            "本模板重点关注消费公司常见核心变量：同店、复购、客单价、渠道库存、品牌溢价。".to_string()
        }
        IndustryTemplate::IndustrialDefense => {
            "本模板重点关注工业/国防公司常见核心变量：订单、积压订单、book-to-bill、交付、产能。".to_string()
        }
        IndustryTemplate::Financials => {
            "本模板重点关注金融公司常见核心变量：净息差、不良、拨备、资本充足率、负债成本。".to_string()
        }
    }
}

pub(super) fn normalize_stock_code(value: &str) -> String {
    value.trim().to_uppercase()
}

pub(super) fn normalize_company_name(value: &str) -> String {
    value
        .trim()
        .chars()
        .flat_map(|ch| ch.to_lowercase())
        .collect::<String>()
}

pub(super) fn sanitize_id(value: &str) -> String {
    let sanitized = value
        .trim()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches(|ch| matches!(ch, '-' | '.'))
        .to_string();
    validate_storage_component(&sanitized).unwrap_or_default()
}

pub(super) fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut previous_dash = false;
    for ch in value.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            previous_dash = false;
        } else if ch.is_alphanumeric() {
            slug.push(ch);
            previous_dash = false;
        } else if !previous_dash {
            slug.push('-');
            previous_dash = true;
        }
    }
    slug.trim_matches('-').to_string()
}

pub(super) fn validate_storage_component(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut components = Path::new(trimmed).components();
    let component = match components.next() {
        Some(Component::Normal(component)) => component.to_str()?.to_string(),
        _ => return None,
    };
    if components.next().is_some() {
        return None;
    }
    if component.is_empty() {
        None
    } else {
        Some(component)
    }
}

pub(super) fn safe_component_join(root: &Path, value: &str) -> Option<PathBuf> {
    let sanitized = sanitize_id(value);
    let component = validate_storage_component(&sanitized)?;
    Some(root.join(component))
}

pub(super) fn normalize_event_date(value: &str) -> String {
    if let Ok(parsed) = DateTime::parse_from_rfc3339(value.trim()) {
        parsed.format("%Y-%m-%d").to_string()
    } else {
        value.trim().chars().take(10).collect::<String>()
    }
}
