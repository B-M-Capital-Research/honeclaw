# Gemini Flash 反代与图片理解

## 目的

HONE 可把任意 OpenAI-compatible 端点配置成独立的 Gemini Flash profile，并把它用于默认文本回答或图片附件理解。图片链路发送服务端已经准入、落盘并读入内存的真实字节；不会根据用户提示词中的“本地路径”读取文件。视觉调用失败时保留 OCR 降级，不让附件整轮失败。

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

现有本地反代凭据对测试过的 `gemini-3.7-flash`、`google/gemini-3.7-flash` 和 `gemini-3.7-flash-preview` 均返回 HTTP 403。代码和配置能力已就绪，但在新的可用端点、账号权限或精确模型 ID 通过上述双探针前，不得把本机 Gemini 3.7 Flash 标记为已接通。
