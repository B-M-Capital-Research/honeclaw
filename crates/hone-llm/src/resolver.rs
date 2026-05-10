use std::sync::Arc;

use hone_core::api_key_pool::ApiKeyPool;
use hone_core::config::{
    HoneConfig, LlmProfileEntryConfig, LlmProfileParamsConfig, LlmProviderEntryConfig,
};
use serde_json::{Map, Value};

use crate::{LlmProvider, LlmRequestOptions, OpenAiCompatibleProvider, OpenRouterProvider};

const OPENROUTER_PROVIDER: &str = "openrouter";
const DEFAULT_OPENROUTER_BASE_URL: &str = "https://openrouter.ai/api/v1";

#[derive(Clone)]
pub struct CreatedLlmProvider {
    pub provider: Arc<dyn LlmProvider>,
    pub model: String,
    pub provider_name: String,
    pub profile_name: Option<String>,
}

pub struct LlmResolver<'a> {
    config: &'a HoneConfig,
}

impl<'a> LlmResolver<'a> {
    pub fn new(config: &'a HoneConfig) -> Self {
        Self { config }
    }

    pub fn provider_for_profile_or_openrouter_model(
        &self,
        profile_ref: Option<&str>,
        legacy_model: &str,
        fallback_model: &str,
        max_tokens_override: Option<u16>,
    ) -> hone_core::HoneResult<CreatedLlmProvider> {
        if let Some(profile_name) = normalized_ref(profile_ref) {
            return self.provider_for_profile(profile_name, max_tokens_override);
        }

        let model = normalize_model(legacy_model, fallback_model);
        let max_tokens =
            max_tokens_override.unwrap_or(self.config.llm.openrouter.max_tokens as u16);
        let provider = OpenRouterProvider::from_config_with_model_and_max_tokens(
            self.config,
            &model,
            max_tokens,
        )?;
        Ok(CreatedLlmProvider {
            provider: Arc::new(provider),
            model,
            provider_name: OPENROUTER_PROVIDER.to_string(),
            profile_name: None,
        })
    }

    pub fn auxiliary_provider(
        &self,
        profile_ref: Option<&str>,
        max_tokens_override: Option<u16>,
    ) -> hone_core::HoneResult<CreatedLlmProvider> {
        if let Some(profile_name) = normalized_ref(profile_ref) {
            return self.provider_for_profile(profile_name, max_tokens_override);
        }

        if self.config.llm.auxiliary.is_configured() {
            let api_key = self.config.llm.auxiliary.resolved_api_key();
            if api_key.trim().is_empty() {
                return Err(hone_core::HoneError::Config(
                    "auxiliary API key is empty".to_string(),
                ));
            }
            let max_tokens =
                max_tokens_override.unwrap_or(self.config.llm.auxiliary.max_tokens as u16);
            let provider = OpenAiCompatibleProvider::new(
                &api_key,
                self.config.llm.auxiliary.base_url.trim(),
                self.config.llm.auxiliary.model.trim(),
                self.config.llm.auxiliary.timeout,
                max_tokens,
            )?;
            return Ok(CreatedLlmProvider {
                provider: Arc::new(provider),
                model: self.config.llm.auxiliary.model.trim().to_string(),
                provider_name: "auxiliary".to_string(),
                profile_name: None,
            });
        }

        self.provider_for_profile_or_openrouter_model(
            None,
            self.config.llm.openrouter.auxiliary_model(),
            self.config.llm.openrouter.auxiliary_model(),
            max_tokens_override,
        )
    }

    pub fn provider_for_profile(
        &self,
        profile_name: &str,
        max_tokens_override: Option<u16>,
    ) -> hone_core::HoneResult<CreatedLlmProvider> {
        let profile = self.config.llm.profiles.get(profile_name).ok_or_else(|| {
            hone_core::HoneError::Config(format!("llm.profiles.{profile_name} 不存在"))
        })?;
        let provider_name = profile.provider.trim();
        if provider_name.is_empty() {
            return Err(hone_core::HoneError::Config(format!(
                "llm.profiles.{profile_name}.provider 不能为空"
            )));
        }
        let provider_cfg = self
            .config
            .llm
            .providers
            .get(provider_name)
            .ok_or_else(|| {
                hone_core::HoneError::Config(format!("llm.providers.{provider_name} 不存在"))
            })?;
        let model = profile.model.trim();
        if model.is_empty() {
            return Err(hone_core::HoneError::Config(format!(
                "llm.profiles.{profile_name}.model 不能为空"
            )));
        }

        let mut options = request_options_from_profile(profile, provider_name);
        if let Some(max_tokens) = max_tokens_override {
            options.max_tokens = Some(max_tokens as u32);
        }
        let max_tokens = options
            .max_tokens
            .unwrap_or(self.config.llm.openrouter.max_tokens)
            .clamp(1, u16::MAX as u32) as u16;

        let provider_kind = provider_cfg.kind.trim().to_ascii_lowercase();
        let provider: Arc<dyn LlmProvider> =
            if provider_name == OPENROUTER_PROVIDER || provider_kind == OPENROUTER_PROVIDER {
                let pool = self.provider_key_pool(provider_name, provider_cfg);
                let base_url = if provider_cfg.base_url.trim().is_empty() {
                    DEFAULT_OPENROUTER_BASE_URL
                } else {
                    provider_cfg.base_url.trim()
                };
                Arc::new(OpenRouterProvider::from_key_pool(
                    pool.keys(),
                    base_url,
                    model,
                    provider_cfg
                        .timeout
                        .unwrap_or(self.config.llm.openrouter.timeout),
                    max_tokens,
                    options,
                )?)
            } else {
                let key = self
                    .provider_key_pool(provider_name, provider_cfg)
                    .first()
                    .map(str::to_string)
                    .ok_or_else(|| {
                        hone_core::HoneError::Config(format!(
                            "llm.providers.{provider_name} API key 未配置"
                        ))
                    })?;
                if provider_cfg.base_url.trim().is_empty() {
                    return Err(hone_core::HoneError::Config(format!(
                        "llm.providers.{provider_name}.base_url 不能为空"
                    )));
                }
                Arc::new(
                    OpenAiCompatibleProvider::new(
                        &key,
                        provider_cfg.base_url.trim(),
                        model,
                        provider_cfg.timeout.unwrap_or(120),
                        max_tokens,
                    )?
                    .with_request_options(options),
                )
            };

        Ok(CreatedLlmProvider {
            provider,
            model: model.to_string(),
            provider_name: provider_name.to_string(),
            profile_name: Some(profile_name.to_string()),
        })
    }

    fn provider_key_pool(
        &self,
        provider_name: &str,
        provider: &LlmProviderEntryConfig,
    ) -> ApiKeyPool {
        let mut keys = Vec::new();
        if !provider.api_key.trim().is_empty() {
            keys.push(provider.api_key.trim().to_string());
        }
        keys.extend(
            provider
                .api_keys
                .iter()
                .map(|key| key.trim())
                .filter(|key| !key.is_empty())
                .map(str::to_string),
        );
        let env_name = provider.api_key_env.trim();
        if !env_name.is_empty() {
            if let Ok(value) = std::env::var(env_name) {
                let value = value.trim().to_string();
                if !value.is_empty() {
                    keys.push(value);
                }
            }
        }
        if keys.is_empty() && provider_name == OPENROUTER_PROVIDER {
            keys.extend(
                self.config
                    .llm
                    .openrouter
                    .effective_key_pool()
                    .keys()
                    .iter()
                    .cloned(),
            );
            let env_name = self.config.llm.openrouter.api_key_env.trim();
            if !env_name.is_empty() {
                if let Ok(value) = std::env::var(env_name) {
                    let value = value.trim().to_string();
                    if !value.is_empty() {
                        keys.push(value);
                    }
                }
            }
        }
        ApiKeyPool::new(keys)
    }
}

fn normalized_ref(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

fn normalize_model(value: &str, fallback: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        fallback.trim().to_string()
    } else {
        value.to_string()
    }
}

fn request_options_from_profile(
    profile: &LlmProfileEntryConfig,
    provider_name: &str,
) -> LlmRequestOptions {
    let mut options = request_options_from_params(&profile.params);
    if let Some(provider_options) = profile.provider_options.get(provider_name) {
        for (key, value) in &provider_options.extra_body {
            options.extra_body.insert(key.clone(), value.clone());
        }
    }
    options
}

fn request_options_from_params(params: &LlmProfileParamsConfig) -> LlmRequestOptions {
    let reasoning = params
        .reasoning
        .as_ref()
        .and_then(|value| serde_json::to_value(value).ok())
        .filter(|value| !is_empty_json_value(value));
    LlmRequestOptions {
        max_tokens: params.max_tokens,
        temperature: params.temperature,
        top_p: params.top_p,
        stop: params.stop.clone(),
        seed: params.seed,
        reasoning,
        response_format: params.response_format.clone(),
        tool_choice: params.tool_choice.clone(),
        parallel_tool_calls: params.parallel_tool_calls,
        extra_body: params
            .extra_body
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect::<Map<String, Value>>(),
    }
}

fn is_empty_json_value(value: &Value) -> bool {
    value.is_null()
        || matches!(value, Value::Array(items) if items.is_empty())
        || matches!(value, Value::Object(map) if map.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hone_core::config::HoneConfig;

    #[test]
    fn resolves_profile_params_into_request_options() {
        let yaml = r#"
llm:
  providers:
    openrouter:
      kind: openrouter
      api_key: test
  profiles:
    digest:
      provider: openrouter
      model: x-ai/grok-4.1-fast
      params:
        max_tokens: 123
        temperature: 0.2
        reasoning:
          effort: low
        response_format:
          type: json_object
"#;
        let cfg: HoneConfig = serde_yaml::from_str(yaml).unwrap();
        let created = LlmResolver::new(&cfg)
            .provider_for_profile("digest", None)
            .unwrap();
        assert_eq!(created.model, "x-ai/grok-4.1-fast");
        assert_eq!(created.profile_name.as_deref(), Some("digest"));
    }
}
