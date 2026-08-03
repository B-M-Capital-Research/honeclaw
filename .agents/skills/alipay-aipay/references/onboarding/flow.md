# 签约流程说明

> ⚠️ **前置声明**：本 flow 仅支持**网站支付、APP支付、按量付费**三种产品的签约。**其他产品（当面付、订单码支付、JSAPI支付、预授权支付、商家扣款等）暂不支持**，如需签约请前往[支付宝商家平台](https://b.alipay.com/page/portal/home)完成签约。

---

## 内容导航

- [文件路由表](#-文件路由表)
- [核心铁律](#-核心铁律签约流程强制遵守)
- [产品映射](#一产品映射)
- [主流程](#二主流程6步)
- [Step 详解](#三step-详解)
- [错误处理](#四错误处理全局规则)
- [流程状态](#五流程状态)

## 📂 文件路由表

> 本文件位于 `references/onboarding/`。下表中的 `modules/...`、`../normal/...` 路径均相对本文件所在目录；若从技能包根目录定位，请加前缀 `references/onboarding/`。

| 文件 | 说明 | 引用位置 |
|------|------|----------|
| **flow.md** | ⭐ 主入口 - 签约流程全览（本文档） | - |
| **../normal/** | 通用模块 | - |
| ↳ ../normal/alipay-cli-env.md | alipay-cli 检测与安装 | Step 1 环境检查 |
| ↳ ../normal/rejection-guide.md | 拒绝引导话术 | 通用 |
| ↳ ../normal/scripts/detect_dev_tool.sh | AI编程工具检测 | Step 1 |
| ↳ ../normal/scripts/common.sh | shell 公共函数与初始化入口 | 签约脚本公共依赖 |
| **modules/** | 子模块目录 | - |
| ↳ modules/mcc-reference.md | MCC类目参考表 | Step 2 方案规划 |
| ↳ modules/authorization.md | 登录授权模块 | Step 3 登录授权 |
| ↳ modules/product-sign.md | 签约模块 | Step 3.1 签约查询 + Step 5.1 签约提交 |
| ↳ modules/service-registration.md | 服务市场注册模块 | Step 3.1 按量付费服务查询 + Step 5.2 写操作 |
| ↳ modules/app-release.md | 应用发布模块 | Step 3.1 应用查询 + Step 5.3 后续操作 |
| ↳ modules/error-handling.md | 错误处理说明 | 全流程 |
| **modules/scripts/** | 签约流程脚本 | - |
| ↳ modules/scripts/auth.sh | 登录授权全流程 | Step 3 |
| ↳ modules/scripts/query_sign_status.sh | 签约状态查询 | Step 3.1 |
| ↳ modules/scripts/upload_screenshots.sh | 截图上传 | Step 4 |
| ↳ modules/scripts/ar_sign_apply.sh | 签约提交 | Step 5.1 |
| ↳ modules/scripts/service.sh | 服务查询与注册 | Step 3.1 查询 + Step 5.2 写操作 |
| ↳ modules/scripts/app.sh | 应用查询与发布 | Step 3.1 查询 + Step 5.3 后续操作 |
| ↳ modules/scripts/error_handler.sh | 统一错误检测（共享库） | 全流程 |

> 📌 **使用建议**：从 flow.md 主入口开始，按流程顺序执行各 Step。各 Step 中已标注对应的脚本调用命令。

---

## ⛔ 核心铁律（签约流程强制遵守）

> ⚠️ 以下铁律为签约流程的最高优先级规则，所有 Step 执行必须遵守。详细说明见各模块文档。

### 1. 应用公钥管理铁律
**核心原则：公钥由用户自行生成，skill 只负责接收和配置**
```
❌ 禁止：自动为用户生成公钥或密钥对
❌ 禁止：向用户表示可以"帮助生成公钥"、"帮助生成密钥对"
❌ 禁止：提供任何密钥生成代码、命令或脚本
❌ 禁止：请求、接收或处理用户的私钥信息
❌ 禁止：在用户未明确提供 publicKey 时调用 createKeyConfirmPage
❌ 禁止：自动生成、推断、补全、改写或添加前后缀到 publicKey
✅ 必须：引导用户前往支付宝开放平台官方密钥生成工具
✅ 必须：只接受用户明确输入的完整 publicKey
✅ 必须：提醒用户妥善保管私钥，切勿泄露给任何人
```
> 详见 `modules/app-release.md`

### 2. 用户项目文件修改铁律
```
❌ 禁止：直接修改用户项目中的 .ts / .tsx / .js 文件
❌ 禁止：修改 alipay-sdk-config.ts 或类似的支付配置文件
✅ 正确：输出配置内容让用户自行复制
✅ 正确：告知用户需要修改的文件路径和具体内容
```

### 3. Agent 话术约束铁律（全局）
**本铁律适用于签约全流程所有需要输出引导话术的场景**
```
🚨 【强制】Agent 话术约束（最高优先级）：
❌ 禁止：篡改各模块标准引导话术中的链接、格式或内容
❌ 禁止：添加标准话术中没有的内容（如"PEM格式"、"-----BEGIN PUBLIC KEY-----"等）
❌ 禁止：替换标准话术中的官方链接（如将 opendocs.alipay.com 替换为其他链接）
❌ 禁止：自行创造、补充、解释任何未见于标准话术的格式说明
✅ 必须：100% 复制使用各模块文档中的「标准引导话术」原文
✅ 必须：当需要补充说明时，只能引用官方文档链接

📌 典型反例：
❌ 错误：将 https://opendocs.alipay.com/common/02kipk 替换为 https://open.alipay.com/keytool
❌ 错误：在引导话术中添加"公钥以 -----BEGIN PUBLIC KEY----- 开头"
❌ 错误：自行解释"PEM格式"、"PKCS#1/PKCS#8"等格式概念
```
> 详见各模块文档中的「标准引导话术」

### 4. CLI环境变量铁律
```
✅ 必须：在每个 alipay-cli 调用前设置 PLATFORM 环境变量
✅ 必须：DEV_TOOL_NAME 由脚本公共初始化获得，缺失时填 "unknown"
❌ 禁止：省略 PLATFORM 环境变量直接调用 alipay-cli
❌ 禁止：硬编码 PLATFORM 值（如 PLATFORM=claudeCode）
❌ 禁止：使用分号 ; 连接（可能导致环境变量泄露）
```

### 5. 授权链接铁律
**CLI 的 `login` 命令返回 `verification_url` 字段，此链接无法用于授权，禁止透出给用户！**
```
❌ 禁止透出：https://opengw.alipay.com/oauth/device（此链接无法授权）
✅ 正确链接：https://aipay.alipay.com/cli-auth?deviceCode=xxx&productCode=xxx&mccCode=xxx（可选追加 platform=xxx）
✅ 必须：先单独提示“无法跳链时，请复制下方链接到网页浏览器打开”，授权链接独占下一行
```
> 详见 `modules/authorization.md`

### 6. createKeyConfirmPage 返回处理铁律
**返回结果中禁止展示二维码链接和 `alipays://` 协议链接，只展示 `confirmPageUrl` 的 Markdown 链接。**
```
❌ 禁止展示：二维码提示、alipays:// 协议链接
✅ 必须：将 confirmPageUrl 以 Markdown 链接格式输出
✅ 必须：先单独提示“无法跳链时，请复制下方链接到网页浏览器打开”，确认链接独占下一行
✅ 必须：等待用户确认后再继续后续步骤
```
> 详见 `modules/app-release.md`

### 7. 授权范围不满足处理铁律
**当检测到授权范围不满足时，必须执行 logout 退出登录，然后重新授权。**
```
❌ 禁止：检测到授权范围不满足后继续执行后续操作
❌ 禁止：不执行 logout 直接重新登录
✅ 必须：检测到后先 logout 再重新授权
✅ 必须：重新授权时使用正确的 scope（根据当前 salesCode 确定）
```
> 详见 `modules/authorization.md` §8.1：检测脚本只返回不匹配信号并停止当前操作；`modules/scripts/auth.sh mismatch` 统一执行一次 logout，再使用正确 scope 发起重新授权

### 8. 模块强读铁律
**执行任何模块操作前，必须读取当前操作对应的 references 章节并覆盖该分支的完整参数、约束和错误处理，禁止凭记忆执行；无关产品和尚未进入的写操作章节不提前读取。**
```
✅ 签约查询/提交前 → 分别读取 modules/product-sign.md 中对应章节
✅ 应用查询/复用/创建/提审前 → 分别读取 modules/app-release.md 中对应章节
✅ 服务查询/复用/创建/修改前 → 分别读取 modules/service-registration.md 中对应章节
✅ 登录授权前 → 读取 modules/authorization.md 中当前授权操作所需章节
❌ 禁止：不读文档直接调用任何 MCP 方法
❌ 禁止：凭记忆猜测或推断 MCP 方法名、参数名、参数结构
❌ 禁止：自行编造文档中不存在的 MCP 方法
```

### 9. MCP调用铁律
**签约/查询模块与应用/服务模块的参数结构完全不同，禁止混用！**
```
✅ 签约/查询用 ctx：{"request":{...},"ctx":{}}
✅ 应用/服务不用 ctx：{"request":{...}}
❌ 禁止：将签约模块的 ctx 习惯带入应用发布模块
❌ 禁止：将应用发布模块的无 ctx 习惯带入签约模块
❌ 禁止：省略 ctx:{}（签约模块）
❌ 禁止：添加 ctx（应用/服务模块）
```

### 10. 依据不足时不编造
```
✅ 必须：没有确定依据时明确说明无法确认
✅ 签约、费率、额度解限、可收款时间解限等不确定问题 → 引导用户前往支付宝商家平台咨询客服
✅ 代码集成、应用创建、应用发布等不确定问题 → 引导用户前往支付宝开放平台或支付宝技术支持咨询客服
❌ 禁止：猜测签约审核规则、额度解限条件、可收款时间、费率优惠、应用创建限制或接口能力
```

商家平台咨询入口：[支付宝商家平台](https://b.alipay.com/page/portal/home)

开放平台咨询入口：[支付宝开放平台](https://open.alipay.com/) / [支付宝技术支持](https://opensupport.alipay.com/support/intelligent-services?form=payskill)

---

## 一、产品映射

> 术语说明：本 Skill 中“网站支付”是唯一的网页支付产品概念，覆盖电脑网页和手机浏览器网页/H5 场景；“电脑网站支付”“PC网站支付”“手机网站支付”“H5支付”均按网站支付处理。签约 payload 中的 `appType=PC_WEB` 是接口字段名，不代表只能用于 PC 端网页。

| 产品 | productType | salesCode | scope | 资料采集要求 |
|------|-------------|-----------|-------|-------------|
| 按量付费 | aipay | I1080300001000160457 | app:all,machine_pay:write,agmnt:write | 无需截图 |
| 网站支付 | webpay | I1080300001000041203 | app:all,fast_instant_trade_pay:write | 需要3张网站截图 |
| APP支付 | apppay | I1080300001000041313 | app:all,auth_alipay_apppay:write | 未签约时需要 APP 名称和3张APP界面截图，签约状态固定按 `OFFLINE` 提交；选择新建应用时在 Step 4 前置采集移动平台信息，Step 5.3 才用于创建 |

---

## 二、主流程（6步）

```
Step 1: 环境检查 → Step 2: 方案规划 → Step 3: 登录授权
    → Step 3.1: 状态与资源查询 → Step 4: 一次性资料与资源决策
    → Step 5: 入驻推进 → Step 6: 本轮流程收口
```

### 启动前材料预告与分流

进入 onboarding 后才预告可能需要的材料，不在登录前要求用户确认材料齐备。签约材料以“一、产品映射”为准，新建应用的条件资料以 Step 4“应用分支”为准；登录查询实际状态后再一次性收集当前分支所需材料。`full_process` 在主编排的项目/集成状态检查阶段以及 integration 阶段均不得提前读取或展示本节材料。

**产品尚未明确时的预告话术**：

> "在确定产品前，我先说明材料差异：按量付费无需截图；网站支付需要网站首页、商品页、支付页 3 张截图；APP支付需要 APP 名称，以及首页、商品页、支付页 3 张截图。确定产品后，我会先查询实际签约和应用状态，再只收集当前分支需要的材料。"

**执行规则**：
- 产品尚未明确时使用上述话术；产品已明确时不再展示三产品通用话术，Step 2 只预告当前产品的条件材料。两种情况都不得在 Step 3.1 查询前要求用户提交或确认全部材料。
- `full_process` 在进入 onboarding 前必须已经确定目标产品，因此不得使用“产品尚未明确”的三产品通用材料话术，只展示当前目标产品的签约信息。
- 网站支付或 APP支付确定为 `NOT_SIGNED` 且尚未完成支付页面或支付能力集成时，暂停签约写操作，切换到集成流程。“尚未完成”的依据仅限：`full_process` 用户明确更正统一分流确认中的集成状态；`onboarding_only` 用户明确声明目标产品尚未完成集成；或 Agent 按 `../../SKILL.md` 的目标项目检查规则重新检查项目代码后取得直接证据。用户声明必须标注为“用户声明”，不得表述为代码检测结果；签约状态、应用列表和服务列表只说明 onboarding 资源状态，不能单独证明代码未集成。若页面和支付能力已存在、仅缺签约申请需要上传的首页、商品页、支付页图片，则保留在 Step 4 收集并上传，不进入集成流程；已签约时不再为签约要求这组图片。
- 截图要求以“支付页”为准；不得自行改口为“支付成功页”或要求用户提供文档未定义的页面。
- `full_process` 按 `../../SKILL.md` 的“完整接入编排”完成二维项目状态判断和统一分流确认；只有用户确认所选项目为 `TARGET_COMPLETE`，或本轮 integration 全部完成后才能进入 onboarding。进入后复用已经取得的实际材料，不重复询问。

---

## 三、Step 详解

### Step 1: 环境检查

**必读文档**：
- `../normal/alipay-cli-env.md` - alipay-cli 检测、安装、验证规范

**脚本**：
- `../normal/scripts/detect_dev_tool.sh` - 检测 AI 编程工具，输出 DEV_TOOL_NAME 取值
- `../normal/scripts/common.sh` - shell 公共函数与初始化入口；签约脚本通过 `error_handler.sh` 间接调用 `init_dev_tool_name`

**任务**：
1. 检查 alipay-cli 是否安装，未安装则自动安装
2. 确认 AI 编程工具检测脚本可用；签约脚本执行时会通过公共初始化设置 DEV_TOOL_NAME
3. 有任务工具时创建7个任务：环境检查、方案规划、登录授权、状态与资源查询、一次性资料与资源决策、入驻推进、本轮流程收口；没有 `TaskCreate` / `TaskUpdate` 时，不在本步骤单独输出待办，直接把 Step 2 一次性方案确认中的完整待办作为用户可见清单并在后续更新
4. 初始化内存状态

工具能力差异不得导致任务、步骤或完成状态被省略。

### Step 2: 方案规划

**必读文档**：
- `modules/mcc-reference.md` - MCC类目参考表

**产品匹配规则**：

| 场景关键词 | 推荐产品 |
|-----------|----------|
| AI、智能体、大模型、Agent、MCP | 按量付费 |
| 网站、网页、PC、电脑、电商、商城、H5 | 网站支付 |
| APP、应用内支付、手机APP | APP支付 |

产品确定后，`salesCode`、`scope` 和材料要求统一取自“一、产品映射”，禁止在本步骤另建映射或改写固定值。

**MCC类目**：
- 读取 `modules/mcc-reference.md` 进行语义匹配
- 示例：互联网综合电商平台 (A0002_B0114)
- mccCode 格式：`Axxxx_Bxxxx`

**一次性方案确认**：

完成产品与 MCC 匹配后，在执行 `auth.sh init` 前一次性展示：

1. 产品名称、`salesCode` 和授权 `scope`。
2. MCC 名称与编码。
3. 当前产品在“未签约”或“需要新建应用”时可能需要的条件材料；不在状态查询前要求用户确认材料齐备状态。
4. 已识别的接入意图和完整待办步骤。
5. 即将生成支付宝授权链接，以及当前流程后续实际适用的外部授权或公钥确认。不得在此混入或重复展示集成流程、沙箱付款动作或集成待办。

只集中补问无法可靠识别的产品、业务类型或 MCC。用户已经明确提供的信息不得重复询问，但未经用户明确确认不得执行 `login`。登录后必须先执行 Step 3.1 的只读查询，再根据实际结果收集签约、服务和应用分支的必要资料。

<INTERNAL_BLOCKING_CONFIRMATION>

必须等待用户明确确认产品、MCC、授权范围和签约执行范围。确认信息有误时先更新并重新展示受影响内容。`full_process` 的项目状态回答、integration 产品确认和服务声明确认均不能替代本确认；已经取得材料时，继承实际图片引用、APP 名称、缺失项和校验结果，不重复采集，但仍需完成本步骤的签约方案确认。缺少实际入参或校验证据时必须补齐。产品、MCC 或执行范围发生变化时必须重新确认受影响内容。

</INTERNAL_BLOCKING_CONFIRMATION>

### Step 3: 登录授权

**必读文档**：
- `modules/authorization.md` - 登录授权模块

**脚本**：
- `modules/scripts/auth.sh` - 登录授权全流程（init / confirm / mismatch）

**授权前确认继承**：Step 2 的一次性方案确认满足 `modules/authorization.md` 第三节的授权前确认要求。必须确认同一产品和 MCC 已得到用户明确确认；没有有效确认时仍禁止执行 `login`，不得因 `full_process` 阶段衔接绕过该红线。

**完整流程**：
```
1. bash modules/scripts/auth.sh init --scope "$SCOPE" --sales-code "$SALES_CODE" --mcc-code "$MCC_CODE" --product-name "$PRODUCT_NAME" --mcc-name "$MCC_NAME"
   ├─ AUTH_FLOW:SKIP → 当前登录有效且目标产品 scope、MCC 均已校验通过，进入 Step 3.1
   ├─ AUTH_FLOW:READY → 未登录，自动 login + 输出授权信息表格，再执行第 2-3 项
   ├─ AUTH_FLOW:SCOPE_MISMATCH / MCC_MISMATCH → 停止当前操作，执行第 4 项
   └─ AUTH_FLOW:FAILED → 停止并按错误输出恢复，禁止进入 Step 3.1
2. 仅 AUTH_FLOW:READY：等待用户回复"好了"确认授权完成
3. 仅 AUTH_FLOW:READY：bash modules/scripts/auth.sh confirm --scope "$SCOPE" --sales-code "$SALES_CODE" --mcc-code "$MCC_CODE" --product-name "$PRODUCT_NAME" --mcc-name "$MCC_NAME"
   ├─ AUTH_FLOW:AUTH_SUCCESS → 进入 Step 3.1
   ├─ AUTH_FLOW:PENDING → 等待用户完成授权，禁止自动轮询
   ├─ AUTH_FLOW:EXPIRED → 重新执行第 1 项生成授权链接
   ├─ AUTH_FLOW:FAILED → 停止并按错误输出恢复，禁止进入 Step 3.1
   └─ AUTH_FLOW:SCOPE_MISMATCH / MCC_MISMATCH → 停止当前操作，执行第 4 项
4. bash modules/scripts/auth.sh mismatch --scope "$SCOPE" --sales-code "$SALES_CODE" --mcc-code "$MCC_CODE" --product-name "$PRODUCT_NAME" --mcc-name "$MCC_NAME"
   └─ 由 mismatch 统一执行一次 logout；成功后使用正确 scope 重新生成授权链接
```

> `confirm` / `mismatch` 优先使用显式传入的非敏感授权上下文。这样即使 Agent 在沙箱化执行环境和可联网执行环境之间切换，也不依赖临时状态文件是否可见。

授权 `scope` 统一使用“一、产品映射”中当前产品的固定值，禁止自行修改。

**⚠️ 授权信息展示规范**：执行前必须读取 `modules/authorization.md` 第 5.2～6.2 节，并原样展示 `auth.sh init` 输出，禁止简写、重排或改写。该章节是完整约束来源：产品类型、经营类目、确认码和有效期四项必须展示；必须校验 `deviceCode` / `productCode` / `mccCode` 完整且仅把 `platform` 作为可选参数；禁止展示 CLI `verification_url`、半成品或裸链接；确认码只用于核对，禁止引导用户输入验证码。

#### Step 3.1: 状态与资源查询（登录授权后置检查）

**只读查询组**：登录成功后自动连续执行，不增加“是否查询”确认：

查询前按当前产品读取最小必要章节：`modules/product-sign.md` 的“签约状态查询”、`modules/app-release.md` 的“阶段 2: 查询已有应用”；按量付费同时读取 `modules/service-registration.md` 的 Step 1、Step 2 和“服务状态说明”。Step 5 再按实际写操作分支读取对应模块的提交、创建、修改、公钥或审核章节，不提前加载无关写操作内容，也不重复读取已经读过且未变化的查询章节。

1. 执行 `bash modules/scripts/query_sign_status.sh --sales-code "$SALES_CODE" --product-type "$PRODUCT_TYPE"` 查询当前产品签约状态。
2. 执行 `bash modules/scripts/app.sh list --product-type "$PRODUCT_TYPE" --sales-code "$SALES_CODE"` 查询当前登录账号下与当前产品所需应用类型匹配的候选应用。该接口按 `WEBAPP` / `MOBILEAPP` 查询，不得宣称返回应用已与当前支付产品绑定。
3. 仅按量付费再执行 `bash modules/scripts/service.sh list` 查询已有服务。

> ⚠️ 脚本通过 `error_handler.sh` 间接初始化 `DEV_TOOL_NAME`；签约查询必须传入当前产品的 `--sales-code`，建议同时传入 `--product-type`（aipay|webpay|apppay）做一致性校验。网站支付和 APP支付使用不同 salesCode，禁止混用。

各适用查询仍是独立 CLI/MCP 调用，必须分别完成原有错误检测，禁止伪造合并 MCP。失败后按错误类型处理：

- MCP 认证失败或产品/scope/MCC 授权不匹配：立即停止查询组，不再调用剩余查询；授权不匹配时调用 Step 3 的 `auth.sh mismatch`，由该命令统一 logout 并重新授权。重新授权后仅恢复尚未执行的查询，保留重新授权前已经成功取得且主体未变化的结果。若无法确认仍为同一账号主体，则重新执行全部适用查询。
- 单个业务失败、响应结构错误或 MCP 服务失败，但当前会话仍有效：把该分支标记为查询失败，继续其他不依赖该结果的只读查询，最后一次性输出“已取得结果 + 失败分支”摘要。不得把失败视为空列表；应用查询失败时禁止应用复用/创建，服务查询失败时禁止服务复用/新建/修改。
- 签约查询失败：可继续应用/服务查询、候选展示、资源决策及其独立资料采集；不得推断签约状态，不得采集签约截图或签约所需 APP 名称，`signStatus` 和签约 `materialStatus` 保持未设置，并暂停 Step 5 的全部 onboarding 外部写操作。签约查询成功后，只有状态为 `NOT_SIGNED` 才补充采集对应签约材料，再恢复尚未执行的分支。

任何恢复都不得要求用户重交未变化且已验证的资料，也不得重做不受会话主体变化影响的成功步骤。

签约状态和 `FLOW:*` 映射只按 `modules/product-sign.md` 的“签约状态查询”执行：`SIGNED_EFFECTIVE` 与 `SIGN_SUBMITTED` 均禁止重复提交签约；`OTHER_STATUS` 不提交签约，只展示已确认结果和待核验项。Step 4 再根据脚本输出的实际状态收集签约材料并处理应用/服务候选。

### Step 4: 一次性资料与资源决策

将 Step 3.1 已成功取得的签约状态、应用候选和按量付费服务候选合并展示，并计算当前已知分支的必需资料。用户应能在一条回复中完成服务/应用选择和所有可确定的必要资料提供；只补问缺失或校验失败字段。签约状态尚未取得时，签约资料需求仍未知，不得提前采集或混入应用/服务资料包。

**签约分支**：

- `NOT_SIGNED` 网站支付：一次收集首页、商品页、支付页 3 张截图。
- `NOT_SIGNED` APP支付：一次收集 APP 名称和首页、商品页、支付页 3 张截图，签约状态仍固定按 `OFFLINE` 提交。
- `NOT_SIGNED` 按量付费：签约申请无需上传页面图片。
- `SIGNED_EFFECTIVE` / `SIGN_SUBMITTED`：不再为签约采集 APP 名称或截图，禁止重复提交签约。

**应用分支**：

- 有 `ON_LINE` 候选时，让用户选择复用序号或新建；复用时不采集新建应用字段。
- 只有非 `ON_LINE` 应用时，展示实际状态并让用户选择暂不新建或明确新建；禁止自动新建。
- 列表确实为空或用户选择新建时，按量付费/网站支付一次说明应用公钥要求；APP支付一次收集 `mobilePlatform` 及对应的 `bundleId` / `appPackage` / `appSign`，并同时说明应用公钥要求。这些平台字段可在签约提交前采集和校验，但只能在 Step 5.3 创建应用时使用。
- 应用公钥原文只能由用户明确提供，不得生成、推断、改写或持久化到资料状态；可在资料状态中只记录“已准备/未准备”。未准备公钥不阻止应用创建，但创建后必须停在设置公钥阶段。

**按量付费服务分支**：完整展示服务查询返回的候选及实际状态，不额外按状态过滤复用候选。让用户选择复用序号、新建，或使用 serviceId 修改已有服务。新建/修改时在同一条回复中收集完整五项服务资料；不得自行增加接口文档未定义的服务状态限制。

APP支付未签约时的签约字段：
| 字段 | 必填条件 | 说明 |
|------|----------|------|
| APP名称 | 必填 | 传入 `ar_sign_apply.sh --app-name` |
| APP界面截图 | 必填 | 3张截图，支持拖拽上传或提供本地文件路径，顺序为首页、商品页、支付页 |

**脚本**：
- `bash modules/scripts/upload_screenshots.sh <img1> <img2> <img3>` - 上传3张截图；用户可拖拽上传图片，Agent 将拖拽后的本地文件路径传给脚本

上传成功后按首页、商品页、支付页顺序记录三个图片引用；APP支付同时保留已校验的 APP 名称。

### Step 5: 入驻推进

**按分支读取文档**：
- `NOT_SIGNED`：读取 `modules/product-sign.md` 的签约提交、提交后推进和错误处理章节。
- 按量付费且处理服务：读取 `modules/service-registration.md` 中当前决策对应的收集、校验、提交或复用章节。
- 处理应用：读取 `modules/app-release.md` 中当前决策对应的复用、创建、公钥设置或提审章节。
- 已在 Step 3.1 读取且内容未变化的查询章节不重复读取；不适用的产品或分支不读取。

**外部写操作确认**：执行本步骤中的 `ar_sign_apply.sh`、`service.sh save`、`app.sh create`、`app.sh key` 或 `app.sh audit` 前，必须展示当前有效登录会话、可用的脱敏主体标识、产品、动作和目标资源摘要。签约摘要包含 MCC 和材料状态；服务摘要包含目标 `serviceId`（新建时标明新建）、服务名称、地址和单价；应用创建摘要包含应用类型和适用的平台字段；设置应用公钥只展示 `appId`，禁止回显公钥原文；应用提审展示 `appId`。不得展示私钥、token、Payment-Proof、截图内容或完整签名串。

<INTERNAL_BLOCKING_CONFIRMATION>

只有用户在看到同一有效登录会话、产品、动作和目标摘要后明确确认，才能执行对应写操作。CLI 未返回主体标识时，必须明确提示并让用户确认当前登录账号就是操作目标。用户对该摘要的资源选择或明确执行回复可以直接复用；启动确认、其他目标或动作的确认、沉默和仅提供资料均不能替代。登录会话、主体、产品、动作、目标或关键金额变化时必须重新确认。

</INTERNAL_BLOCKING_CONFIRMATION>

写操作失败或响应不明时，先查询实际状态再决定是否重试，禁止直接重复提交。只有相邻写操作的目标和摘要都已经确定时才能合并确认；应用创建时尚不存在的 `appId`、公钥确认结果或审核状态不得提前授权。

**写操作顺序**：Step 3.1 的应用/服务查询和 Step 4 的资料采集只是前置发现与准备，不改变原写操作顺序。只有签约查询成功并得到 `NOT_SIGNED`、`SIGNED_EFFECTIVE` 或 `SIGN_SUBMITTED` 后，才允许执行 Step 5 的 onboarding 外部写操作；查询失败或状态为 `OTHER_STATUS` 时保持全部写操作阻断并先核验签约状态。当 `signStatus=NOT_SIGNED` 时，必须先完成 5.1 签约提交，成功后才执行 5.2 服务创建/修改和 5.3 应用复用/创建等操作。已签约或已提交签约时跳过 5.1。

**按量付费入驻产物关系**：
- 产品签约负责开通按量付费能力，使用 `salesCode=I1080300001000160457` 和 `scope=app:all,machine_pay:write,agmnt:write`。
- 服务市场注册负责生成或复用按量付费服务，服务信息中的服务单价、服务地址和请求示例用于正式环境配置与收银。
- 应用发布负责生成或复用 `WEBAPP` 应用，并取得正式集成所需的 `appId`、应用私钥对应的应用公钥配置状态、支付宝公钥。
- 正式环境按量付费配置中，`sellerId` 来自商户 PID/2088，服务市场注册结果中的真实 `serviceId` 作为正式环境产物保留，在跨流程上下文中标记为 `productionServiceId`（仅用于区分用途，不改变 MCP 的 `serviceId` 字段名）；`appId/alipayPublicKey` 来自应用发布或复用结果。`full_process` 进入 onboarding 前已经完成 integration 沙箱联调，取得这些正式产物后不得重新进入 integration 或重复沙箱测试，只需把生产配置替换列为上线待办。`onboarding_only` 后续另行进入 integration 时，沙箱联调仍固定使用 `api_mock_service_id`，不得使用 `productionServiceId` 或再次向用户索要 `serviceId`；正式上线前再替换为已保留的 `productionServiceId`。


#### 5.1 产品签约

仅 `NOT_SIGNED` 执行本节；其他状态禁止重复提交。按 `modules/product-sign.md` 的“签约提交”使用当前产品、MCC 和已验证材料执行；完整参数、固定字段、FLOW 信号和错误处理只在该模块维护。

**固定命令契约**：按当前产品原样使用对应命令结构，只替换尖括号中的实际值，禁止增删、改名或猜测参数：

```bash
# 按量付费
bash modules/scripts/ar_sign_apply.sh --product aipay --sales-code "I1080300001000160457" --mcc-code "<mccCode>"
# 网站支付
bash modules/scripts/ar_sign_apply.sh --product webpay --sales-code "I1080300001000041203" --mcc-code "<mccCode>" --picurl1 "<imageRef1>" --picurl2 "<imageRef2>" --picurl3 "<imageRef3>"
# APP支付
bash modules/scripts/ar_sign_apply.sh --product apppay --sales-code "I1080300001000041313" --mcc-code "<mccCode>" --app-name "<APP名称>" --picurl1 "<imageRef1>" --picurl2 "<imageRef2>" --picurl3 "<imageRef3>"
```

提交成功后不等待或轮询签约生效：按量付费继续 5.2 和 5.3，网站支付/APP支付继续 5.3。复用 Step 3.1 和 Step 4 的成功结果；只在会话、主体、候选资源变化或原查询失败时刷新受影响分支，刷新失败不清除其他成功结果。

#### 5.2 服务市场注册（仅按量付费）

复用 Step 3.1 候选，按 `modules/service-registration.md` 执行复用、创建或修改分支。创建/修改前必须用完整五项资料通过该模块定义的 `validate`；只补齐或重新校验变化字段。成功后记录服务 ID、名称、地址和单价，并将实际返回的 `serviceId` 作为 `productionServiceId` 交给后续生产配置。保存成功响应不包含服务状态；只有后续服务列表只读查询成功并按 `serviceId` 匹配到该服务时才记录实际状态，查询失败或尚未返回时保持“状态未取得”，不得推断状态或重复执行 `save`。`full_process` 只记录正式产物和生产配置替换待办，不返回 integration 或重复沙箱联调；`onboarding_only` 未来进入 integration 时仍按上方产物关系使用固定沙箱值，正式上线前再用 `productionServiceId` 替换。

**固定命令契约**：复用已有服务不调用 `save`；创建不传 `--service-id`，修改必须传入用户从 Step 3.1 候选中选定的实际 `serviceId` 和全部字段：

```bash
bash modules/scripts/service.sh validate --name "<name>" --desc "<desc>" --url "<url>" --pricing "<pricing>" --schema "<json>"
bash modules/scripts/service.sh save --name "<name>" --desc "<desc>" --url "<url>" --pricing "<pricing>" --schema "<json>"
bash modules/scripts/service.sh save --service-id "<serviceId>" --name "<name>" --desc "<desc>" --url "<url>" --pricing "<pricing>" --schema "<json>"
```

#### 5.3 应用发布

**应用分支**：按量付费/网站支付使用 `WEBAPP`，APP支付使用 `MOBILEAPP`。复用 Step 3.1 候选和 Step 4 条件资料，按 `modules/app-release.md` 执行复用或新建、设置/校验应用公钥和提审；产品上下文、平台字段、`appSign` 边界、MCP 参数和返回处理只在该模块维护。

**固定命令契约**：以下命令只允许替换实际值，禁止把位置参数改写为 `--app-id`、`--public-key` 等未定义选项。`reuse/key/verify-key/audit` 使用的 `appId` 必须逐字复用 Step 3.1 中用户选定的候选 ID，或本轮 `create` 输出的 `APP_ID`；禁止从其他候选、历史轮次或示例中替换。

```bash
# 按量付费/网站支付新建 WEBAPP
bash modules/scripts/app.sh create --product-type "<aipay|webpay>" --sales-code "<当前产品码>"
# APP支付新建 MOBILEAPP：按实际平台三选一
bash modules/scripts/app.sh create --product-type apppay --sales-code "I1080300001000041313" --mobile-platform IOS --bundle-id "<bundleId>"
bash modules/scripts/app.sh create --product-type apppay --sales-code "I1080300001000041313" --mobile-platform ANDROID --app-package "<appPackage>" --app-sign "<appSign>"
bash modules/scripts/app.sh create --product-type apppay --sales-code "I1080300001000041313" --mobile-platform ALL --bundle-id "<bundleId>" --app-package "<appPackage>" --app-sign "<appSign>"
# 下列四个子命令的 appId/publicKey 均为位置参数
bash modules/scripts/app.sh reuse "<appId>"
bash modules/scripts/app.sh key "<appId>" "<用户明确提供的完整publicKey>"
bash modules/scripts/app.sh verify-key "<appId>" "<同一publicKey>"
bash modules/scripts/app.sh audit "<appId>"
```

**复用应用**：执行 `bash modules/scripts/app.sh reuse "<appId>"`，其中 `<appId>` 必须是用户从本轮候选中选定的同一 ID。复用已有 `MOBILEAPP` 时不需要采集 `mobilePlatform`、`bundleId`、`appPackage` 或 `appSign`；这些字段仅在新建 `MOBILEAPP` 时需要。

仅 `CREATE` 分支在最终创建摘要前重查应用列表；结果未变时不增加用户确认，结果变化时更新候选和决策，重查失败时禁止创建但不回滚其他分支。非 `ON_LINE` 应用不可复用；`FLOW:PENDING_APPLICATIONS` 必须由用户选择暂不新建或新建，禁止当作空列表。

用户完成公钥页面确认后自动校验，不再询问是否校验；应用公钥校验失败或 RSA2 项未返回支付宝公钥时禁止提审。已取得支付宝公钥但未能写入本地文件时，不阻塞已确认的提审，但必须记录 `MANUAL_CONFIGURATION_REQUIRED`；只有实际导出成功时才记录公钥文件路径。最终保留 `appId`、应用状态和公钥导出状态供集成使用。

复用应用返回 `FLOW:REUSE_NO_KEY` 时，必须将应用分支标记为未完成，不得输出复用成功。一次向用户说明当前 `appId` 缺少已确认的 RSA2 应用公钥，并让用户选择为当前应用设钥或返回 Step 4 重新决策。用户选择设钥时，按 `modules/app-release.md` 的标准引导、外部写操作确认和公钥红线执行；确认生效后重新执行 `bash modules/scripts/app.sh reuse "<同一appId>"`，不得跳过应用状态与支付宝公钥检查，也不得改变参数形式或替换 ID。

### Step 6: 本轮流程收口

`integration_only` 不应进入本步骤。`onboarding_only` 和 `full_process` 按下方输出模板中的“后续衔接”分支处理；`full_process` 只保留两个子流程的实际状态，不转换成另一套状态。

**输出格式**：
```markdown
支付宝商家入驻本轮流程已执行至当前可推进终点。

只输出本轮实际取得且有判断依据的字段。没有取得的费率、`serviceId`、`appId`、应用状态或公钥导出状态不得使用占位值、空值或推断值填充；改在“待办”中说明缺失原因和恢复动作。

| 模块 | 项目 | 信息 |
|------|------|------|
| 📦 产品信息 | 产品类型 | {productName} |
| 📦 产品信息 | 经营类目 | {mccName} |
| 📋 签约信息 | 签约状态 | {实际取得时输出} |
| 📋 签约信息 | 费率 | {实际取得费率时才输出} |
| 🧩 服务信息 | 服务ID | {按量付费且实际取得 serviceId 时输出} |
| 🧩 服务信息 | 服务状态 | {按量付费且实际取得状态时输出} |
| 📱 应用信息 | 应用ID | {实际取得 appId 时输出} |
| 📱 应用信息 | 应用状态 | {实际取得状态时输出} |
| 🔑 配置信息 | 支付宝公钥导出状态 | {实际取得导出状态时输出} |

{仅在实际展示费率时紧随输出：由于支付宝可能会有阶段性的优惠活动，实际费率可能低于该页面费率，具体以相应费用账单为准。}

**待办**
- {只列出未取得、待生效、待审核、待手动配置或需要用户继续处理的项目；没有则省略本节。}

**后续衔接**
- 如果本次只完成签约/入驻：还需要完成代码集成，才能在业务系统中实际发起支付。
- 如果本次是完整接入/一站式流程：进入 onboarding 前已经通过主编排确认集成存在或完成；本步骤结束后直接输出完整接入最终汇总，不重复进入集成流程。只有用户明确更正此前的集成状态，或 Agent 重新检查项目代码并取得支付能力未完成的直接证据时，才暂停签约写操作并转入 integration，完成后返回 onboarding 的中断点。签约状态、应用列表、服务列表及其查询失败均不得作为回跳依据。
```

使用 TaskUpdate 只把已经实际执行并取得结果的内部任务标记为 completed。签约待生效、应用待审核、人工配置或其他外部待办必须保持 pending 或在用户可见待办中明确列出，不得为了结束本轮对话把这些事项标记为 completed。对于 `full_process`，此处只表示 onboarding 子流程已执行至当前可推进终点；最终是否完成直接依据两个子流程的实际结果和待办，不得另建状态推断。

---

## 四、错误处理（全局规则）

签约流程的任何 CLI 命令（包括 MCP、login、whoami、logout 和 file upload）执行后都必须立即使用脚本内置错误检测；失败时禁止继续解析业务结果。认证、联网、授权不匹配和业务错误的完整识别与恢复规则以 `modules/scripts/error_handler.sh` 和 `modules/error-handling.md` 为唯一详细来源；依据不足按本文件“核心铁律 10”处理；Step 3.1 的多查询失败隔离仍按该步骤规则执行。

---

## 五、流程状态

只在当前任务中保留恢复流程所需的最小上下文，不建立第三套状态机：

| 类别 | 必要字段 |
|------|----------|
| 意图与方案 | `intention`、`productName`、`productType`、`salesCode`、`scope`、`mccCode`、`mccName` |
| 签约 | 查询成功后记录 `signStatus`；仅状态可确定后记录 `materialStatus`、`missingMaterials`、三个图片引用，APP支付未签约时另保留 `appName` |
| 服务 | 决策、候选 ID/名称/状态、`serviceId`、五项服务资料、校验状态和实际结果 |
| 应用 | 决策、候选 ID/名称/状态、`appId`、应用类型、新建 MOBILEAPP 所需平台字段、`publicKeyPrepared`、校验状态、应用状态和支付宝公钥导出状态/可选路径 |

只记录用户明确提供或更正的资料，复用未变化且已校验通过的字段；用户更正字段时只清除受影响的决策或校验结果，只补问缺失或校验失败字段。

`materialStatus` 只描述签约材料，不能代替服务或应用的 `validationStatus`。签约查询成功后，按量付费或已签约/已提交签约时设为 `NOT_REQUIRED`；网站支付/APP支付未签约时按实际结果设为 `MISSING` / `DECLARED_READY` / `VERIFIED`。签约查询未成功时，`signStatus`、`materialStatus` 和 `missingMaterials` 均保持未设置；`onboarding_only` 不在 Step 3.1 前推定材料状态，`full_process` 也只继承实际已确认的值和产物。

`full_process` 继承 `VERIFIED` 材料时，进入 Step 5 前仍必须检查 `collect_information.screenshot` 恰好包含三个非空图片引用，APP支付还必须检查 `collect_information.appName` 非空。通过后跳过重复上传；失败时将 `materialStatus` 改为 `MISSING`、记录对应 `missingMaterials` 并回到 Step 4 补齐，禁止仅凭 `VERIFIED` 提交签约。

禁止保存私钥、公钥原文、签名串、授权 token、完整支付表单或其他业务凭据。`appSign` 只是 Android 应用签名摘要，仅在创建 MOBILEAPP 时使用；不得把应用公钥工具引导套用到 `appSign`。
