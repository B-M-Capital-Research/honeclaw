//! ApiKeyPool — 多 API Key 容错池
//!
//! 统一管理一组 API Key，支持：
//! - 自动过滤空 key
//! - 顺序 fallback：第一个 key 失败时依次尝试下一个
//! - 向后兼容：合并旧版 `api_key`（单值）和新版 `api_keys`（列表）
//!
//! ## 使用方式
//!
//! ```rust
//! let pool = ApiKeyPool::merged(&config.fmp.api_key, &config.fmp.api_keys);
//! if pool.is_empty() {
//!     return Ok(json!({ "error": "未配置 API Key" }));
//! }
//! let mut last_err = None;
//! for key in pool.keys() {
//!     match call_api(key).await {
//!         Ok(result) => return Ok(result),
//!         Err(e) => last_err = Some(e),
//!     }
//! }
//! Err(last_err.unwrap())
//! ```

use std::collections::HashSet;

/// 多 API Key 容错池
///
/// 内部存储去重、去空后的有效 Key 列表，调用方通过 `keys()` 遍历并实现 fallback 逻辑。
#[derive(Debug, Clone, Default)]
pub struct ApiKeyPool {
    keys: Vec<String>,
}

impl ApiKeyPool {
    /// 从多个 key 创建 Pool（自动过滤空 key、保留顺序、去重）
    pub fn new(keys: impl IntoIterator<Item = String>) -> Self {
        let mut seen = HashSet::new();
        let keys: Vec<String> = keys
            .into_iter()
            .filter(|k| !k.is_empty() && seen.insert(k.clone()))
            .collect();
        Self { keys }
    }

    /// 从单个 key 创建 Pool
    pub fn from_single(key: impl Into<String>) -> Self {
        let key = key.into();
        if key.is_empty() {
            Self { keys: vec![] }
        } else {
            Self { keys: vec![key] }
        }
    }

    /// 合并旧版 `api_key`（单值，向后兼容）和新版 `api_keys`（列表），去重、去空
    ///
    /// 优先级：`primary` 排在 `extras` 之前（即旧版单值的优先级更高）
    pub fn merged(primary: &str, extras: &[String]) -> Self {
        let mut seen = HashSet::new();
        let mut keys = Vec::new();
        for k in std::iter::once(primary.to_string()).chain(extras.iter().cloned()) {
            if !k.is_empty() && seen.insert(k.clone()) {
                keys.push(k);
            }
        }
        Self { keys }
    }

    /// Pool 是否为空（无可用 Key）
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// 获取所有有效 Key（只读引用）
    pub fn keys(&self) -> &[String] {
        &self.keys
    }

    /// 获取第一个有效 Key（用于简单单 Key 场景或作为代表值）
    pub fn first(&self) -> Option<&str> {
        self.keys.first().map(|s| s.as_str())
    }

    /// 返回有效 Key 数量
    pub fn len(&self) -> usize {
        self.keys.len()
    }
}

impl From<Vec<String>> for ApiKeyPool {
    fn from(keys: Vec<String>) -> Self {
        Self::new(keys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filters_empty_keys() {
        let pool = ApiKeyPool::new(vec!["".to_string(), "key1".to_string(), "".to_string()]);
        assert_eq!(pool.keys(), &["key1"]);
        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn test_deduplication() {
        let pool = ApiKeyPool::new(vec!["a".to_string(), "b".to_string(), "a".to_string()]);
        assert_eq!(pool.keys(), &["a", "b"]);
    }

    #[test]
    fn test_merged_compat() {
        let pool = ApiKeyPool::merged("primary", &["extra1".to_string(), "extra2".to_string()]);
        assert_eq!(pool.keys(), &["primary", "extra1", "extra2"]);
    }

    #[test]
    fn test_merged_dedup() {
        let pool = ApiKeyPool::merged("key1", &["key1".to_string(), "key2".to_string()]);
        assert_eq!(pool.keys(), &["key1", "key2"]);
    }

    #[test]
    fn test_empty_primary_with_extras() {
        let pool = ApiKeyPool::merged("", &["key1".to_string(), "key2".to_string()]);
        assert_eq!(pool.keys(), &["key1", "key2"]);
    }

    #[test]
    fn test_is_empty() {
        let pool = ApiKeyPool::new(vec![]);
        assert!(pool.is_empty());

        let pool = ApiKeyPool::new(vec!["k".to_string()]);
        assert!(!pool.is_empty());
    }
}
