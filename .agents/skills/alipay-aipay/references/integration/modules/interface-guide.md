# 接口说明与代码示例索引

本文件汇总了支付宝支付产品的通用接口和产品专用接口的参考文档及代码示例索引，供主 Skill 集成参考。**本 Skill 仅支持按量付费、网站支付、APP支付三种产品**。

---

## 一、通用接口

下表所列接口适用于网站支付和APP支付产品，请结合用户集成诉求按需查阅。**按量付费产品不使用这些通用接口**。

### 通用接口文档索引

| 接口名称 | 文档链接 |
| --- | --- |
| 统一收单交易查询接口 | <https://ideservice.alipay.com/cms/site/0izblf> |
| 统一收单交易退款接口 | <https://ideservice.alipay.com/cms/site/0izam4> |
| 统一收单交易退款查询接口 | <https://ideservice.alipay.com/cms/site/0izl48> |
| 统一收单交易关闭接口 | <https://ideservice.alipay.com/cms/site/0izcu8> |
| 异步通知说明 | <https://ideservice.alipay.com/cms/site/0izal6> |

### 通用接口代码示例索引

各接口的代码示例按编程语言拆分，存放于各语言目录的 `1-通用接口/` 子目录下。请根据用户实际使用的编程语言查找所需接口的示例代码，**不要混用不同语言**。

> **⚠️ 按量付费产品不涉及这些通用接口**，按量付费使用专属的 A2M 接口进行支付。

表格中某语言列为空，表示该语言暂无对应接口的示例文档。此时请依据上述通用接口文档中的**公共请求参数**、**业务请求参数**及**响应参数**，获取字段名、类型、是否必填及取值规则，按照当前编程语言规范自行完成实现。

| 接口名称 | Java | Python | Node.js | PHP | C# |
| --- | --- | --- | --- | --- | --- |
| 统一收单交易查询接口 | [示例](./code-examples/java/1-通用接口/统一收单交易查询接口代码示例.md) | [示例](./code-examples/python/1-通用接口/统一收单交易查询接口代码示例.md) | [示例](./code-examples/nodejs/1-通用接口/统一收单交易查询接口代码示例.md) | [示例](./code-examples/php/1-通用接口/统一收单交易查询接口代码示例.md) | [示例](./code-examples/csharp/1-通用接口/统一收单交易查询接口代码示例.md) |
| 统一收单交易退款接口 | [示例](./code-examples/java/1-通用接口/统一收单交易退款接口代码示例.md) | [示例](./code-examples/python/1-通用接口/统一收单交易退款接口代码示例.md) | [示例](./code-examples/nodejs/1-通用接口/统一收单交易退款接口代码示例.md) | [示例](./code-examples/php/1-通用接口/统一收单交易退款接口代码示例.md) | [示例](./code-examples/csharp/1-通用接口/统一收单交易退款接口代码示例.md) |
| 统一收单交易退款查询接口 | [示例](./code-examples/java/1-通用接口/统一收单交易退款查询接口代码示例.md) | [示例](./code-examples/python/1-通用接口/统一收单交易退款查询接口代码示例.md) | [示例](./code-examples/nodejs/1-通用接口/统一收单交易退款查询接口代码示例.md) | [示例](./code-examples/php/1-通用接口/统一收单交易退款查询接口代码示例.md) | [示例](./code-examples/csharp/1-通用接口/统一收单交易退款查询接口代码示例.md) |
| 统一收单交易关闭接口 | [示例](./code-examples/java/1-通用接口/统一收单交易关闭接口代码示例.md) | [示例](./code-examples/python/1-通用接口/统一收单交易关闭接口代码示例.md) | [示例](./code-examples/nodejs/1-通用接口/统一收单交易关闭接口代码示例.md) | [示例](./code-examples/php/1-通用接口/统一收单交易关闭接口代码示例.md) | [示例](./code-examples/csharp/1-通用接口/统一收单交易关闭接口代码示例.md) |

---

## 二、产品专用接口

本 Skill 支持的三种产品的专用接口说明。代码示例存放于各语言目录的对应产品子目录下。

### 按量付费 API

> ⚠️ 按量付费使用专属的 A2M（Agent to Machine Payment）接口，不使用统一收单接口。

| 接口名称 | 文档链接 | 说明 |
| --- | --- | --- |
| 支付凭证验证 `alipay.aipay.agent.payment.verify` | [文档](https://ideservice.alipay.com/cms/site/0jaqax) | 验证 AI 智能体支付凭证 |
| 商家履约回执确认 `alipay.aipay.agent.fulfillment.confirm` | [文档](https://ideservice.alipay.com/cms/site/0jaqax) | 确认商家已完成服务交付 |

> **说明**：以上两个接口的在线文档同属按量付费聚合文档（0jaqax），并非两个独立文档，查阅时请在同一文档内定位对应接口。

**按量付费代码示例**（A2M 接口，位于各语言 `4-按量付费/` 子目录）：

| 语言 | 代码示例 |
| --- | --- |
| Java | [示例](./code-examples/java/4-按量付费/A2MPaymentDemoController.java) |
| Python | [示例](./code-examples/python/4-按量付费/A2MPaymentDemo.py) |
| Node.js | [示例](./code-examples/nodejs/4-按量付费/A2MPaymentDemo.js) |
| PHP | [示例](./code-examples/php/4-按量付费/A2MPaymentDemo.php) |
| C# | [示例](./code-examples/csharp/4-按量付费/A2MPaymentDemo.cs) |

### 网站支付 API

| 接口名称 | 文档链接 | Java | Python | Node.js | PHP | C# |
| --- | --- | --- | --- | --- | --- | --- |
| 统一收单下单并支付页面接口 | [文档](https://ideservice.alipay.com/cms/site/0iztfv) | [示例](./code-examples/java/2-网站支付/统一收单下单并支付页面接口代码示例.md) | [示例](./code-examples/python/2-网站支付/统一收单下单并支付页面接口代码示例.md) | [示例](./code-examples/nodejs/2-网站支付/统一收单下单并支付页面接口代码示例.md) | [示例](./code-examples/php/2-网站支付/统一收单下单并支付页面接口代码示例.md) | [示例](./code-examples/csharp/2-网站支付/统一收单下单并支付页面接口代码示例.md) |

> **说明**：网站支付使用统一收单下单并支付页面接口 `alipay.trade.page.pay`，适用于电脑网页与手机网页场景。上表仅是接口调用示例，实际集成不得止于设置 `return_url`；用户未明确关闭同步回跳时，必须继续按 `alipay-sdk-reminder.md` 的“网站支付同步回跳”实现并验证结果页。本地未上线场景的异步通知口径以 `../flow.md` 为准。

### APP支付 API

| 接口名称 | 文档链接 | Java | Python | Node.js | PHP | C# |
| --- | --- | --- | --- | --- | --- | --- |
| APP支付接口 | [文档](https://ideservice.alipay.com/cms/site/0izsn4) | [示例](./code-examples/java/3-APP支付/APP支付接口代码示例.md) | [示例](./code-examples/python/3-APP支付/APP支付接口代码示例.md) | [示例](./code-examples/nodejs/3-APP支付/APP支付接口代码示例.md) | [示例](./code-examples/php/3-APP支付/APP支付接口代码示例.md) | [示例](./code-examples/csharp/3-APP支付/APP支付接口代码示例.md) |

---

## 三、代码示例使用说明

### 按量付费
- 使用 `4-按量付费/` 目录下的代码示例
- 按量付费**不需要**参考通用接口
- 五语言 A2M 示例用于说明 402、Payment-Needed、Payment-Proof、验付和履约确认的核心协议与 SDK 调用流程；关键生产控制仍可能是 TODO、注释占位、内存模拟或伪代码，因此禁止原样作为生产实现或据此宣称集成完成。
- 沙箱联调时 `serviceId` 固定使用 `api_mock_service_id`，该值是本 Skill 定义的沙箱开发值，不得改为其他占位符，也不得为继续沙箱测试向用户索要正式 `serviceId`。正式上线前必须替换为服务市场注册或复用结果中的真实 `serviceId`。
- 示例中的 `sellerId`、`appId`、`alipayPublicKey` 只能作为占位说明。正式集成时必须替换为签约入驻产物：`sellerId` 来自商户 PID/2088，`appId/alipayPublicKey` 来自应用发布或复用结果。
- 生成按量付费代码时必须补齐订单持久化、本地订单匹配、金额一致性、资源防串、幂等履约和履约确认失败可重试逻辑；不要把示例中的内存/占位实现当作生产实现。

### 网站支付 / APP支付
- 必须同时参考：
  1. 对应产品的专用接口（如 `2-网站支付/` 或 `3-APP支付/`）
  2. 通用接口 `1-通用接口/`（交易查询、退款、退款查询、关闭交易等）
- 完整集成必须覆盖产品下单、交易查询、退款、退款查询、关闭交易和异步通知处理代码。本地示例缺失时仍须依据上述已核验的官方文档实现，不得因缺少示例而省略适用接口。
- 若项目尚未上线，可先按 `../flow.md` 的“本地生产参数验收模式”完成本地结果确认链路验证；正式上线前仍必须补齐公网 HTTPS `notify_url`、验签、幂等和补偿查询。
