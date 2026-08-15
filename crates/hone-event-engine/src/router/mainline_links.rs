//! 跨票主线关联(2026-08,推送体检 item 6)。
//!
//! 用户的 `mainline_by_ticker` 常常互相引用:CAI 的主线写「把它当作 Tempus
//! 的深度对照组」,LITE 的主线拿 AAOI 对比纯度。推送 TEM 事件时用户应该被
//! 提醒「你有一条 CAI 主线以它为对照」——这层关联现在从不出现。
//!
//! 匹配规则(在真实主线数据上验证,零误报):
//! - **规则 A**:其他主线文本按词边界直接提到本 ticker(如 LITE 主线提 AAOI);
//! - **规则 B**:本标的与其他标的的主线共享 TitleCase 专名 token(如两边都写
//!   "Tempus")。全大写缩写(NAND/HBM/GPU/NVIDIA)天然被 TitleCase 规则排除,
//!   避免"都是 AI 股"级别的假关联。

use std::collections::{HashMap, HashSet};

/// 单条跨票关联:`ticker` 的主线与当前事件标的相关,`excerpt` 是提及处的子句。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MainlineCrossLink {
    pub ticker: String,
    pub excerpt: String,
}

const MAX_LINKS: usize = 2;
const MAX_EXCERPT_CHARS: usize = 80;

/// 计算事件标的与其他主线的关联。`symbol` 大小写不敏感;返回按 ticker 排序、
/// 最多 [`MAX_LINKS`] 条。
pub(crate) fn mainline_cross_links(
    symbol: &str,
    mainline_by_ticker: &HashMap<String, String>,
) -> Vec<MainlineCrossLink> {
    let symbol_upper = symbol.trim().to_ascii_uppercase();
    if symbol_upper.is_empty() {
        return Vec::new();
    }
    let own_tokens: HashSet<String> = mainline_by_ticker
        .iter()
        .find(|(ticker, _)| ticker.eq_ignore_ascii_case(&symbol_upper))
        .map(|(_, text)| titlecase_name_tokens(text))
        .unwrap_or_default();

    let mut links: Vec<MainlineCrossLink> = mainline_by_ticker
        .iter()
        .filter(|(ticker, _)| !ticker.eq_ignore_ascii_case(&symbol_upper))
        .filter_map(|(ticker, text)| {
            // 规则 A:直接提到本 ticker
            if contains_word(text, &symbol_upper) {
                return Some(MainlineCrossLink {
                    ticker: ticker.to_ascii_uppercase(),
                    excerpt: excerpt_around(text, &symbol_upper),
                });
            }
            // 规则 B:与本标的主线共享 TitleCase 专名
            let shared = titlecase_name_tokens(text)
                .intersection(&own_tokens)
                .next()
                .cloned()?;
            Some(MainlineCrossLink {
                ticker: ticker.to_ascii_uppercase(),
                excerpt: excerpt_around(text, &shared),
            })
        })
        .collect();
    links.sort_by(|a, b| a.ticker.cmp(&b.ticker));
    links.truncate(MAX_LINKS);
    links
}

/// 文本中提取 TitleCase 拉丁专名 token(首字母大写 + ≥4 个小写字母,如
/// "Tempus" / "Coherent" / "Neutron")。全大写缩写与普通中文不命中。
fn titlecase_name_tokens(text: &str) -> HashSet<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut tokens = HashSet::new();
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i].is_ascii_uppercase() && (i == 0 || !chars[i - 1].is_ascii_alphanumeric()) {
            let start = i;
            i += 1;
            while i < chars.len() && chars[i].is_ascii_lowercase() {
                i += 1;
            }
            let boundary_ok = i >= chars.len() || !chars[i].is_ascii_alphanumeric();
            if boundary_ok && i - start >= 5 {
                tokens.insert(chars[start..i].iter().collect());
            }
        } else {
            i += 1;
        }
    }
    tokens
}

/// `needle` 是否以词边界形式出现(邻字符非 ASCII 字母数字;中文邻接视为边界)。
fn contains_word(text: &str, needle: &str) -> bool {
    let text_chars: Vec<char> = text.chars().collect();
    let needle_chars: Vec<char> = needle.chars().collect();
    if needle_chars.is_empty() || text_chars.len() < needle_chars.len() {
        return false;
    }
    for start in 0..=(text_chars.len() - needle_chars.len()) {
        let candidate = &text_chars[start..start + needle_chars.len()];
        if !candidate
            .iter()
            .zip(&needle_chars)
            .all(|(a, b)| a.eq_ignore_ascii_case(b))
        {
            continue;
        }
        let left_ok = start == 0 || !text_chars[start - 1].is_ascii_alphanumeric();
        let end = start + needle_chars.len();
        let right_ok = end >= text_chars.len() || !text_chars[end].is_ascii_alphanumeric();
        if left_ok && right_ok {
            return true;
        }
    }
    false
}

/// 取包含 `needle` 的子句(按 。；;!？ 切分),超长截断到 [`MAX_EXCERPT_CHARS`]。
fn excerpt_around(text: &str, needle: &str) -> String {
    let clause = text
        .split(['。', '；', ';', '!', '！', '？'])
        .find(|clause| contains_word(clause, needle) || clause.contains(needle))
        .unwrap_or(text)
        .trim();
    let mut chars = clause.chars();
    let head: String = chars.by_ref().take(MAX_EXCERPT_CHARS).collect();
    if chars.next().is_some() {
        format!("{head}…")
    } else {
        head
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn real_mainlines() -> HashMap<String, String> {
        // 真实用户主线的关键片段(2026-08-15 快照)
        let mut m = HashMap::new();
        m.insert(
            "LITE".into(),
            "持有LITE核心看AI光互连里OCS和CPO从概念转向订单验证；它在纯度上介于Coherent和AAOI之间，胜在datacenter业务聚焦。".into(),
        );
        m.insert(
            "CAI".into(),
            "我长期跟踪CAI是把它当作Tempus的深度对照组，核心看它能否把WES/WTS带来的数据库持续转化为recurring收入。".into(),
        );
        m.insert(
            "TEM".into(),
            "我把 Tempus 看作靠诊断业务持续抓取多模态临床数据、再二次变现的混合平台。".into(),
        );
        m.insert(
            "MU".into(),
            "Micron 已从 AI 存储跟随者转为 HBM 第二梯队核心受益者，关键跟踪变量是 NVIDIA 平台份额进展。".into(),
        );
        m.insert(
            "NBIS".into(),
            "持有NBIS的核心逻辑是高端GPU算力供不应求，NVIDIA合作与长期合同已把需求可见度拉高。"
                .into(),
        );
        m
    }

    /// 规则 A:LITE 主线直接提 AAOI → AAOI 事件带 LITE 关联。
    #[test]
    fn ticker_mention_links_aaoi_to_lite() {
        let links = mainline_cross_links("AAOI", &real_mainlines());
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].ticker, "LITE");
        assert!(links[0].excerpt.contains("AAOI"), "{}", links[0].excerpt);
    }

    /// 规则 B:CAI 与 TEM 主线共享 "Tempus" → TEM 事件带 CAI 关联(反向同理)。
    #[test]
    fn shared_titlecase_name_links_tem_and_cai() {
        let links = mainline_cross_links("TEM", &real_mainlines());
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].ticker, "CAI");
        assert!(links[0].excerpt.contains("Tempus"), "{}", links[0].excerpt);

        let reverse = mainline_cross_links("CAI", &real_mainlines());
        assert_eq!(reverse.len(), 1);
        assert_eq!(reverse[0].ticker, "TEM");
    }

    /// 全大写缩写(NVIDIA/GPU/HBM)不构成关联:MU 与 NBIS 都提 NVIDIA,
    /// 但那是行业共性词,不是主线互指。
    #[test]
    fn all_caps_acronyms_do_not_create_spurious_links() {
        assert!(mainline_cross_links("MU", &real_mainlines()).is_empty());
        assert!(mainline_cross_links("NBIS", &real_mainlines()).is_empty());
    }

    /// ticker 子串不跨词边界误命中:BE 不应命中 "backlog" / "Bloomberg" 类词。
    #[test]
    fn ticker_substring_requires_word_boundary() {
        let mut m = HashMap::new();
        m.insert(
            "X".into(),
            "关注backlog与BEV产业,顺带看Bloomberg报道。".into(),
        );
        assert!(mainline_cross_links("BE", &m).is_empty());
        m.insert("Y".into(), "对比 BE 的燃料电池路线。".into());
        let links = mainline_cross_links("BE", &m);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].ticker, "Y");
    }

    #[test]
    fn excerpt_is_clause_scoped_and_truncated() {
        let mut m = HashMap::new();
        let long_tail = "这一段完全无关的超长内容".repeat(20);
        m.insert(
            "Z".into(),
            format!("第一句无关；这里提到 TGT 做对照；{long_tail}"),
        );
        let links = mainline_cross_links("TGT", &m);
        assert_eq!(links[0].excerpt, "这里提到 TGT 做对照");
    }
}
