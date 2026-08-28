# SNDK 深度投研回答重构交接

## 状态

- 工作树：`/Users/wangxx/Documents/Playground/honeclaw-sndk-answer-rebuild`
- 分支：`codex/sndk-answer-rebuild-20260829`
- 基线：`832e6b9e`
- 生产状态：未部署、未推送、未修改生产配置。
- 原 `oldwang` 脏工作树保持原状；本任务只把相关蒸馏成果按文件移入独立工作树。

代码、491 轮可追踪验证、本地数据预取和 CI-safe 门禁已完成。唯一未闭环项是 Gemini 3.7 Flash 的真实模型调用与 131 条问题 live 重答：当前代理没有 Gemini 通道。

## 已落地

1. SNDK 深度回答契约
   - 基本面必须连接需求、客户用途、收入、成本/资本开支和价值捕获。
   - 护城河与稀缺性必须讨论良率、主控/固件、客户认证、切换成本、供给约束和证据状态，不能用份额替代护城河。
   - 行业竞争至少覆盖两个具名对手，并回答替代性、供给策略和相对强弱。
   - 财务拆分量、价、毛利率、OCF/FCF、资本开支、现金/债务、库存/应收，并区分周期与结构。
   - 估值披露中周期假设、至少两种适用方法、熊/基准/牛情景、当前位置和反向估值；把需求传导到利润/FCF、持续期和倍数。
   - NBM/RPO 不等于现金或利润；HBF 在认证和规模交付前不进入基准情景。
2. 当前证据链
   - DataFetch 增加受控的 Nasdaq/SEC 官方降级路径，补齐身份、行情、财务和估值证据；`fmp.official_fallback_enabled` 可在封闭测试中关闭。
   - 复用当前 Hari Invest 蒸馏的判断框架与公司决策上下文，但不把历史材料当作当前事实。
3. 图像识别
   - LLM provider 新增真实图片字节输入；OpenAI-compatible 请求发送真实 base64 data URL 和高细节图片内容块。
   - 附件只有在下载、类型和尺寸校验后才交给视觉模型；不会读取用户提示中的任意本地路径。
   - 提示内容顺序为 `【视觉模型描述】` 后接 `【图片文字提取】`；视觉失败时保留 OCR 降级。
4. Gemini Flash 反代
   - `config.example.yaml` 提供 `gemini_flash_proxy` provider、`gemini_flash` profile 和 `agent.image_understanding_profile` 示例。
   - 手工探针 `tests/regression/manual/test_gemini_flash_proxy.sh` 同时验证文本和真实图片，且不会打印密钥。
   - 操作说明见 `docs/runbooks/gemini-flash-proxy-and-image-understanding.md`。

## 491 轮台账

测试：

```text
cargo test -p hone-channels --lib sndk_360_round_validation_ledger -- --nocapture
HONE_RESCORED_ROWS_JSON=/absolute/path/rescored_rows.json \
  cargo test -p hone-channels --lib hone_131_target_samples_extend_ledger_to_491_rounds -- --ignored --nocapture
```

输出：

```text
target/sndk-validation/sndk-360-round-ledger.ndjson
target/sndk-validation/hone-491-round-validation-ledger.ndjson
```

- 1–240：八类评分失败家族，每类 30 个输入变形。
- 241–320：视觉+OCR、仅视觉、仅 OCR、无提取四种状态，每类 20 轮。
- 321–360：完整 SNDK 深度回答通过本地确定性契约管线。
- 361–491：评分表全部 131 条真实问题；只读取问题、旧回答、问题说明和新评分，不读取或写入用户身份字段。
- 真实样本每行分别记录 `contract_coverage` 和 `live_model_validation`。当前前者全部通过，后者全部为 `pending_gemini_channel`；不得把断言数量称为模型对话次数。

## 验证结果

- `hone-channels`：808 通过、2 忽略；其中评分表 131 样本测试需要显式路径，已单独执行通过。
- `hone-core`：155 通过。
- `hone-llm`：36 通过。
- `hone-tools`：192 通过、1 忽略。
- 默认相关库合计：1191 通过、3 忽略、0 失败；额外 131 样本手工测试 1 通过。
- 财经自动化契约：49/49 通过。
- 全部 CI-safe 脚本通过。全量串行运行在后段因磁盘耗尽中断；删除可再生 `target` 后，剩余脚本分别继续执行并全部通过。这不是测试断言失败。
- 本地 HONE 真实探针成功预取 SNDK 身份、行情、财务和 SEC 证据；模型流随后被代理以 HTTP 403 拒绝，因此没有最终自然语言回答。测试后已删除临时凭据配置。

## 外部阻塞与继续步骤

Google 官方已经确认精确稳定模型 ID 为 `gemini-3.7-flash`，官方 OpenAI-compatible 地址为 `https://generativelanguage.googleapis.com/v1beta/openai`。现在只需要二选一：有该模型权限的反代 endpoint/API key，或 Google Gemini API key。拿到后：

1. 只在本机私有 `config.yaml` 中填写 provider/profile，不提交密钥。
2. 设置 `HONE_GEMINI_FLASH_BASE_URL`、`HONE_GEMINI_FLASH_API_KEY`，必要时设置 `HONE_GEMINI_FLASH_MODEL`，运行手工探针。
3. 探针文本和图片都通过后，设置 `agent.image_understanding_profile: gemini_flash`。
4. 用真实图片验证视觉描述与 OCR 顺序，再用本地 HONE 重跑 SNDK 深度问题。
5. 检查最终回答满足七段因果链、当前来源、情景估值与反向估值，然后才可将计划标记完成并归档。

本机进一步核验了全部可复用入口：Luna 令牌对目录和目标模型返回 403；Claude 令牌目录只有八个 Claude 模型，对目标模型返回 503 无可用通道；Bob 登录控制台当前 31 个模型中没有 Gemini，本机也没有 Gemini CLI、Gemini API key 或监听中的 Gemini 代理。不要继续猜测模型别名，也不要通过吞掉错误把它记录为成功；密钥不得写进仓库、日志或交接文件。
