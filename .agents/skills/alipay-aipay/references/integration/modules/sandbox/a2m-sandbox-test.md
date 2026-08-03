# 按量付费沙箱测试

> ⚠️ **仅当产品为「按量付费」时执行本文档定义的联调**。网站支付不执行本文档，但仍必须按 `sandbox-setup-guide.md` 第 5.4 节完成网站支付沙箱付款提示；APP支付按集成主流程跳过步骤 5。

本文档定义按量付费 402 协议的沙箱端到端服务端联调流程。测试脚本位于 `../scripts/local_402_sandbox_pay.py`。

> 本文档位于 `references/integration/modules/sandbox/`，文中的 `../scripts/...` 路径相对本目录；从技能包根目录执行时对应 `references/integration/modules/scripts/...`。

---

## 前提条件

在执行此步骤前，必须满足：

1. **已完成步骤 4 代码生成**：按量付费集成代码已生成并整合进用户项目
2. **用户本地服务已启动**：服务端已运行在可访问的地址上（如 `http://localhost:5000/demo/a2m/resource`）
3. **已知沙箱买家 2088 账号**：从步骤 2 沙箱信息输出的买家账号中获取 `userId`（如 `2088722100508485`）
4. **Python 3 可用**：测试脚本依赖 Python 3 运行
5. **已使用沙箱服务 ID**：用户服务的沙箱运行配置中 `serviceId` 固定为 `api_mock_service_id`，不要向用户索要正式 `serviceId`

---

## 测试前预检

执行沙箱测试脚本前，先解码服务返回的 `Payment-Needed`，确认关键字段存在。`SERVICE_URL` 使用用户实际服务地址。

```bash
curl -s -D - -o /dev/null "$SERVICE_URL" \
  | python3 -c 'import sys, base64, json
headers = sys.stdin.read().splitlines()
statuses = [line.split() for line in headers if line.startswith("HTTP/")]
status = statuses[-1][1] if statuses and len(statuses[-1]) > 1 else ""
if status != "402":
    raise SystemExit(f"HTTP 状态不是 402: {status or '未知'}")
value = next((line.split(":", 1)[1].strip().strip("\"") for line in headers if line.lower().startswith("payment-needed:")), "")
if not value:
    raise SystemExit("未找到 Payment-Needed 响应头")
value += "=" * ((4 - len(value) % 4) % 4)
data = json.loads(base64.urlsafe_b64decode(value).decode())
required = [
    ("protocol", "out_trade_no"), ("protocol", "amount"), ("protocol", "currency"),
    ("protocol", "resource_id"), ("protocol", "pay_before"), ("protocol", "seller_signature"),
    ("protocol", "seller_sign_type"), ("protocol", "seller_unique_id"),
    ("method", "seller_name"), ("method", "seller_id"), ("method", "seller_app_id"),
    ("method", "goods_name"), ("method", "seller_unique_id_key"), ("method", "service_id"),
]
missing = [f"{path[0]}.{path[1]}" for path in required if not data.get(path[0], {}).get(path[1])]
if missing:
    raise SystemExit("缺少字段: " + ", ".join(missing))
if data.get("method", {}).get("service_id") != "api_mock_service_id":
    raise SystemExit("沙箱联调 method.service_id 必须为 api_mock_service_id")
print("Payment-Needed 预检通过，已确认 HTTP 402 和关键字段存在。")
print("已校验字段: " + ", ".join(f"{path[0]}.{path[1]}" for path in required))'
```

POST 服务将首行 curl 替换为 `curl -s -D - -o /dev/null -X POST -H "Content-Type: ${CONTENT_TYPE:-application/json}" -d "$BODY" "$SERVICE_URL"`。`seller_signature` 由代码示例中的 `generateSellerSignature` / `generate_seller_signature` 生成，签名入参中的 `service_id` 与 `method.service_id` 都必须使用 `api_mock_service_id`；禁止使用空值、其他占位符或要求用户提供正式 `serviceId`。预检输出只打印校验结论和字段名，不打印字段值。

---

## 测试脚本路径

```bash
python3 ../scripts/local_402_sandbox_pay.py
```

脚本提供两个子命令：
- `run`：获取 Payment-Needed 并生成沙箱付款链接
- `complete`：携带 Payment-Proof 重试原始服务

---

## 完成条件

- [M-05] 已确认用户本地按量付费服务已启动可访问
- [M-05] 已完成 Payment-Needed 预检，确认 HTTP 402 和关键字段存在
- [M-05] 已执行沙箱支付测试脚本 `run --auto-complete`，成功获取 Payment-Needed、生成付款链接并连续执行服务端联调
- [M-05] 已携带 Payment-Proof 重试原服务并验证服务响应成功
- [M-05] 已确认 402 沙箱服务端联调流程通过

---

## 测试流程

### 第一步：确定服务与测试参数

先从已确认的目标项目、已有运行上下文和已校验沙箱配置中确定：
1. 按量付费集成代码已部署到本地服务
2. 服务已启动并可通过 URL 访问
3. 服务实现了 402 协议（无 Payment-Proof 时返回 HTTP 402 + Payment-Needed 头）

**确定测试参数**：
- `SERVICE_URL`：本地服务请求地址（含 GET 参数，如 `http://localhost:5000/demo/a2m/resource?gift=demo`）
- `HTTP_METHOD`：`GET` 或 `POST`
- `BUYER_2088`：沙箱买家 2088 账号（`userId`，如 `2088722100508485`）

> ⚠️ 只有缺少无法从当前上下文取得的真实值时，才询问用户对应信息。不要编造买家账号，也不得将补齐参数变成“是否执行测试”的确认。参数和服务就绪后，向用户输出测试说明并直接进入第二步。

### 第二步：执行沙箱支付测试脚本

**⚠️ 脚本的收银接口地址和付款链接前缀已硬编码在脚本中（`PAY_ENDPOINT`、`PAY_URL_PREFIX`），执行时无需额外指定。**

先输出对客执行说明，然后立即运行脚本，不等待用户确认：

> 开始执行按量付费沙箱服务端联调。
> - 服务地址：`{SERVICE_URL}`
> - HTTP 方法：`{HTTP_METHOD}`
>
> 我将依次验证 402 Payment-Needed、沙箱收银链路、Payment-Proof 重试和资源交付，完成后输出测试结论。

GET 请求：

```bash
python3 ../scripts/local_402_sandbox_pay.py run \
  --url "$SERVICE_URL" \
  --method GET \
  --buyer-id "$BUYER_2088" \
  --auto-complete
```

POST 请求：

```bash
python3 ../scripts/local_402_sandbox_pay.py run \
  --url "$SERVICE_URL" \
  --method POST \
  --body "$BODY" \
  --buyer-id "$BUYER_2088" \
  --auto-complete
```

**脚本执行步骤**：
1. 请求 `$SERVICE_URL`，提取 `Payment-Needed` 响应头
2. Base64 解码账单 JSON
3. 将下划线字段转换为驼峰字段（适配沙箱收银接口）
4. 补充 `method.buyerUniqueIdKey = "buyerExternalId"`
5. 补充 `protocol.buyerUniqueId = <买家 2088>`
6. 补充 `signature` 字段
7. POST 到沙箱收银接口 `http://aicashier.dl.alipaydev.com/openclawpay/agent/v1/pay`
8. 生成浏览器付款链接
9. 使用同一过程产物构建 Payment-Proof 并重试原始服务
10. 输出服务端联调结果

**脚本输出**：
- Payment-Needed 获取结果和本地过程产物路径（终端不输出原始值）
- 脱敏后的账单 JSON
- 脱敏后的收银接口请求体
- 收银接口返回
- **浏览器付款链接**（格式：`https://render.alipay.com/p/yuyan/180020010001290755/pay.html?schema=...`）
- 过程产物目录路径（`$ARTIFACT_DIR`）
- Payment-Proof 重试后的服务响应与测试结论

**错误处理**：
- 沙箱化 Agent 环境执行本脚本时，首次执行即申请可联网权限；如果已因网络受限失败，复用同一条命令申请可联网权限重试
- 如果未找到 `Payment-Needed` 响应头，确认服务端是否实现了 402 协议
- 如果收银接口返回 `PARAM_INVALID: protocol.sellerSignature不能为空`，回到预检步骤检查 `protocol.seller_signature` 是否为空、缺失或仍是占位符
- 如果 `pay_before` 相关参数报错，检查是否使用 ISO 8601 带时区格式
- 如果收银接口返回 `PAY_SUBMIT_FAILED`、`系统繁忙` 或缺少 `payScheme`，不要重新请求用户服务生成新订单。脚本默认会自动重试；仍失败时，复用过程产物重试收银接口：
  ```bash
  python3 ../scripts/local_402_sandbox_pay.py run \
    --reuse-artifact "$ARTIFACT_DIR" \
    --auto-complete
  ```
  `--reuse-artifact` 会复用上一轮产物中的 `Payment-Needed`、服务 URL、HTTP 方法、POST 请求体和 Content-Type。脚本不会重新请求用户服务生成新订单。这样可以避免本地内存订单堆积，导致后续 `ORDER_NOT_FOUND` 或多订单兜底匹配失败。
- `run` 命令支持 `--pay-retries` 和 `--pay-retry-delay`，用于调大沙箱收银接口临时失败时的重试次数与间隔。

### 第三步：连续验证 Payment-Proof 重试

标准流程使用第二步命令中的 `--auto-complete`。脚本生成付款链接后，使用同一过程产物继续执行 `complete` 逻辑并输出服务端联调结果。

如果标准命令在生成付款链接后因进程中断而未完成最终重试，使用同一过程产物恢复：

```bash
python3 ../scripts/local_402_sandbox_pay.py complete \
  --artifact-dir "$ARTIFACT_DIR"
```

**脚本执行步骤**：
1. 读取过程产物目录中的 `state.json`
2. 使用前面沙箱收银接口下单返回的 `trade_no` 构建 `Payment-Proof` 请求头（含 `payment_proof`、`trade_no`、`client_session`）
3. 携带 `Payment-Proof` 请求头重试原始服务地址
4. 打印最终服务响应

**Payment-Proof 处理约束**：
- `trade_no` 必须使用前面沙箱收银接口返回并写入 `state.json` 的交易号；不得改用商户侧 `out_trade_no` 或重新生成。
- `payment_proof`、`client_session` 和最终请求头由脚本按沙箱联调约定自动处理；Agent 不得手工拼接、改写或向用户展开其原始值和内部处理规则。
- 进程中断时只复用同一个 `$ARTIFACT_DIR` 恢复，禁止脱离脚本另行构造凭证。

Payment-Proof 参考结构：

```json
{
  "protocol": {
    "payment_proof": "bc0535fc673722fee9d37306239859fb30bddae8fdc971df2069e37401380452",
    "trade_no": "20260702008281225779190000002293"
  },
  "method": {
    "client_session": "eyJleHRlcm5hbElkIjoiMjA4ODcyMjEwMjczMjcwNyIsInNpZ25hdHVyZSI6Ii0iLCJ0aW1lc3RhbXAiOiIxNzgyODg0OTkyODc5In0="
  }
}
```

**验证要点**：
- 服务返回 HTTP 200（而非 402）
- 响应体包含服务资源内容（如 `resource_id`、`content` 等）
- 如果服务实现了 `Payment-Validation` 响应头，检查其内容

**常见失败与排查**：
- `ORDER_NOT_FOUND`：`alipay.aipay.agent.payment.verify` 成功只代表支付宝凭证有效，不代表商户本地订单已匹配成功。检查服务端是否在返回 `Payment-Needed` 前持久化了 `out_trade_no`、`resource_id`、金额和订单状态；验证成功后应优先用验付返回的 `outTradeNo` 查询订单。沙箱响应字段可能为空字符串，排查时先判断字段是否非空；如果本地服务重启导致内存订单丢失，已付款旧链接不能继续用于重启后的服务。
- `RESOURCE_ID_MISMATCH`：检查验付返回的 `resourceId` 是否非空且与本地订单保存的资源一致。沙箱字段为空时不要把空字符串当作有效资源字段参与强校验；生产环境必须把资源字段缺失作为异常处理。
- `FULFILLMENT_CONFIRM_FAILED` 或 HTTP 5xx：资源生成后调用 `alipay.aipay.agent.fulfillment.confirm` 失败时，不要返回成功交付；应保留服务结果并允许同一笔 `Payment-Proof` 重试履约确认。

### 第四步：测试结论与可选付款体验

测试通过后输出：

```
✅ 按量付费沙箱服务端联调通过

| 测试项 | 状态 |
|--------|------|
| 402 Payment-Needed 返回 | ✅ |
| 沙箱收银链路 | ✅ |
| Payment-Proof 重试 | ✅ |
| 资源交付 | ✅ |
```

输出测试结论后，将脚本生成的付款链接作为可选体验入口提供给用户，并从已校验的本地沙箱配置读取工具实际返回的沙箱买家账号和密码。字段缺失时省略对应字段，禁止编造：

> 如需体验沙箱付款页面，可在浏览器打开以下链接，并使用沙箱买家账号操作：
> - 买家账号：`{email}`（如有）
> - 登录密码：`{logonPassword}`（如有）
> - 支付密码：`{payPassword}`（如有）
>
> {付款链接}

同时提供安卓客户端体验说明：

> 沙箱支付宝当前仅支持安卓系统。如需安装沙箱版支付宝，请复制以下链接到网页浏览器打开，并使用支付宝客户端扫码下载：
>
> https://mdn.alipayobjects.com/sandboxsys/afts/img/_itHRrdOD9oAAAAAAAAAAAAADgSLAQBr/original

最后必须提醒用户：

> 当前 `serviceId=api_mock_service_id` 仅用于沙箱联调。正式上线前，必须替换为支付宝服务市场注册或复用服务后实际返回的真实 `serviceId`。

**沙箱支付宝下载红线（Agent 内部约束，不要作为用户提示逐字输出）：**
- 默认只提供上方沙箱支付宝安装直链，复制到网页浏览器打开，并使用支付宝客户端扫码下载。
- 严禁声称 iOS 也有沙箱版支付宝或可在 iOS 完成沙箱支付宝安装。
- 严禁引导用户去应用市场、应用商店、搜索引擎或任何第三方站点搜索/下载“沙箱支付宝”。
- 严禁提供 `https://sandbox.alipay.com`、其他自编 URL、二维码、安装包名称或下载方式。
- 如果用户明确反馈上述直链、页面二维码无法打开或无法下载，只能引导用户前往支付宝开放平台沙箱工具页 `https://open.alipay.com/develop/sandbox/tool`，按页面指引安装沙箱版支付宝；不得自行补充其他替代渠道。

---

## 已有 Payment-Needed 值（可选）

如果用户已经提供了 `Payment-Needed` 响应头值（如之前已从其他渠道获取），可以通过 `--payment-needed` 参数直接传入，跳过对用户服务的首次请求：

```bash
python3 ../scripts/local_402_sandbox_pay.py run \
  --url "$SERVICE_URL" \
  --method GET \
  --buyer-id "$BUYER_2088" \
  --payment-needed "$PAYMENT_NEEDED_VALUE" \
  --auto-complete
```

脚本将直接解码该值并继续后续流程，不再发起 curl 请求。

---

## 注意事项

1. **沙箱收银接口**：脚本默认使用 `http://aicashier.dl.alipaydev.com/openclawpay/agent/v1/pay`，此地址已硬编码在脚本中
2. **付款链接**：前缀为 `https://render.alipay.com/p/yuyan/180020010001290755/pay.html?schema=`，通过 URL 编码的 `payScheme` 拼接
3. **过程产物**：默认写入 `/tmp/alipay_local_402_sandbox_pay_<timestamp>`，可通过 `--artifact-dir` 指定
4. **买家签名**：默认占位值 `-`，无需修改
5. **沙箱化 Agent 网络权限**：脚本内 `fetch_payment_needed` 使用 curl，且收银测试会访问沙箱接口；沙箱化 Agent 环境首次执行即申请可联网权限
6. **沙箱支付宝安装包**：沙箱支付宝当前仅支持安卓系统；默认引导用户复制 `https://mdn.alipayobjects.com/sandboxsys/afts/img/_itHRrdOD9oAAAAAAAAAAAAADgSLAQBr/original` 到网页浏览器打开，并使用支付宝客户端扫码下载。若用户明确反馈该直链、页面二维码无法打开或无法下载，只能引导用户前往支付宝开放平台沙箱工具页 `https://open.alipay.com/develop/sandbox/tool`，按页面指引安装沙箱版支付宝。严禁引导用户使用 iOS 沙箱支付宝、应用市场搜索、`https://sandbox.alipay.com` 或其他自编下载 URL。

---

## 脚本内部流程概要

```
用户服务 (无 Payment-Proof)
    ↓ HTTP 402 + Payment-Needed (base64 JSON)
脚本提取并解码
    ↓ snake_case → camelCase + 补充买家ID
沙箱收银接口
    ↓ payScheme
浏览器付款链接（保留为可选体验入口）
    ↓
脚本连续构建 Payment-Proof
    ↓ base64(protocol + method)
重试用户服务
    ↓ HTTP 200 + 资源内容
验证通过 ✅
```
