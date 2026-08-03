# 产品签约模块

> 本文档定义产品签约的完整流程、参数规范和状态判断逻辑。
> 被引用文档：`onboarding/flow.md` → Step 3.1 签约查询 + Step 5.1 签约提交
> 本文档位于 `references/onboarding/modules/`，文中的 `scripts/...` 路径相对本目录；从技能包根目录执行时对应 `references/onboarding/modules/scripts/...`。

---

## 功能概述

签约模块负责产品签约流程，包括签约状态查询和签约申请提交。

本模块采用 `ar-sign.apply` **外部直调 / 静态参数模式**：由 `scripts/ar_sign_apply.sh` 基于当前已确认的产品类型、MCC 和资料字段直接构造 `businessProperty` 后提交申请。它不走 `previewFormView` 动态 schema 收集循环，因此只覆盖本文档明确列出的字段和产品。

**使用条件：** Step 3.1 始终使用本模块的签约状态查询规则；只有查询结果为 `NOT_SIGNED` 时，才使用本模块的签约申请提交规则。`SIGNED_EFFECTIVE`、`SIGN_SUBMITTED`、查询失败或其他状态均禁止提交签约申请。

---

## MCP能力

| MCP Server | 方法 | 用途 |
|------------|------|------|
| `ar-sign` | `apply` | 提交签约申请（主流程使用） |
| `ar-query` | `queryArInfosBySalesProd` | 按前台产品码查询合约状态 |
| `ar-query` | `queryArInfosByBackProd` | 按后台产品码查询合约状态 |
| `ar-order-query` | `queryBizOrder` | 按订单号查询签约订单 |

> **注意：** 通过 `scripts/ar_sign_apply.sh` 提交签约申请。

---

## 签约状态查询

### 查询命令

执行 `scripts/query_sign_status.sh --sales-code <当前产品码> --product-type <aipay|webpay|apppay>`。

### 状态判断规则

**根据 `ar-query.queryArInfosBySalesProd` 返回结果判断签约状态：**

| 返回结果 | 状态判定 | 说明 |
|----------|----------|------|
| `resultObj.arInfoList` 为空数组 `[]` | `NOT_SIGNED` | 未签约 |
| `resultObj.arInfoList` 不为空，且存在 `arStatus` 为 `"02"` 的记录 | `SIGNED_EFFECTIVE` | 签约已生效，跳过签约材料采集，仍在 Step 4 处理应用/服务决策 |
| `resultObj.arInfoList` 不为空，且存在 `arStatus` 为 `"01"` 的记录 | `SIGN_SUBMITTED` | 已提交签约（待生效），跳过签约材料和重复签约提交，仍在 Step 4 处理应用/服务决策 |

### 判断逻辑

`scripts/query_sign_status.sh` 自动完成以下状态判断并输出状态信号（`SIGN_STATUS=...`）与分支信号（`FLOW:PC_WEB_NOT_SIGNED` 等）。`SIGNED_EFFECTIVE` 与 `SIGN_SUBMITTED` 的后续 FLOW 保持一致，均不重复提交签约，继续应用发布/服务注册。

### 产品×状态分支处理表

| 签约状态 | 产品类型 | FLOW 信号 | 后续流程 |
|----------|----------|-----------|----------|
| `NOT_SIGNED` | 网站支付 | `FLOW:PC_WEB_NOT_SIGNED` | 进入 Step 4 一次性资料与资源决策（3张网站截图 + 应用决策/条件资料）→ Step 5 签约 + 应用发布 |
| `NOT_SIGNED` | APP支付 | `FLOW:APP_NOT_SIGNED` | 进入 Step 4 一次性资料与资源决策（APP名称 + 3张APP界面截图 + 应用决策/条件资料）→ Step 5 签约 + 应用发布 |
| `NOT_SIGNED` | 按量付费 | `FLOW:AI_PAY_NOT_SIGNED` | Step 4 无需收集页面图片，但一次完成服务/应用决策和条件资料；再进入 Step 5.1 签约 |
| `SIGNED_EFFECTIVE` | 网站支付 | `FLOW:PC_WEB_SIGNED` | Step 4 完成应用决策后，跳过签约提交并执行应用发布 |
| `SIGNED_EFFECTIVE` | APP支付 | `FLOW:APP_SIGNED` | Step 4 完成应用决策后，跳过签约提交并执行应用发布 |
| `SIGNED_EFFECTIVE` | 按量付费 | `FLOW:AI_PAY_SIGNED` | Step 4 完成服务/应用决策后，跳过签约提交并执行服务注册 + 应用发布 |
| `SIGN_SUBMITTED` | 网站支付 | `FLOW:PC_WEB_SIGNED` | Step 4 完成应用决策后，跳过重复签约提交并执行应用发布 |
| `SIGN_SUBMITTED` | APP支付 | `FLOW:APP_SIGNED` | Step 4 完成应用决策后，跳过重复签约提交并执行应用发布 |
| `SIGN_SUBMITTED` | 按量付费 | `FLOW:AI_PAY_SIGNED` | Step 4 完成服务/应用决策后，跳过重复签约提交并执行服务注册 + 应用发布 |

**⚠️ 按量付费无需签约材料：**
```
✅ 网站支付未签约：需先采集 3 张网站截图，再进入 Step 5 签约
✅ APP支付未签约：需先采集 APP 名称和 3 张APP界面截图，再进入 Step 5 签约
✅ 按量付费 未签约：Step 4 无需签约页面图片，但前置完成服务/应用决策和条件资料；Step 5 仍先提交签约
✅ 按量付费在 Step 4 基于已查询的服务候选，一次收集新建/修改分支的完整五项服务资料；Step 5.2 才执行写操作
```

**按量付费签约后的必要后续动作：**
```
✅ 未签约且提交成功：继续服务市场注册 + 应用发布
✅ 已签约：跳过签约提交，继续服务市场注册 + 应用发布
❌ 禁止：只完成产品签约就宣称按量付费入驻完成
❌ 禁止：已签约状态下重复提交签约申请
```

按量付费最终接入至少需要三类正式产物：产品签约状态、服务市场的 `serviceId`、应用发布或复用得到的 `appId` 与支付宝公钥。签约模块只解决第一类产物。

---

## 签约提交

**执行 `scripts/ar_sign_apply.sh` 提交签约申请。脚本通过 `error_handler.sh` 间接初始化 `DEV_TOOL_NAME`；产品码和类目编码优先通过 `--sales-code`、`--mcc-code` 显式传入。**

```bash
# 按量付费
bash scripts/ar_sign_apply.sh --product aipay --sales-code "I1080300001000160457" --mcc-code "<mccCode>"

# 网站支付
bash scripts/ar_sign_apply.sh --product webpay --sales-code "I1080300001000041203" --mcc-code "<mccCode>" --picurl1 <imageRef1> --picurl2 <imageRef2> --picurl3 <imageRef3>

# APP支付
bash scripts/ar_sign_apply.sh --product apppay --sales-code "I1080300001000041313" --mcc-code "<mccCode>" --app-name "<APP名称>" --picurl1 <imageRef1> --picurl2 <imageRef2> --picurl3 <imageRef3>
```

### 与动态表单签约流程的边界

| 项目 | 当前 onboarding 签约 | 动态表单签约流程 |
|------|----------------------|------------------------|
| 调用模式 | 外部直调 `ar-sign.apply` | 默认先 `previewFormView`，再按动态 schema 收集并 apply |
| 字段来源 | 本文档静态定义的三类产品字段 | `previewFormView` 返回的 `artisanPage` schema |
| 覆盖产品 | 按量付费、网站支付、APP支付 | 由产品推荐和 schema 决定 |
| 风险边界 | 后端新增/调整字段时，需同步更新本文档和脚本 | schema 变化会在 preview 流程中动态体现 |

> 如果后端签约字段发生变化，或当前静态字段提交被 `checkedError` 拒绝，应以 `previewFormView` 返回 schema 为准补齐字段，禁止在本模块中凭经验新增未确认字段。

### 网站支付签约 JSON（有截图）

```json
{
  "request": {
    "bizFeatures": {},
    "bizRequestNo": "<UUID>",
    "businessProperty": {
      "mccCode": "${MCC_CODE}",
      "webAppDTO": {
        "placeType": "ONLINE_WEBAPP",
        "appType": "PC_WEB",
        "appStatus": "OFFLINE",
        "screenshot": ["<imageRef1>", "<imageRef2>", "<imageRef3>"]
      }
    },
    "channelCode": "B_SK_SH_RPC",
    "extension": {},
    "orderType": "NEW_SIGN",
    "salesProductCodes": ["I1080300001000041203"]
  },
  "ctx": {}
}
```

### APP支付签约 JSON（有 APP 名称和截图）

```json
{
  "request": {
    "bizFeatures": {},
    "bizRequestNo": "<UUID>",
    "businessProperty": {
      "mccCode": "${MCC_CODE}",
      "nativeAppDTO": {
        "placeType": "ONLINE_NATIVEYAPP",
        "name": "<APP名称>",
        "appStatus": "OFFLINE",
        "screenshot": ["<imageRef1>", "<imageRef2>", "<imageRef3>"]
      }
    },
    "channelCode": "B_SK_SH_RPC",
    "extension": {},
    "orderType": "NEW_SIGN",
    "salesProductCodes": ["I1080300001000041313"]
  },
  "ctx": {}
}
```

> APP支付本次固定按 `appStatus=OFFLINE` 提交，不传 `appDownloadUrl`。

### 按量付费签约 JSON（无截图）

```json
{
  "request": {
    "bizFeatures": {},
    "bizRequestNo": "<UUID>",
    "businessProperty": {
      "mccCode": "${MCC_CODE}"
    },
    "channelCode": "B_SK_SH_RPC",
    "extension": {},
    "orderType": "NEW_SIGN",
    "salesProductCodes": ["I1080300001000160457"]
  },
  "ctx": {}
}
```

### 关键变量

| 变量 | 来源 | 说明 |
|------|------|------|
| `bizRequestNo` | 主技能生成 | UUID，每次签约提交前通过 `python3 -c "import uuid; print(uuid.uuid4())"` 生成，**禁止省略** |
| `mccCode` | Step 2 方案规划 | 运行时变量，格式 `Axxxx_Bxxxx`，不持久化到状态文件 |
| `channelCode` | 固定值 | `"B_SK_SH_RPC"` |
| `orderType` | 固定值 | `"NEW_SIGN"` |
| `screenshot` | Step 4 一次性资料与资源决策 | **仅未签约的网站支付/APP支付需要**（3个上传后的图片引用值），按量付费无需此项 |
| `appName` | Step 4 一次性资料与资源决策 | **仅未签约的 APP支付需要**；脚本参数仍为 `--app-name`，提交到 MCP 时映射为 `nativeAppDTO.name` |
| `appStatus` | 固定值 | **仅APP支付需要**，本次固定为 `OFFLINE` |

### 按量付费接入产物映射

| 集成字段 | 来源 | 说明 |
|----------|------|------|
| `sellerId` | 商户 PID / 2088 账号 | 通常来自沙箱或正式商户账号信息；不要用示例商户号 |
| `serviceId` | Step 5.2 服务市场注册/复用结果 | 写入 `Payment-Needed.method.service_id` |
| `seller_app_id` / `appId` | Step 5.3 应用发布/复用结果 | 写入 SDK 配置和 `Payment-Needed.method.seller_app_id` |
| `alipayPublicKey` | Step 5.3 应用发布/复用导出结果 | 用于 SDK 验签配置 |

签约成功不直接产出 `serviceId` 或 `appId`。后续集成代码需要这些值时，必须从服务注册和应用发布步骤读取，禁止沿用示例值。

### ⛔ 截图字段规则

```
✅ 网站支付未签约 → 需要 3 张网站截图（首页、商品页、支付页）
✅ APP支付未签约 → 需要 APP 名称和 3 张APP界面截图（首页、商品页、支付页），签约状态固定按 OFFLINE 提交
✅ 按量付费 未签约 → 无需截图，直接签约
✅ 网站支付/APP支付缺少支付页或支付能力 → 不提交签约；先进入/回到集成流程完成实现，再回来上传图片
✅ 网站支付/APP支付页面和支付能力已存在、仅缺三张页面图片 → 保留在 onboarding Step 4 收集和上传，不进入集成流程
❌ 禁止：按量付费 签约时传入 screenshot 字段
❌ 禁止：网站支付/APP支付 签约时省略 screenshot 字段
❌ 禁止：将“支付页”自行改口为“支付成功页”或要求用户提供文档未定义的页面
❌ 禁止：APP支付签约 MCP payload 中使用 webAppDTO、appDTO、appName、appType、placeType=APP、placeType=ONLINE_WEBAPP、placeType=ONLINE_NATIVEAPP 或 appType=PC_WEB
```

### 截图字段规则

截图字段以动态表单签约流程的映射为准：`webSiteInfo.screenshot[].picUrl` / `commonAppDeviceInfo.screenshot[].picUrl` 扁平化为 `webAppDTO.screenshot[]` / `nativeAppDTO.screenshot[]` 的图片引用数组。

因此 `ar_sign_apply.sh` 使用 `--picurl1/2/3` 接收上传后的图片引用值。如果 `upload_screenshots.sh` 未能从上传结果中解析出可用于签约的图片引用值，应停止流程并让用户重新上传图片。

---

## 签约提交后非阻塞推进

> ⚠️ **签约提交成功后，无需等待签约审核通过，应立即继续推进后续步骤。**

签约申请提交后，后端会异步审核，`arStatus` 为 `"01"`（已提交待生效）。应用创建和服务注册不依赖签约审核通过，可以继续推进。为减少用户交互，应用/服务列表查询及分支资料采集可在签约提交前完成；当前流程仍先提交签约，成功后才执行服务或应用写操作。

### FLOW 信号

`ar_sign_apply.sh` 签约成功后输出 FLOW 信号，指示后续步骤：

| FLOW 信号 | 产品类型 | 后续步骤 |
|-----------|----------|----------|
| `FLOW:AI_PAY_SIGN_CONTINUE` | 按量付费 | 立即继续 5.2 服务注册 + 5.3 应用发布 |
| `FLOW:PC_WEB_SIGN_CONTINUE` | 网站支付 | 立即继续 5.3 应用发布 |
| `FLOW:APP_SIGN_CONTINUE` | APP支付 | 立即继续 5.3 应用发布 |

### 处理原则

```
✅ 签约提交成功 → 立即推进 5.2/5.3，无需等待签约审核
✅ 按量付费：5.1 签约 → 5.2 服务注册 → 5.3 应用发布（串行推进）
✅ 网站支付/APP支付：5.1 签约 → 5.3 应用发布（串行推进）
❌ 禁止：签约提交后停住，等待签约审核通过再继续
❌ 禁止：轮询签约状态直到 arStatus变为"02"再继续
```

---

## 签约输出规范

### 签约信息（含费率免责声明）

当签约信息包含费率时，**必须附加免责声明**：

```markdown
📋 签约信息：
  • 签约状态：[状态]
  • 费率：[费率值]

> 由于支付宝可能会有阶段性的优惠活动，实际费率可能低于该页面费率，具体以相应费用账单为准。
```

**费率免责声明规则：**
- ✅ 必须：在展示费率信息的同一区域，紧随费率信息之后添加免责声明
- ✅ 必须：免责声明使用引用块格式（`>` 开头）突出显示
- ❌ 禁止：单独展示费率信息而不附加免责声明
- ℹ️ 如果签约信息中不包含费率字段，则无需添加免责声明

---

## 签约错误处理

### 后端错误响应格式

```json
{
  "errorCode": "xxx",
  "errorMessage": "错误描述",
  "bizTips": "业务提示（可展示给用户）",
  "needRetry": true
}
```

| 场景 | 处理方式 |
|------|----------|
| errorCode 存在 | 展示 errorMessage，如有 bizTips 一并展示 |
| needRetry=true | 告知用户可以重试 |
| checkedError | 提取校验错误信息，引导用户修正字段 |

### 授权信息不匹配错误

> 📎 统一错误检测见 `error-handling.md` 第三节"授权信息不匹配"。

---

## 数据存储

| 文件 | 说明 |
|------|------|
| 无独立持久化 | 签约数据由 MCP 返回，主技能直接消费，不持久化到本地文件 |
