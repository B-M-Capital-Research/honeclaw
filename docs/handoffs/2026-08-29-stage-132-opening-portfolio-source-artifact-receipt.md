# Stage 132：期初组合来源工件单次加密接收

日期：2026-08-29

## 结果

Stage 132 已实现，但没有执行任何真实接收。管理员现在可以在存在有效 Stage 131 claim 且配置稳定加密密钥时，提交一次 request-first multipart 流。服务端会在读取首个来源字节前持久化 start marker，永久消费 claim；成功生成未受信 receipt，失败或中断生成不可重试终态。

## 接收与托管合同

- 仅原始 provider PDF、CSV、JSON；最多 64 件、64 MiB/件、256 MiB/receipt。
- PDF 必须结构可解析并拒绝 JavaScript、Launch、EmbeddedFile、OpenAction 和加密；CSV 拒绝公式主动内容和账户/凭据表头；JSON 递归拒绝账户号、密码、token、API key 等敏感键。
- 不支持 URL 抓取、客户端路径、归档或符号链接。原始文件名、原始账户号和凭据不进入日志、路径或 manifest。
- 原始字节以 AES-256-GCM 加密后按明文 SHA-256 内容寻址并 create-new 提交；同内容幂等，不同内容不可覆盖。`HONE_OPENING_PORTFOLIO_RECEIPT_ENCRYPTION_KEY` 必须是稳定的 64 位 hex 值。
- receipt 只证明接收、筛查、加密托管与哈希事实，始终 `untrusted=true`。失败记录保守标记“可能已读取部分来源流”。

## 权限边界

本阶段不解析财务行，不物化或准入 opening portfolio snapshot，不开放 financial-event allowlist，不写账本、持仓、现金、NAV/绩效、模型指标、训练/RL/reward、订单、券商或交易状态。下一门只能是 Stage 133 责任链外独立 receipt 验证。

## 验证

- Stage 132 Rust：5/5。
- HONE Web API：1295 passed、2 ignored、0 failed。
- 前端：705/705，3508 assertions。
- 金融自动化契约：49/49。
- TypeScript、标准与 public 生产构建、Rust fmt、workspace all-target check 通过；workspace 检查仅使用仓库提供的本地开发开关跳过缺失 iMessage 打包 sidecar 的资源存在性检查。
- 零状态审计：start、result、quarantine、加密 content object、receipt 目录均不存在；未创建真实 Stage 131 claim 或任何财务状态。

## 下一步

Stage 133 必须由与 Stage 132 executor 分离的复核者/实现重新验证精确责任链、密文完整性、内容寻址、manifest 自哈希、脱敏与资源上限，并只输出 append-only terminal validation。不得在 Stage 133 同时解读持仓、生成快照或准入财务事实。

---

## Stage 133 后续：加密 receipt 责任链外独立验证

状态：已实现、未创建真实验证记录。

### 结果

- 新增 GET registry 与 `/{attempt_id}/validate-once`；只接收预期摘要、理由和 13 项确认，不接收文件、路径或来源字节。
- validator 排除 Stage 132 executor、Stage 131 claimant 与完整 Stage 51–132 责任链。第二实现重算 result/receipt/ciphertext，独立派生 nonce/AAD 并执行 AES-256-GCM 认证解密，再重算明文内容地址、格式安全结构和脱敏证据。
- 每个 receipt 只形成一个 create-once、自哈希终态。密钥缺失或不匹配会在终态前停止，工件/凭证漂移会形成失败终态。
- readiness 升级为 v130，管理端与统一决策大脑明确显示：验证通过只证明 receipt 完整，不证明持仓数据真实，只开放 Stage 134 零能力实现登记。

### 验证

- Stage 133 Rust：5/5。
- HONE Web API：1300 passed、2 ignored、0 failed。
- 前端：708/708，3522 assertions。
- 金融自动化契约：49/49。
- TypeScript、标准与 public 生产构建通过。
- 零状态审计：没有 Stage 133 validation 目录/记录，也没有 receipt、解密明文、opening snapshot 或财务状态。

### 风险与回滚

- 运行真实验证必须由同一稳定的 `HONE_OPENING_PORTFOLIO_RECEIPT_ENCRYPTION_KEY` 提供认证解密能力；丢失该密钥无法恢复原始字节。
- terminal failure 是不可覆盖的审计事实；若原因是工件损坏或上游错误，应从新的上游授权/接收链重新开始，不能修改旧记录。
- 代码回滚可移除 Stage 133 路由、readiness 和管理端面板；不得删除已经存在的终态文件。当前零真实记录，因此没有数据迁移或清理动作。

### 下一步

Stage 134 最多只登记期初快照物化的零能力实现合同，冻结 parser/materializer 的输入输出、精确十进制、完整账户与失败关闭边界；登记阶段不得解密来源、解析金融行或创建真实持仓。

---

## Stage 134 后续：期初快照物化零能力实现登记

状态：已实现、未创建真实登记。

### 结果

- 新增 GET registry 与 `/{validation_id}/register-once`；只接受不可变代码版本、理由和 18 项合同确认，不接收 key、文件、路径、明文或解析结果。
- registrar 排除 Stage 133 validator、Stage 132 executor、Stage 131 claimant 与完整 Stage 51–133 责任链，并精确绑定 validation、result、claim、receipt 和 Stage 125 specification。
- 未来合同固定确定性 PDF/CSV/JSON adapter、完整账户/现金/持仓/上市期权/负债/未结算活动、精确十进制、有符号数量、证券身份与公司行动对账，以及每行工件 SHA-256/来源位置。
- 缺失、歧义、不支持资产、部分账户、手填、默认或推断将使整份快照失败；对账单市场价值只作信息字段。任何未来输出仍是 untrusted candidate，须独立验证和准入。
- readiness 升级为 v131；登记通过只开放 Stage 135 责任链外实现复核。

### 验证

- Stage 134 Rust：5/5。
- HONE Web API：1305 passed、2 ignored、0 failed。
- 前端：712/712，3541 assertions。
- 金融自动化契约：49/49。
- TypeScript、标准/public 生产构建、workspace all-target check、Rust fmt 与 diff hygiene 通过。
- 零状态审计：Stage 134 registration、来源工件/receipt、opening snapshot/holdings 均为 0。

### 风险与回滚

- 当前只有合同，没有可执行物化器；任何展示真实持仓的功能仍应明确阻断，不能用研究观点、默认本金或手工补值代替来源。
- 代码回滚可移除 Stage 134 路由、readiness 和管理端面板；未来若已有 append-only 登记，不得删除历史记录。当前零真实记录，无数据迁移或清理动作。

### 下一步

Stage 135 最多实现责任链外独立实现复核：第二实现重新构建并逐字段验证 Stage 134 合同，仍不得读取/解密 receipt、运行 parser、生成候选快照或准入财务事实。

---

## Stage 135 后续：期初快照物化实现责任链外独立审查

状态：已实现、未创建真实审查记录。

### 结果

- 新增 GET registry 与 `/{implementation_id}/review`；只接受不可变摘要、审查说明、批准/修改/拒绝结论和 19 项确认，不接收 key、receipt 字节、路径、明文或解析结果。
- reviewer 排除 Stage 134 registrar、Stage 133 validator、Stage 132 executor、Stage 131 claimant 与 Stage 51–134 完整责任链。第二实现不调用 Stage 134 builder，自行重建 10 个固定函数及完整物化合同，并重算 Stage 125/131/132/133/134 哈希和绑定。
- 审查再次验证完整账户、现金、持仓、上市期权、负债、未结算活动、精确十进制、有符号数量、证券身份、公司行动、逐行来源和整份快照失败语义；对账单市值仍只作信息字段。
- readiness 升级为 v132；管理端、历史治理与统一决策大脑已接通。只有明确独立批准才开放 Stage 136 隔离物化器规格登记。

### 验证

- Stage 135 Rust：5/5。
- HONE Web API：1310 passed、2 ignored、0 failed。
- 前端：717/717，3564 assertions。
- 金融自动化契约：49/49。
- TypeScript、标准/public 生产构建、workspace all-target check、Rust fmt 与 diff hygiene 通过。
- 零状态审计：Stage 134 registration、Stage 135 review、来源工件/receipt、opening snapshot/holdings 目录均不存在。

### 风险与回滚

- 本阶段只证明完整合同可被第二实现重建，不证明未来 parser 工件正确，更不证明任何来源持仓真实。
- changes-required 或 rejected 是当前实现的终态，必须创建新的 Stage 134 登记；不得覆盖旧审查或把拒绝改成批准。
- 代码回滚可移除 Stage 135 路由、readiness 与面板；未来若已有 append-only 审查记录，不得删除。当前零真实记录，无数据迁移或清理动作。

### 下一步

Stage 136 最多登记隔离物化器规格，冻结服务端保管工件、sandbox、资源上限和确定性复现步骤；仍不得读取/解密 receipt、运行 parser、创建候选快照或开放任何财务、训练和交易权限。
