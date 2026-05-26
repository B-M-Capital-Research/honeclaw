# Regression Tests

这个目录分成两类脚本：

- `ci/`：CI-safe 回归脚本
- `manual/`：依赖外部 CLI、账号或本机状态的手工回归脚本

## 运行方式

- `bash tests/regression/run_ci.sh`
- `bash tests/regression/run_manual.sh`
- `bash tests/regression/manual/test_<topic>.sh`
- `RUN_EVENT_ENGINE_LLM_BASELINE=1 bash tests/regression/manual/test_event_engine_news_classifier_baseline.sh`
  - 默认使用 fixture 中的推荐模型；可选: `EVENT_ENGINE_NEWS_CLASSIFIER_MODEL=x-ai/grok-4.3`
  - 默认不调用真实 LLM API；设置环境变量后才会对保存的历史新闻样本做 live drift 对比
- `RUN_EVENT_ENGINE_FMP_LIVE_SMOKES=1 HONE_FMP_API_KEY=... bash tests/regression/manual/test_event_engine_fmp_poller_live_smokes.sh`
- `RUN_ALIYUN_LIVE_SMOKES=1 ALIYUN_LIVE_SCOPE=captcha|sms|all bash tests/regression/manual/test_aliyun_live_smokes.sh`
- `RUN_EVENT_ENGINE_LIVE_SMOKES=1 EVENT_ENGINE_LIVE_SCOPE=fmp|telegram|telegram-llm|portfolio|social|all bash tests/regression/manual/test_event_engine_live_integration_smokes.sh`

## 约定

- CI-safe 脚本必须无交互、可重复、可判定，失败时返回非 0
- 手工脚本可以依赖外部账号或本机环境，但不要把它们当成默认 CI 门禁
- 会调用真实外部服务、发送消息或消耗配额的 live smoke wrapper 默认应跳过；必须设置脚本声明的 `RUN_*_LIVE_SMOKES=1` gate 后才实际执行
- 新增脚本时优先选择明确的主题命名，例如 `test_secret_scan.sh`
