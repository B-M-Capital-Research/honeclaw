# 服务市场注册模块

> 本文档定义按量付费产品的服务市场上架流程。**仅按量付费产品需要服务市场注册**。
> 被引用文档：`onboarding/flow.md` → Step 3.1 服务查询 + Step 5.2 服务注册
> 本文档位于 `references/onboarding/modules/`，文中的 `scripts/...` 路径相对本目录；从技能包根目录执行时对应 `references/onboarding/modules/scripts/...`。

---

## 功能概述

将 MCP 服务上架到支付宝服务市场。**仅对"按量付费"产品需要**，网站支付和APP支付不需要服务注册。

**⚠️ 触发条件：**
```
✅ 按量付费（salesCode = I1080300001000160457）→ 需要服务注册
❌ 网站支付（salesCode = I1080300001000041203）→ 不需要
❌ APP支付（salesCode = I1080300001000041313）→ 不需要
```

---

## 服务注册完整流程（最高优先级）

**⚠️ 必须严格按照以下流程执行，不可跳过任何步骤！**

```
┌─────────────────────────────────────────────────────────────────────┐
│                     按量付费 服务注册流程                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Step 1: 查询已有服务                                                │
│    └─ 调用 discoverBazaarServicesForMcp                             │
│                                                                     │
│  Step 2: 判断服务列表                                                │
│    ├─ data.items 为空 → 进入 Step 3 收集服务信息                       │
│    └─ data.items 不为空 → 展示完整分页结果后进入 Step 3 决策           │
│                                                                     │
│  Step 3: 用户决策                                                    │
│    ├─ 选择序号复用已有服务 → 跳过创建，完成                            │
│    ├─ 选择"新建" → ⚠️ 检查服务数量，若 ≥ 20 则禁止创建                │
│    │                 若 < 20，收集 5 项服务信息，进入 Step 4           │
│    └─ 选择"修改" → 输入服务ID，收集所有服务信息，进入 Step 4          │
│                                                                     │
│  Step 4: 提交服务上架/修改                                           │
│    ├─ 创建新服务：调用 saveBazaarServiceForMcp（不传 serviceId）       │
│    │               ⚠️ 调用前必须确认服务数量 < 20                     │
│    └─ 修改已有服务：调用 saveBazaarServiceForMcp（传入 serviceId）     │
│                                                                     │
│  Step 5: 处理结果                                                    │
│    ├─ 成功 → 对客输出服务信息，继续应用发布流程                        │
│    └─ 失败 → 展示错误，引导用户修正                                   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

**⚠️ 修改服务重要说明：**
```
修改已有服务时，必须传入 serviceId + 所有服务信息（不可只传部分字段）：
✅ 正确：传入 serviceId + serviceName + serviceDesc + resourceUrl + pricing + schemaUrl
❌ 错误：只传入 serviceId + 部分字段（会导致其他字段被清空）
```

---

## Step 1: 查询已有服务

---

**执行 `scripts/service.sh list`。**

---

## Step 2: 根据查询结果执行不同分支

### 场景 A：服务列表为空（无已有服务）

**此时必须进入 Step 3 收集服务信息并创建新服务。**

> 📎 `scripts/service.sh list` 已内置该判断，返回 `FLOW:CREATE_NEW` 时直接进入 Step 3。
>
> ⚠️ 注意：服务数量上限为 20 个，`service.sh save` 在创建前会校验。

### 场景 B：服务列表不为空（有已有服务）

**此时必须按接口返回结果展示服务及其实际状态，让用户选择复用、创建或修改。Skill 不额外按状态过滤复用候选。**

**输出格式：**

```markdown
📋 发现您已有以下服务：

| 序号 | 服务ID | 服务名称 | 描述 | 价格 | 实际状态 | 服务地址 |
|------|--------|----------|------|------|----------|----------|
| 1 | SVC001 | 天气查询 | 提供全球天气查询 | 0.01元/次 | 已上线 | https://api.example.com/weather |
| 2 | SVC002 | AI助手 | 智能对话服务 | 0.05元/次 | 审核中 | https://api.example.com/ai |

请选择：
  • 输入序号（1/2）复用对应服务
  • 输入"新建"创建新服务
  • 输入"修改 + serviceId"修改已有服务
```

> 📎 表格渲染由 `scripts/service.sh list` 自动完成，无需手动拼接。Agent 直接展示脚本输出即可。

**用户选择处理：**

Agent 根据用户输入执行对应分支：

| 用户输入 | 后续操作 |
|----------|----------|
| 数字序号（1-$SERVICE_COUNT） | 复用对应服务 → 跳过创建，直接进入应用发布 |
| "新建" | 进入 Step 3 收集服务信息 → `bash scripts/service.sh save`（创建前自动校验数量 < 20） |
| "修改 + serviceId" | 使用表格中展示的 serviceId → 进入 Step 3 收集**所有**字段 → `bash scripts/service.sh save --service-id <id>` |

> ⚠️ 服务数量上限 20 个，由 `service.sh save` 在创建前自动校验。

用户选择修改时可使用列表中展示的 `serviceId`。展示和记录接口返回的实际状态，不自行增加状态过滤、可用性结论或审核规则。

---

## Step 3: 收集服务信息（创建或修改服务时执行）

**⚠️ 在以下情况需执行此步骤：**
```
✅ 服务列表为空（无已有服务）→ 创建新服务
✅ 用户选择"新建" → 创建新服务
✅ 用户选择"修改" → 修改已有服务（提供表格中的 serviceId）
```

**⚠️ 修改服务特别注意：**
```
修改已有服务时，必须收集所有服务信息，不可只收集部分字段：
✅ 正确：收集 serviceName + serviceDesc + resourceUrl + pricing + schemaUrl 全部字段
❌ 错误：只收集需要修改的字段（会导致其他字段被清空）
```

**收集服务信息交互：**

```
📋 请提供服务注册信息：

1. 服务名称（1-50 字符）：您的服务名称
2. 服务描述（1-500 字符）：简要描述服务功能
3. 服务地址（URL）：服务的 API 地址
4. 服务单价（元，最低 0.01）：用户每次调用的费用
5. 请求示例（JSON）：API 请求参数示例，提交时作为 `schemaUrl` 字符串传入

请在一条回复中一次性提供以上全部信息。修改服务时请同时提供表格中的 serviceId，不再单独增加一轮询问。
```

只补问缺失或校验失败的字段。已经通过校验的字段必须保留，不要求用户重新提交；修改服务仍必须最终获得全部五项字段，禁止只提交局部字段。

Agent 在当前任务中记录操作选择、serviceId 和五项服务资料。用户更正字段时只覆盖并重新校验该字段，其他未变化且已校验通过的字段继续复用；操作类型变化时清除不再适用的 serviceId 或创建资料。只补问缺失或校验失败字段，本模块的 `scripts/service.sh validate` 继续作为业务格式校验唯一来源。

### 入参校验规则

| 字段 | 说明 | 验证规则 | 错误提示 |
|------|------|----------|----------|
| `serviceName` | 服务名称 | 长度 1-50 字符 | 服务名称长度需在 1-50 字符之间 |
| `serviceDesc` | 服务描述 | 长度 1-500 字符 | 服务描述长度需在 1-500 字符之间 |
| `resourceUrl` | 服务地址 | 有效的 URL 格式（以 http:// 或 https:// 开头） | 请提供有效的 URL 地址 |
| `pricing` | 服务单价 | 必须 >= 0.01 元 | 服务单价最低为 0.01 元 |
| `schemaUrl` | 请求示例 | 有效的 JSON 格式；提交时序列化为字符串 | 请提供有效的 JSON 格式请求示例 |

> `schemaUrl` 是 MCP 入参字段名，本流程中承载的是序列化后的 JSON 请求示例，不是 URL。用户提供原始 JSON，例如 `{}`；`scripts/service.sh` 校验 JSON 后将其作为字符串写入 `request.schemaUrl`。禁止因为字段名含 `Url` 而要求用户提供网页地址。

### 校验脚本

> 📎 已收口到脚本：`scripts/service.sh validate`

```bash
# 收集服务信息后，调用校验脚本
bash scripts/service.sh validate   --name "$SERVICE_NAME"   --desc "$SERVICE_DESC"   --url "$RESOURCE_URL"   --pricing "$PRICING"   --schema "$SCHEMA_URL"
```

---

## Step 4: 提交服务上架/修改

### 场景 A：创建新服务（不传 serviceId）

**⚠️ 调用前必须确认服务数量 < 20，否则禁止调用 saveBazaarServiceForMcp 接口！** 执行 `bash scripts/service.sh save --name <name> --desc <desc> --url <url> --pricing <pricing> --schema <json>`。


### 场景 B：修改已有服务（必须传入 serviceId + 所有字段）

执行 `bash scripts/service.sh save --service-id <id> --name <name> --desc <desc> --url <url> --pricing <pricing> --schema <json>`。


---

## Step 5: 处理结果

### 成功输出格式

```markdown
✅ 服务创建成功

| 项目 | 信息 |
|------|------|
| 服务ID | API_xxx |
| 服务名称 | 天气查询 |
| 服务描述 | 提供全球天气查询服务 |
| 服务地址 | https://api.example.com/weather |
| 服务单价 | 0.01 元/次 |
```

`saveBazaarServiceForMcp` 的当前成功响应不包含服务状态。不得根据创建或修改成功推断为“审核中”“已上线”或其他状态。只有后续 `scripts/service.sh list` 只读查询成功并按 `serviceId` 匹配到该服务时，才能记录和展示实际状态；查询失败或尚未返回该服务时，将状态保留为“未取得”，不得因此重复执行 `save`。

### 按量付费集成产物

服务创建、修改或复用成功后，必须记录并向用户说明以下已取得字段：

| 字段 | 用途 |
|------|------|
| `serviceId` | 按量付费 `Payment-Needed.method.service_id`，也是后续 402 收银联调识别服务的关键字段 |
| 服务名称 | 用于核对收银台展示和用户识别 |
| 服务地址 `resourceUrl` | 应与实际提供 402 服务的资源地址一致 |
| 服务单价 `pricing` | 应与 `Payment-Needed.protocol.amount` 的业务定价保持一致 |
| 服务状态 | 仅在只读查询实际取得时记录，用于正式上线判断和后续排查 |

如果服务注册成功但未解析到 `serviceId`，不要继续宣称按量付费入驻完成；应提示用户重新查询服务列表或根据服务名称定位服务。

### 失败处理

> 📎 统一错误检测与处理见 `error-handling.md`。`scripts/service.sh save` 已内置错误检测。

---

## MCP调用规范

**MCP 服务名：`a2a-pay-service`**

| 方法 | 用途 | 调用时机 |
|------|------|----------|
| `a2a-pay-service.discoverBazaarServicesForMcp` | 查询服务列表 / 根据 serviceId 查详情 | **进入服务注册模块时首先调用**（支持两种查询方式，见下表） |
| `a2a-pay-service.saveBazaarServiceForMcp` | 创建新服务 / 修改已有服务 | 用户选择新建/修改时调用（修改须传 serviceId + 所有资料） |

**⚠️ discoverBazaarServicesForMcp 支持两种查询方式：**

| 查询方式 | 参数 | 说明 |
|----------|------|------|
| 查询服务列表 | `{"request":{"limit":20,"offset":0}}` | 返回接口可见的服务列表；脚本按 `pagination.total` 自动翻页并完整展示实际状态，不按状态过滤候选 |
| 根据 serviceId 查询详情 | `{"request":{"keyword":"API_xxx"}}` | 返回指定服务的详细信息 |

**查询返回处理：**

- MCP 信封先通过共享 `unwrap_mcp` 解包 `content[0].text`。
- 当前成功协议以 `code="10000"` 判断，服务数组读取 `data.items`，分页读取 `data.pagination.limit/offset/total`。
- 当前服务状态字段读取 `serviceStatus`，兼容旧响应中的 `status`；`ACTIVE` 表示已上线，其他未定义映射的状态值原样展示，不推断业务含义。
- 脚本兼容旧成功协议中的 `success=true` 与 `resultObj.serviceList`，但不得用旧结构覆盖或猜测当前字段。
- 只有明确取得成功响应中的空数组，才能判定没有已有服务；业务失败、字段缺失、分页不完整或候选缺少 `serviceId` 均必须阻断，禁止输出 `FLOW:CREATE_NEW` 或 `FLOW:SELECT`。

**⚠️ saveBazaarServiceForMcp 创建/修改服务参数说明：**

| 场景 | 必传参数 | 说明 |
|------|----------|------|
| 创建新服务 | serviceName, serviceDesc, resourceUrl, pricing, schemaUrl | 无需传入 serviceId |
| 修改已有服务 | **serviceId**, serviceName, serviceDesc, resourceUrl, pricing, schemaUrl | **必须传入 serviceId + 所有资料信息** |

**写入返回处理：**

- MCP 信封先通过共享 `unwrap_mcp` 解包 `content[0].text`。
- 当前成功协议必须同时满足 `code="10000"` 和 `data.success=true`，服务 ID 读取 `data.serviceId`。
- 脚本兼容旧成功协议中的 `success=true` 和 `resultObj.serviceId`；不得混用两套协议中的成功条件与服务 ID 路径。
- 创建成功但未解析到 `serviceId` 时必须阻断后续流程；修改成功未返回新 ID 时，继续使用本次请求中已经确认的原 `serviceId`。
- 业务失败优先展示 `data.subMsg` / `data.msg`，再读取顶层错误字段；响应结构不符合当前或兼容协议时不得宣称成功。
- 当前保存成功响应不包含服务状态；不得从 `success=true` 推断审核或上线状态。

其中 `schemaUrl` 按本模块约定传入序列化后的 JSON 请求示例。例如空请求示例的实际入参为：

```json
{
  "request": {
    "serviceName": "服务名称",
    "serviceDesc": "服务描述",
    "resourceUrl": "https://your-domain.com/callback",
    "pricing": "0.6",
    "schemaUrl": "{}"
  }
}
```

### 正确调用示例

> 📎 所有 MCP 调用均有对应脚本：

```bash
# ✅ 查询服务列表 → scripts/service.sh list
# ✅ 创建新服务 → scripts/service.sh save --name <name> --desc <desc> --url <url> --pricing <pricing> --schema <json>
# ✅ 修改已有服务 → scripts/service.sh save --service-id <id> --name <name> --desc <desc> --url <url> --pricing <pricing> --schema <json>
```

### 禁止调用示例

```bash
# ❌ 错误：虚拟未定义的方法
export PLATFORM=${DEV_TOOL_NAME} && alipay-cli mcp call a2a-pay-service.deleteService -d '...'

# ❌ 错误：自行推断的方法名
export PLATFORM=${DEV_TOOL_NAME} && alipay-cli mcp call a2a-pay-service.updateService -d '...'

# ❌ 错误：省略 server 名称
export PLATFORM=${DEV_TOOL_NAME} && alipay-cli mcp call discoverBazaarServicesForMcp -d '...'

# ❌ 错误：未先查询已有服务直接创建
export PLATFORM=${DEV_TOOL_NAME} && alipay-cli mcp call a2a-pay-service.saveBazaarServiceForMcp -d '...'
```

---

## 服务状态说明

当前列表响应优先读取 `serviceStatus`，旧响应兼容读取 `status`。下表以外的实际状态值必须原样展示；没有已验证依据时不得自行翻译、归类或推断是否可用。

| 状态 | 说明 | 处理方式 |
|------|------|----------|
| `DRAFT` | 草稿 | 展示接口返回的实际状态，由用户决定后续操作 |
| `PENDING` | 审核中 | 展示接口返回的实际状态，由用户决定后续操作 |
| `ACTIVE` | 已上线 | 展示接口返回的实际状态，由用户决定后续操作 |
| `REJECTED` | 审核拒绝 | 展示接口返回的实际状态，由用户决定后续操作 |

---

## ⛔ 禁止行为（最高优先级）

```
❌ 禁止：未查询已有服务直接创建新服务
❌ 禁止：未在 Step 3.1 成功查询已有服务就要求用户决定新建/修改或提供服务资料
❌ 禁止：服务列表不为空时直接创建新服务（必须让用户选择）
❌ 禁止：用户选择复用已有服务后仍创建新服务
❌ 禁止：跳过服务信息校验直接提交
❌ 禁止：使用虚拟或自行推断的 MCP 方法
❌ 禁止：修改服务时只传入部分字段（必须传入 serviceId + 所有服务信息）
❌ 禁止：用户选择"修改"后不收集完整服务信息直接提交
❌ 禁止：服务数量 ≥ 20 时仍调用 saveBazaarServiceForMcp 创建新服务
```
