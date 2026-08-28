# Gemini Flash 反代与图片理解

## 目的

HONE 可把任意 OpenAI-compatible 端点配置成独立的 Gemini Flash profile，并把它用于默认文本回答或图片附件理解。该端点可以是本地/第三方反代，也可以是 Google 官方兼容地址 `https://generativelanguage.googleapis.com/v1beta/openai`。图片链路发送服务端已经准入、落盘并读入内存的真实字节；不会根据用户提示词中的“本地路径”读取文件。视觉调用失败时保留 OCR 降级，不让附件整轮失败。

Google 官方模型页确认稳定模型 ID 为 `gemini-3.7-flash`，输入支持文本、图片、视频、音频和 PDF；官方 OpenAI compatibility 文档同时确认 `chat/completions`、streaming、function calling 与 image understanding。模型页与兼容文档分别为：

- <https://ai.google.dev/gemini-api/docs/models/gemini-3.7-flash>
- <https://ai.google.dev/gemini-api/docs/openai>

## 启用前探针

不要仅凭模型名或 `/models` 列表判定可用。使用临时环境变量运行：

```bash
HONE_GEMINI_FLASH_BASE_URL='https://proxy.example/v1' \
HONE_GEMINI_FLASH_API_KEY='replace-me' \
HONE_GEMINI_FLASH_MODEL='gemini-3.7-flash' \
bash tests/regression/manual/test_gemini_flash_proxy.sh
```

探针必须同时通过普通文本和 `image_url` data URL 两条请求。HTTP 403、模型不存在、返回空内容或不接受图片都视为不可启用。脚本不会打印 API key。

## 配置

在 `config.yaml` 的 `llm.providers` 增加 OpenAI-compatible provider，并在 `llm.profiles` 增加 profile。字段形状见 `config.example.yaml` 的 `gemini_flash_proxy` 与 `gemini_flash`。

- 仅把图片交给该模型：设置 `agent.image_understanding_profile: gemini_flash`。
- 让 function-calling 默认 LLM 也走该模型：设置 `llm.default_profile: gemini_flash`。
- ACP runner 是否使用这个 profile 由 runner 自己的模型配置决定；不要误以为修改 `llm.default_profile` 会改写 Codex ACP 的模型。

配置后重启本地 HONE，再上传一张同时包含图表与文字的图片。回答上下文应出现“图片证据提取”，视觉描述先于 OCR；两者冲突时回答必须披露不确定性。

## 当前本机结论（2026-08-29）

模型 ID 已由 Google 官方文档确认，不再是待猜测项。本机现有 Bob API 状态如下：

- Luna 令牌访问 `/models` 和 `gemini-3.7-flash` 均返回 HTTP 403，属于令牌分组权限不足。
- Claude 令牌可读取 `/models`，但目录只有八个 Claude 模型；请求 `gemini-3.7-flash` 返回 HTTP 503 “无可用通道”。
- 登录控制台的模型广场当前列出 31 个模型和 Anthropic、Moonshot、OpenAI 等供应商，没有 Gemini 供应商或 Gemini 模型；控制台公告也说明其 Gemini 上游不稳定。
- 本机没有 `gemini` CLI、`GEMINI_API_KEY` 或正在监听的 Gemini 代理。

因此当前阻塞是“没有可用 Gemini 通道/凭据”，不是模型 ID、HONE 请求格式或图片协议问题。在新的反代分组或 Google Gemini API key 通过上述文本+图片双探针前，不得把本机 Gemini 3.7 Flash 标记为已接通。
