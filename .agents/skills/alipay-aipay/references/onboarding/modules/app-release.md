# 应用发布模块

> 本文档定义应用发布的完整流程、MCP调用规范和关键处理逻辑。
> 被引用文档：`onboarding/flow.md` → Step 3.1 应用查询 + Step 5.3 应用发布
> 本文档位于 `references/onboarding/modules/`，文中的 `scripts/...` 路径相对本目录；从技能包根目录执行时对应 `references/onboarding/modules/scripts/...`。

---

## 功能概述

负责应用发布全流程，包括查询已有应用、复用/创建应用、公钥设置、审核提交。应用类型支持 `WEBAPP` 和 `MOBILEAPP`。

**触发条件：**
- 用户说"发布应用"、"创建应用"
- onboarding 登录授权完成后触发应用列表查询；需要签约时，应用复用/创建等后续操作仍在签约提交成功后执行

> APP支付对应移动应用发布：应用列表查询和应用创建使用 `MOBILEAPP`；网站支付和按量付费继续使用 `WEBAPP`。

---

## 流程骨架

```
前置阶段 0: 初始化 .gitignore（防止密钥文件泄露）
    ↓
前置阶段 1: 检查 CLI 登录态
    ↓
阶段 2: 登录后查询已有应用 (queryApplicationList)
    ↓
阶段 3: 与签约状态/其他资源合并展示，用户一次选择并提供条件资料
    ↓
onboarding 且当前未签约: 先返回主流程完成 Step 5.1 签约提交
    ↓
签约提交成功，或已签约/已提交签约: 执行应用分支
    ├─ 【复用】选择已有上线应用 → 查询安全密钥
    │      ├─ RSA2 应用公钥已生效 → 导出 alipayPublicKey → 完成
    │      └─ RSA2 应用公钥未生效 → 停止复用成功结论，由用户选择设钥或返回应用决策
    └─ 【新建】创建新应用 → 进入阶段 3.2
    ↓
阶段 3.2: 创建应用 (createApplication) — 仅新建路径
    ↓
阶段 4: 设置应用公钥 (createKeyConfirmPage) — 新建路径或复用缺公钥恢复路径
    ↓
阶段 4.1: 确认应用公钥已生效 (queryApplicationSecurityKey) — 新建路径或复用缺公钥恢复路径
    ├─ 复用缺公钥恢复路径 → 重新执行复用分支，重新校验应用状态并导出支付宝公钥
    └─ 新建路径 → 阶段 5: 提交应用审核 (submitApplicationAudit)
```

---

## MCP调用

| 方法 | 用途 |
|------|------|
| `queryApplicationList` | 查询同类型已有应用 |
| `createApplication` | 创建应用 |
| `queryApplicationInfo` | 查询应用信息 |
| `queryApplicationDetail` | 查询应用详情 |
| `createKeyConfirmPage` | 创建密钥确认页 |
| `queryApplicationSecurityKey` | 查询应用安全密钥 |
| `submitApplicationAudit` | 提交应用审核 |

## 应用类型规则

应用类型必须在查询和创建阶段保持一致：

| 产品 | productType | salesCode | 应用类型 | 查询应用入参 | 创建应用入参 |
|------|-------------|-----------|----------|--------------|--------------|
| 按量付费 | `aipay` | `I1080300001000160457` | `WEBAPP` | `{"request":{"appTypes":["WEBAPP"]}}` | `{"request":{"applicationType":"WEBAPP","createScene":"cli"}}` |
| 网站支付 | `webpay` | `I1080300001000041203` | `WEBAPP` | `{"request":{"appTypes":["WEBAPP"]}}` | `{"request":{"applicationType":"WEBAPP","createScene":"cli"}}` |
| APP支付 | `apppay` | `I1080300001000041313` | `MOBILEAPP` | `{"request":{"appTypes":["MOBILEAPP"]}}` | `IOS` 需补充 `mobilePlatform`+`bundleId`；`ANDROID` 需补充 `mobilePlatform`+`appPackage`+`appSign`；`ALL` 需三项都传 |

`scripts/app.sh` 的应用类型解析规则：

1. 如果显式传入 `--application-type WEBAPP|MOBILEAPP`，必须与产品上下文匹配。
2. 否则如果传入 `--product-type apppay` 或 `--sales-code I1080300001000041313`，使用 `MOBILEAPP`。
3. 其他情况默认使用 `WEBAPP`。

`MOBILEAPP` 平台字段规则：

- `alipay-aipay` 当前要求 APP支付创建 `MOBILEAPP` 前必须明确 `mobilePlatform`，禁止用最小入参创建 `MOBILEAPP`。
- `mobilePlatform` 允许 `IOS`、`ANDROID` 或 `ALL`；`ALL` 表示同时支持 iOS 和 Android。
- `mobilePlatform=IOS`：必须同时传 `bundleId`。
- `mobilePlatform=ANDROID`：必须同时传 `appPackage` 和 `appSign`。
- `mobilePlatform=ALL`：必须同时传 `bundleId`、`appPackage` 和 `appSign`。
- 用户说“bundle_id”时，映射到脚本参数 `--bundle-id`，MCP 请求字段为 `bundleId`。
- `appSign` 是 Android 应用签名摘要字段，只在创建 `MOBILEAPP` 时作为 `createApplication` 入参采集；它不是应用公钥、支付签名串或私钥。
- 采集 `appSign` 时只要求用户提供其 Android 应用对应的签名摘要值；不得引导用户通过支付宝开放平台官方密钥生成工具、签名工具或格式转换工具获取，也不得把阶段 4 的应用公钥生成引导套用到 `appSign`。
- 不确定字段值时不要编造，也不要用包名、签名摘要或 bundleId 的占位符调用 `createApplication`。

---

## ⛔ 应用相关 MCP 调用铁律（最高优先级）

**应用相关 MCP 调用具有独特的参数结构，与签约模块完全不同，禁止混用！**

### 参数结构对照表

| 方法 | 必需参数 | 参数结构 | 是否需要 ctx |
|------|----------|----------|--------------|
| `queryApplicationList` | `appTypes` | `{"request":{"appTypes":["WEBAPP"]}}` 或 `{"request":{"appTypes":["MOBILEAPP"]}}` | ❌ 不需要 |
| `createApplication` | `applicationType`, `createScene`；APP支付创建 `MOBILEAPP` 时还必须按平台传字段 | `WEBAPP`: `{"request":{"applicationType":"WEBAPP","createScene":"cli"}}`；`MOBILEAPP IOS`: `{"request":{"applicationType":"MOBILEAPP","createScene":"cli","mobilePlatform":"IOS","bundleId":"..."}}`；`MOBILEAPP ANDROID`: `{"request":{"applicationType":"MOBILEAPP","createScene":"cli","mobilePlatform":"ANDROID","appPackage":"...","appSign":"..."}}`；`MOBILEAPP ALL`: `{"request":{"applicationType":"MOBILEAPP","createScene":"cli","mobilePlatform":"ALL","bundleId":"...","appPackage":"...","appSign":"..."}}` | ❌ 不需要 |
| `queryApplicationInfo` | `appId` | `{"request":{"appId":"..."}}` | ❌ 不需要 |
| `queryApplicationDetail` | `appId` | `{"request":{"appId":"..."}}` | ❌ 不需要 |
| `createKeyConfirmPage` | `appId`, `signType`, `publicKey` | `{"request":{"appId":"...","signType":"RSA2","publicKey":"..."}}` | ❌ 不需要 |
| `queryApplicationSecurityKey` | `appId` | `{"request":{"appId":"..."}}` | ❌ 不需要 |
| `submitApplicationAudit` | `appId` | `{"request":{"appId":"..."}}` | ❌ 不需要 |

### 正确调用示例

> 📎 所有 MCP 调用均有对应脚本，直接执行脚本即可。以下列出各命令与脚本的对应关系：

```bash
# ✅ 查询应用列表 → scripts/app.sh list --product-type <type> --sales-code <code>（按产品上下文自动选择 WEBAPP/MOBILEAPP）
# ✅ 创建应用 → scripts/app.sh create --product-type <type> --sales-code <code>（按产品上下文自动选择 WEBAPP/MOBILEAPP）
# ✅ 创建指定平台移动应用 → scripts/app.sh create --product-type apppay --sales-code I1080300001000041313 --mobile-platform IOS --bundle-id <bundleId>
# ✅ 创建 Android 移动应用 → scripts/app.sh create --product-type apppay --sales-code I1080300001000041313 --mobile-platform ANDROID --app-package <appPackage> --app-sign <appSign>
# ✅ 创建 iOS + Android 移动应用 → scripts/app.sh create --product-type apppay --sales-code I1080300001000041313 --mobile-platform ALL --bundle-id <bundleId> --app-package <appPackage> --app-sign <appSign>
# ✅ 设置公钥 → scripts/app.sh key <appId> <publicKey>
# ✅ 确认公钥 → scripts/app.sh verify-key <appId> [publicKey]
# ✅ 提交审核 → scripts/app.sh audit <appId>
```

其他未封装为独立脚本的 MCP 方法（queryApplicationInfo、queryApplicationDetail）请根据参数结构对照表构建调用。`queryApplicationSecurityKey` 的公钥确认、支付宝公钥导出已封装在 `scripts/app.sh verify-key|reuse|audit` 中。

### 禁止调用示例

```bash
# ❌ 最常见错误：添加 ctx，且省略必需的 appTypes
export PLATFORM=${DEV_TOOL_NAME} && alipay-cli mcp call apprelease.queryApplicationList \
  -d '{"request":{"pageSize":10,"pageNum":1},"ctx":{}}' \
  --json 2>/dev/null

# ❌ 错误：添加 ctx 参数（应用相关 MCP 调用不需要 ctx）
export PLATFORM=${DEV_TOOL_NAME} && alipay-cli mcp call apprelease.queryApplicationList \
  -d '{"request":{"appTypes":["WEBAPP"]},"ctx":{}}' \
  --json 2>/dev/null

# ❌ 错误：受 ar-sign 上下文影响使用错误的参数结构
export PLATFORM=${DEV_TOOL_NAME} && alipay-cli mcp call apprelease.queryApplicationList \
  -d '{"request":{"salesProductCodes":["<salesProductCode>"]},"ctx":{}}' \
  --json 2>/dev/null

# ❌ 错误：省略 request 包裹层
export PLATFORM=${DEV_TOOL_NAME} && alipay-cli mcp call apprelease.queryApplicationList \
  -d '{"appTypes":["WEBAPP"]}' \
  --json 2>/dev/null

# ❌ 错误：APP支付仍按 WEBAPP 查询/创建应用
export PLATFORM=${DEV_TOOL_NAME} && alipay-cli mcp call apprelease.queryApplicationList \
  -d '{"request":{"appTypes":["WEBAPP"]}}' \
  --json 2>/dev/null
```

### 应用模块与签约模块参数结构对比

| 模块 | 参数结构特征 | ctx 参数 | 示例 |
|------|-------------|----------|------|
| ar-sign / ar-query | `{"request":{...},"ctx":{}}` | ✅ 必须包含 | `{"request":{"salesProductCodes":["..."]},"ctx":{}}` |
| apprelease | `{"request":{...}}` | ❌ 不需要 | `{"request":{"appTypes":["WEBAPP"]}}` / `{"request":{"appTypes":["MOBILEAPP"]}}` |

**重要：应用发布模块的 MCP 调用参数结构与签约模块完全不同，必须严格按照上表执行，禁止混用！**

---

## 前置阶段 1: CLI 登录态检查

> 📎 登录态检查、登录授权流程详见 `authorization.md`，本文档不重复。

---

## 阶段 2: 查询已有应用

**⚠️ 重要：在创建应用前必须先查询当前主体下是否已有可复用的应用。** 执行 `scripts/app.sh list --product-type "$PRODUCT_TYPE" --sales-code "$SALES_CODE"`。

在 onboarding 中，本查询必须在登录授权成功后与签约状态查询连续执行，不等待签约提交。查询结果与签约状态合并展示，新建分支的条件资料也与未签约材料一次性收集。只读查询前置不代表可以提前创建应用；需要签约时仍先提交签约。

`productType` / `salesCode` 在此只用于校验上下文并选择 `WEBAPP` 或 `MOBILEAPP`。`queryApplicationList` 请求只按应用类型查询，因此返回项只能表述为“与当前产品所需应用类型匹配的候选应用”，不得表述为已绑定当前支付产品。

### 返回结果处理

`scripts/app.sh list --product-type "$PRODUCT_TYPE" --sales-code "$SALES_CODE"` 自动完成以下处理，输出应用列表并引导用户选择。`FLOW:CREATE_NEW` 只表示当前类型应用列表确实为空；`FLOW:SELECT` 表示存在可复用上线应用；`FLOW:PENDING_APPLICATIONS` 表示只存在不可复用的非上线应用，需要用户明确等待或新建：

保留本次查询的候选 `appId` 和实际状态，供用户序号选择和新建前重查比对。不保存与决策无关的完整接口响应。

| 结果 | 处理方式 |
|------|----------|
| 查询失败 | 直接返回错误并停止应用分支；仅当前会话仍有效的独立业务/服务/结构错误可继续其他只读查询、候选展示和独立资料采集。认证失败或授权不匹配按 `../flow.md` Step 3.1 立即停止查询组并重新授权；应用查询恢复前禁止应用复用或创建 |
| 命中 ON_LINE 应用 | 返回已有上线应用列表，让用户选择"复用上线应用"或"新建应用" |
| 无同类型应用 | 记录新建分支并一次性收集条件资料；onboarding 未签约时等待签约提交成功后再创建 |
| 存在同类型应用但均非 ON_LINE | 展示应用 ID、名称和实际状态，说明当前不可复用；等待用户选择暂不新建或明确新建，禁止自动进入创建 |

### ⛔ 应用复用强制原则

<important>
只有状态为 `ON_LINE` 的应用才允许作为可复用候选。非 `ON_LINE` 应用可以作为不可复用的状态提示展示，用于防止重复创建，但禁止出现在复用序号中。

- 可复用候选和复用序号只允许包含 `ON_LINE` 状态的应用
- 用户指定某个 `appId` 要复用时，必须先查询该应用状态
- 状态不是 `ON_LINE` 时，禁止复用
- 用户坚持要求复用非 `ON_LINE` 应用时，直接拒绝该复用请求；是否新建必须由用户在看到现有非上线应用后明确选择，不得自动进入新建
- 复用已有 `MOBILEAPP` 时不采集、不传入 `mobilePlatform`、`bundleId`、`appPackage`、`appSign`；平台字段只属于新建 `MOBILEAPP` 的 `createApplication` 入参
</important>

### 用户选择输出格式

```
📋 发现您已有以下上线应用：

| 序号 | 应用ID | 应用名称 | 状态 |
|------|--------|----------|------|
| 1 | 2021001234567890 | 我的网站应用 | 已上线 |
| 2 | 2021001234567891 | 我的测试应用 | 已上线 |

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  请选择操作：
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  【复用】输入序号 1 或 2 → 复用对应应用
  【新建】输入 "新建" → 创建新应用

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

展示候选应用时同步说明：选择“新建”后需要应用公钥；如果是 APP支付，还需要一次性提供平台及对应字段。用户可以在选择“新建”的同一条回复中提前提供这些字段，Agent 不得在每个字段之间逐轮询问。

### 复用成功后流程

```
用户选择复用 ON_LINE 应用
    ↓
调用 queryApplicationSecurityKey 获取安全密钥
    ↓
取 signType="RSA2" 项的 alipayPublicKey
    ↓
写入 ~/.config/<appId>-alipayPublicKey.keytext
    ↓
对客输出复用成功信息
    ↓
   结束
```

对按量付费产品，复用应用成功后仍需确认服务市场注册产物已经存在。应用只提供 `appId` 和支付宝公钥，不提供 `serviceId`。

### queryApplicationSecurityKey 返回结果处理

同一次查询只针对传入的一个 `appId`，但返回中的两类列表职责不同，禁止要求应用公钥和支付宝公钥出现在同一条记录中：

- 应用公钥配置状态：检查 `securityConfigList` 中 `signType="RSA2"` 的项，`partnerPublicKey` 非空即表示已配置；兼容旧返回中 RSA2 项的 `certInfoDTO.publicKey`。
- 支付宝公钥导出：检查 `alipayKeyList` 中 `signType="RSA2"` 的项并读取非空 `alipayPublicKey`。
- `securityKeys` 仅作为旧返回结构兼容来源。不得因为 `alipayKeyList[].certInfoDTO` 为空，就忽略 `securityConfigList[].partnerPublicKey` 并误判应用未配置公钥。

> 复用应用的完整处理（查询密钥 + 确认 RSA2 应用公钥 + 导出支付宝公钥）已封装在 `scripts/app.sh reuse <appId>` 中。应用公钥未确认时输出 `FLOW:REUSE_NO_KEY`；RSA2 项未返回 `alipayPublicKey` 时输出 `FLOW:ERROR`；已取得支付宝公钥但未能写入本地时，仍输出 `FLOW:REUSE_SUCCESS` 和 `ALIPAY_PUBLIC_KEY_EXPORT_STATUS=MANUAL_CONFIGURATION_REQUIRED`。

`FLOW:REUSE_NO_KEY` 是阻断结果，脚本返回非零退出码，禁止输出复用成功或把该 `appId` 记为可用于集成的完成产物。向用户合并说明当前所选 `ON_LINE` 应用尚未确认 RSA2 应用公钥，并让用户选择：

1. 为当前 `appId` 设置应用公钥：按“阶段 4: 设置应用公钥”的标准引导和确认规则执行 `scripts/app.sh key`，用户完成页面确认后自动执行 `scripts/app.sh verify-key`；确认成功后重新执行 `scripts/app.sh reuse <appId>`，由脚本再次校验应用状态并导出支付宝公钥。
2. 返回应用决策：重新选择其他 `ON_LINE` 应用、新建应用或暂不继续。

不得自动生成、补全或持久化用户公钥，不得在用户未选择设钥且未提供完整 `publicKey` 时调用 `createKeyConfirmPage`。重新执行 `reuse` 前不得推定应用仍为 `ON_LINE`；由脚本重新查询实际状态。

### 复用成功输出格式

```
✅ 应用复用成功

| 项目 | 信息 |
|------|------|
| AppId | <appId> |
| 应用状态 | ON_LINE（已上线） |
| 支付宝公钥本地配置 | <导出成功：~/.config/<appId>-alipayPublicKey.keytext；本地写入失败：待手动配置> |
```

### 按量付费集成产物

应用创建、审核提交或复用成功后，必须记录并向用户说明：

| 字段 | 用途 |
|------|------|
| `appId` | SDK 配置中的应用 ID，也是 `Payment-Needed.method.seller_app_id` |
| 支付宝公钥本地配置状态 | SDK 验签配置使用；导出成功时记录文件路径，已取得公钥但本地写入失败时记录“待手动配置” |
| 应用状态 | 只有 `ON_LINE` 应用允许复用；新建应用需按审核结果判断正式可用性 |

按量付费还需要服务市场注册产物 `serviceId`。如果只有 `appId` 而没有 `serviceId`，不能进入正式按量付费集成收口。

---

## 阶段 3: 创建应用

**⚠️ 仅在用户选择"新建应用"或无可复用应用时执行此阶段。**

### 创建前列表重查

在展示最终“创建应用”外部写操作摘要前，自动重新执行 `scripts/app.sh list --product-type "$PRODUCT_TYPE" --sales-code "$SALES_CODE"`：

- 候选 appId 和状态与 Step 3.1 一致：不重复询问用户，继续展示创建摘要。
- 出现新的 `ON_LINE` 或非 `ON_LINE` 应用，或原候选状态变化：展示最新候选，清除旧的应用决策，等待用户基于新状态选择复用、新建或暂不新建。
- 重查失败：禁止调用 `createApplication`，仅将应用分支标记为待恢复。

该重查只在 `CREATE` 分支执行，不增加正常复用分支的工具成本，也不替代外部写操作确认。

执行规则：

- 按量付费/网站支付：执行 `scripts/app.sh create --product-type "$PRODUCT_TYPE" --sales-code "$SALES_CODE"`。
- APP支付 iOS：执行 `scripts/app.sh create --product-type apppay --sales-code I1080300001000041313 --mobile-platform IOS --bundle-id "<bundleId>"`。
- APP支付 Android：执行 `scripts/app.sh create --product-type apppay --sales-code I1080300001000041313 --mobile-platform ANDROID --app-package "<appPackage>" --app-sign "<appSign>"`。
- APP支付 iOS + Android：执行 `scripts/app.sh create --product-type apppay --sales-code I1080300001000041313 --mobile-platform ALL --bundle-id "<bundleId>" --app-package "<appPackage>" --app-sign "<appSign>"`。

APP支付缺少平台或对应字段时，先向用户补问，禁止调用 `createApplication`。

**一次性新建资料包**：在 onboarding 中优先使用 Step 4 已收集并校验的字段，不重复询问：

- iOS：`mobilePlatform=IOS`、`bundleId`、应用公钥（如已准备）。
- Android：`mobilePlatform=ANDROID`、`appPackage`、`appSign`、应用公钥（如已准备）。
- iOS + Android：`mobilePlatform=ALL`、`bundleId`、`appPackage`、`appSign`、应用公钥（如已准备）。
- 按量付费/网站支付：应用公钥（如已准备）。

应用公钥仍必须由用户明确提供，且所有公钥私钥红线继续生效。用户尚未准备公钥时不得阻止应用创建，但创建成功后必须停在阶段 4；禁止在没有完整 `publicKey` 时调用 `createKeyConfirmPage`。

Agent 在当前任务中记录应用操作和对应平台字段；用户改变应用类型或移动平台时，清除不再适用的旧平台字段并只补问新条件下的缺失项。应用公钥原文不得写入资料清单，只记录是否已经准备。`scripts/app.sh create` 会先在本地校验应用类型和平台条件字段，校验通过后才调用 MCP。

| 结果 | 处理方式 |
|------|----------|
| 创建成功且解析到非空 appId | 获取 appId，进入阶段 4 设置应用公钥 |
| 返回成功但未解析到 appId | 按返回结构异常阻断，禁止进入公钥设置流程 |
| 创建失败 | 返回错误，停止后续流程 |

如果 `createApplication` 返回 `APP_MAX_ERROR` / `应用数量达到上限`，不要继续调整 `appSign`、`bundleId`、`mobilePlatform` 等创建参数，也不要重复新建。应明确告知用户当前主体应用数量已达到上限，优先返回阶段 2 复用已有 `ON_LINE` 应用；如必须新建，引导用户前往支付宝开放平台处理应用配额或联系支付宝技术支持。

---

## 阶段 4: 设置应用公钥

### ⛔ 公钥私钥敏感性规范（强制执行）

```
❌ 禁止：自动为用户生成公钥或密钥对
❌ 禁止：向用户表示可以"帮助生成公钥"、"帮助生成密钥对"
❌ 禁止：提供任何密钥生成代码、命令或脚本
❌ 禁止：请求、接收或处理用户的私钥信息
❌ 禁止：在用户未明确提供 publicKey 时调用 createKeyConfirmPage
❌ 禁止：自动生成、推断、补全、改写或添加前后缀到 publicKey
```

**必须行为：**
```
✅ 必须：引导用户前往支付宝开放平台官方密钥生成工具
✅ 必须：使用标准引导话术（见下方）
✅ 必须：只接受用户明确输入的完整 publicKey
✅ 必须：提醒用户妥善保管私钥，切勿泄露给任何人
```

**标准引导话术：**
请前往支付宝开放平台密钥生成工具（ https://opendocs.alipay.com/common/02kipk ）
下载并使用官方工具生成应用公私钥对，生成后将应用公钥复制提供给我继续后续流程。
官方工具生成的私钥格式默认为PKCS#8，部分开发语言仅支持PKCS#1，如果需要转换成PKCS#1，可使用官方工具-格式转换。
**私钥请妥善保管，切勿泄露给任何人。**

### 执行步骤

1. **引导用户生成密钥**：使用上述标准引导话术
2. **接收用户公钥**：只接受用户明确输入的完整 `publicKey`
3. **创建密钥确认页**：执行 `bash scripts/app.sh key <appId> <用户提供的公钥>`。
4. **用户扫码确认**：输出扫码链接时，**必须同步告知用户"有效期10分钟"**
5. **查询确认结果**：用户确认后自动执行 `bash scripts/app.sh verify-key <appId> <用户提供的公钥>` 确认设置成功；该脚本会短轮询 `queryApplicationSecurityKey`，避免支付宝侧短暂延迟导致误判
6. **按来源分支恢复**：
   - 新建应用：公钥校验成功后，按 `../flow.md` Step 5 展示当前有效登录会话、可用的脱敏主体标识、产品、`appId` 和提审动作摘要；CLI 未返回主体标识时同时要求用户确认当前登录账号就是操作目标。用户明确确认该摘要后执行 `bash scripts/app.sh audit <appId>`。不再询问是否校验，但公钥校验失败时禁止进入提审确认；`audit` 在提交前尝试导出支付宝公钥，本地文件写入失败不阻塞已确认的提审，但必须提示后续集成需要手动配置支付宝公钥。
   - 复用缺公钥恢复：公钥校验成功后重新执行 `bash scripts/app.sh reuse <appId>`，重新校验应用状态并导出支付宝公钥；不进入新建应用的提审分支。

### ⛔ createKeyConfirmPage 返回结果处理规范（最高优先级）

**`createKeyConfirmPage` 调用返回的结果中可能包含二维码链接和 `alipays://` 协议的深度链接，这些信息禁止展示给用户！**

```
❌ 禁止展示：调用返回中的 "或使用支付宝扫描以下二维码" 提示
❌ 禁止展示：调用返回中的 alipays:// 协议链接
❌ 禁止展示：调用返回中的任何二维码图片或二维码链接

✅ 必须：将 confirmPageUrl 以 markdown 链接格式输出给用户
✅ 必须：告知用户请点击链接或扫码确认公钥设置
✅ 必须：“无法跳链”提示放在确认链接前，确认链接独占一行
✅ 必须：等待用户确认后再继续后续步骤
```

**正确输出格式：**

```markdown
📋 应用公钥设置中...

请点击以下链接确认公钥设置：

提示：无法跳链时，请复制下方链接到网页浏览器打开。
[点击确认公钥设置](https://aipay.alipay.com/public-key-confirm?keyConfirmToken=xxx)

确认完成后，请告诉我"好了"继续后续流程。
```

确认链接必须独占一行并放在“无法跳链”提示之后，禁止将提示文案拼接到 Markdown 链接或 URL 后。

**禁止输出格式：**

```
❌ alipays://platformapi/startapp?appId=2018082061148052&page=/pages/public-key-upload/index?keyConfirmToken=xxx
```

**处理方式：**

> 📎 公钥设置调用 `bash scripts/app.sh key <appId> <publicKey>`，脚本自动提取 `confirmPageUrl` 并按标准 Markdown 格式输出。

### alipayPublicKey 本地写入失败非阻塞

以脚本输出为准记录导出结果：`ALIPAY_PUBLIC_KEY_EXPORT_STATUS=EXPORTED` 时同时记录 `ALIPAY_PUBLIC_KEY_FILE`；`ALIPAY_PUBLIC_KEY_EXPORT_STATUS=MANUAL_CONFIGURATION_REQUIRED` 只表示已取得支付宝公钥但本地写入失败，正式集成前需手动配置。RSA2 项未返回 `alipayPublicKey` 时直接报错，不得记录为导出失败。禁止根据固定路径推断文件已经存在。

```bash
# 导出路径，由 app.sh reuse/audit 自动写入
~/.config/${APP_ID}-alipayPublicKey.keytext

# 导出时机
# 1. 复用已有上线应用时
# 2. 提交应用审核前
```

---

## 阶段 5: 提交应用审核

**⚠️ 只有确认应用公钥存在后，才继续提交审核。`scripts/app.sh verify-key` 和 `scripts/app.sh audit` 会通过 `queryApplicationSecurityKey` 判断 RSA2 公钥配置：**

执行 `audit` 前还必须按 `../flow.md` Step 5 展示当前有效登录会话、可用的脱敏主体标识、产品、`appId` 和“提交应用审核”动作摘要，并取得针对该摘要的明确执行确认。CLI 未返回主体标识时，摘要必须要求用户确认当前登录账号就是操作目标。应用公钥页面的“好了”只表示外部公钥确认完成，不等于授权提交应用审核。

- 应用公钥状态与支付宝公钥必须分别按本模块“queryApplicationSecurityKey 返回结果处理”解析：RSA2 `securityConfigList[].partnerPublicKey` 非空即表示应用公钥已配置，同时兼容旧返回中的 `certInfoDTO.publicKey`；支付宝公钥只从 RSA2 `alipayKeyList[].alipayPublicKey` 读取。
- `alipayPublicKey` 是支付宝公钥，只用于导出验签配置，不得用于证明应用公钥已生效。
- `verify-key` 默认最多重试 6 次、每次间隔 5 秒；可通过 `APP_KEY_VERIFY_RETRIES` 和 `APP_KEY_VERIFY_INTERVAL_SECONDS` 调整。
- 应用公钥已确认但 RSA2 项未返回 `alipayPublicKey` 时，直接返回错误，不继续提审。
- 校验通过后由 `audit` 在提交审核前尝试写入 `~/.config/<appId>-alipayPublicKey.keytext`。
- 已获取 `alipayPublicKey` 但本地文件写入失败时，按本地写入失败处理并重试；重试后仍失败则输出同一待配置状态，但不阻塞提审或复用成功路径。

执行 `bash scripts/app.sh audit <appId>`。

| 结果 | 处理方式 |
|------|----------|
| 提审成功 | 输出提审成功结论，不对客展示审核单号 |
| 提审失败 | 返回错误，停止后续流程 |

---

## 输出格式

### 复用应用成功

```
✅ 应用复用成功

| 项目 | 信息 |
|------|------|
| AppId | <appId> |
| 应用状态 | ON_LINE（已上线） |
| 支付宝公钥本地配置 | <导出成功：~/.config/<appId>-alipayPublicKey.keytext；本地写入失败：待手动配置> |
```

### 新建应用成功

```
✅ 应用创建成功，已提交审核

| 项目 | 信息 |
|------|------|
| AppId | <appId> |
| 应用状态 | AUDITING（审核中） |
| 支付宝公钥本地配置 | <导出成功：~/.config/<appId>-alipayPublicKey.keytext；本地写入失败：待手动配置> |
```
