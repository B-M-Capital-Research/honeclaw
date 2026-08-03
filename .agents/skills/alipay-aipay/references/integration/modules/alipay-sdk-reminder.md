# Alipay-SDK Reminder

以下各项为接入支付宝 SDK 的相关规则，**严禁在未阅读全部内容的情况下编写任何 SDK 相关代码**。每条规则均来自真实集成踩坑，请逐条阅读并确认后再继续，编写代码遵守规则。

---

## SDK 引入方式

SDK 的引入方式取决于对应语言的包管理方式和实际导出方式，**绝不能凭猜测选择**，必须通过查阅 SDK 文档或类型定义确定。

**各语言 SDK 引入方式参考：**

| 语言 | 包管理 | 引入方式 | 备注 |
|------|--------|---------|------|
| Java | Maven | `import com.alipay.api.*;` | 通过 Maven 依赖引入，包结构固定 |
| .NET | NuGet | `using Aop.Api;` 等 | 通过 NuGet 安装 `AlipaySDKNet.Standard`，命名空间固定 |
| PHP | Composer | `require_once '../aop/AopClient.php';` 等 | 通过 GitHub 仓库或 Composer 引入，类文件路径以实际目录为准 |
| Python | pip | `from alipay.aop.api.AlipayClientConfig import AlipayClientConfig`<br>`from alipay.aop.api.DefaultAlipayClient import DefaultAlipayClient` | 通过 pip 安装 `alipay-sdk-python`，模块路径以实际版本为准 |
| Node.js | npm | 需查阅类型定义确认 | 导入方式可能随版本变化，**必须验证**，详见下方「Node.js 导入格式判断」 |

### Node.js 导入格式判断

`alipay-sdk`（Node.js）的导入方式取决于包的实际导出方式，**绝不能凭猜测选择**。

**判断流程：**

1. **查看 package.json**：确认模块类型（ESM/CJS）和入口路径
   ```bash
   cat node_modules/alipay-sdk/package.json
   ```
   重点看 `type`、`exports`、`main`、`types` 字段。

2. **查看类型定义文件**：确认导出方式
   ```bash
   cat node_modules/alipay-sdk/dist/types/index.d.ts
   ```
   注意：类型定义文件路径以 package.json 中 `types` 字段为准。

3. **选择导入语法**：
   - 命名导出（`export class AlipaySdk`）→ `import { AlipaySdk } from 'alipay-sdk'`
   - 默认导出（`export default`）→ `import AlipaySdk from 'alipay-sdk'`
   - 混合导出 → `import AlipaySdk, { AlipaySdkConfig } from 'alipay-sdk'`

**优先级**：类型定义（.d.ts）> 官方文档 > 源码

**注意：**
- package.json 的 `exports` 字段是入口路径配置，不是导出方式
- 包的导出方式可能随版本更新而改变，务必验证

---

## 页面跳转类 API 方法选择

`alipay.trade.page.pay` 是页面跳转类 API，**严禁使用 `exec()` 方法**，必须使用页面跳转方法。各语言 SDK 的方法名不同，请按下表选择：

| 语言 | 页面跳转方法 | SDK 执行方法 | 页面跳转类 API 举例 |
|------|-------------|-------------|-------------------|
| Java | `alipayClient.pageExecute(request, "POST")` | `alipayClient.execute(request)` | `alipay.trade.page.pay` |
| .NET | `alipayClient.PageExecute(request, null, "POST")` | `alipayClient.Execute(request)` | 同上 |
| PHP | `$alipayClient->pageExecute($request, "POST")` | `$alipayClient->execute($request)` | 同上 |
| Node.js | `alipaySdk.pageExec('alipay.trade.page.pay', 'POST', {...})` | `alipaySdk.exec('alipay.trade.query', {...})` | 同上 |
| Python | `alipay_client.page_execute(request)` | `alipay_client.execute(request)` | 同上 |

**此外，APP支付（`alipay.trade.app.pay`）属于客户端 SDK 调用类 API，需使用 `sdkExecute`/`sdkExec` 方法：**

| 语言 | 客户端 SDK 方法 |
|------|---------------|
| Java | `alipayClient.sdkExecute(request)` |
| .NET | `alipayClient.SdkExecute(request)` |
| PHP | `$alipayClient->sdkExecute($request)` |
| Node.js | `alipaySdk.sdkExecute('alipay.trade.app.pay', {...})` |

**Node.js 对比示例：**
```javascript
// ✅ 正确 - 页面跳转类 API 使用 pageExec()
const htmlForm = alipaySdk.pageExec('alipay.trade.page.pay', 'POST', {
  bizContent: { out_trade_no: '123', total_amount: '10.00', ... }
});
// 返回：<form action="...">...</form> HTML 字符串

// ❌ 错误 - 对页面跳转类 API 使用 exec()
const result = await alipaySdk.exec('alipay.trade.page.pay', {
  bizContent: { out_trade_no: '123', total_amount: '10.00', ... }
});
// 返回：JSON 对象，无法用于页面跳转，这是最常见的集成错误！
```

| 方法类型 | 用途 | 返回值 | 示例 |
| --- | --- | --- | --- |
| 页面跳转方法（`pageExec()`/`pageExecute()` 等） | 页面跳转类 API（如：支付） | HTML 表单字符串 | 网站支付 |
| `exec()` | 服务端 API（查询 / 退款） | 结构化数据（JSON 等） | 交易查询、退款 |

**对支付接口调用 exec() 将不会得到可用的支付表单，是最常见的集成错误之一。**

### 前端支付表单处理

页面跳转类 API 返回的是包含自动提交脚本的 HTML `<form>` 字符串，前端**必须将此 HTML 渲染到页面并自动提交表单**，禁止直接用 URL 跳转。

**❌ 错误写法（以浏览器端 JavaScript 为例）**：
```javascript
// paymentHtml 是 HTML 表单字符串，不是 URL
if (data.paymentHtml) {
  window.location.href = data.paymentHtml; // 页面只显示 ?method=alipay.trade.page.pay... 参数，无法跳转
}
```

**✅ 正确写法**：
```javascript
if (data.paymentHtml) {
  const container = document.createElement('div');
  container.innerHTML = data.paymentHtml;
  container.style.position = 'fixed';
  container.style.top = '0';
  container.style.left = '0';
  container.style.width = '100%';
  container.style.height = '100%';
  container.style.zIndex = '9999';
  container.style.background = 'white';
  document.body.appendChild(container);
  const form = container.querySelector('form');
  if (form) form.submit();
}
```

---

## 私钥格式

**沙箱返回两个私钥字段，必须按语言选对：**

| 字段 | 格式 | 使用场景 |
|------|------|---------|
| `appPrivateKey` | PKCS#8 格式 | ✅ Java 语言必须使用 |
| `appPrivatePkcsKey` | PKCS#1 格式 | ✅ 非 Java 语言（Python、Node.js、PHP、.NET 等）必须使用 |

**严禁以下误判行为：**

| 误判行为 | 为什么错 | 正确做法 |
|---------|---------|---------|
| 看到 `ERR_OSSL_UNSUPPORTED` 就推断私钥格式有问题 | 该错误也可能来自私钥复制污染、字段选错或 SDK 配置参数错误，不能证明私钥格式或运行时不兼容 | 先确认项目实际读取值与可信配置原值一致，再检查字段映射和 SDK 参数 |
| 向 SDK 配置写入 PEM 头尾、其他前后缀、包装行或说明文字 | 支付宝 SDK 会按配置自行处理密钥包装，手动添加会导致格式错误 | SDK 配置直接使用沙箱或生产配置中的原始私钥值，不做任何格式处理 |
| 将沙箱返回的 `appPrivatePkcsKey` 转换为 PKCS#8 格式 | 沙箱已同时返回两种格式，转换会引入不必要风险 | 非 Java 语言使用原始 `appPrivatePkcsKey` 值，不做转换 |
| 将沙箱返回的 `appPrivateKey` 转换为 PKCS#1 格式 | 沙箱已同时返回两种格式，转换会引入不必要风险 | Java 语言使用原始 `appPrivateKey` 值，不做转换 |
| 写入 SDK 配置前先对私钥做 Base64 解码/重新编码 | SDK 配置无需这些处理 | SDK 配置直接使用沙箱返回的原始字符串 |
| 因为字段名含 "Pkcs" 就认为是 PKCS#8 格式 | `appPrivatePkcsKey` 是 **PKCS#1** 格式，"Pkcs" 仅表示这是 PKCS 系列标准之一，不代表 PKCS#8 | 非 Java 语言使用 `appPrivatePkcsKey`（PKCS#1），Java 语言使用 `appPrivateKey`（PKCS#8） |

**核心原则：沙箱配置直接使用沙箱返回的对应私钥字段，不做格式转换；生产配置必须确认应用私钥格式与项目语言匹配。所有写入 SDK 配置的私钥值都使用不带 PEM 头尾或其他前后缀的原始密钥字符串，禁止手动添加 PEM 头尾、添加其他前后缀、编码处理或换行符调整。Java 语言使用 PKCS#8，非 Java 语言（Node.js、Python、PHP、.NET 等）使用 PKCS#1。**

**调用边界说明**：上述禁止加工规则针对持久化配置和 SDK 配置。若 A2M 等业务协议需要直接调用语言原生密码库生成业务签名，而该密码库明确要求 PEM 或密钥对象，可从已经校验一致的原始私钥临时构造调用入参；禁止写回配置、覆盖原始值，或把这种调用适配解释成私钥修复。支付宝 SDK 请求签名仍直接使用原始字段，不得预先包装。

### 生产密钥替换与生产支付测试提醒

当用户要求 Agent 直接替换生产环境密钥配置，或用户声称已经替换完成并要求启动生产环境支付测试时，必须先提醒并核对以下事项：

1. `appId`、应用公钥、应用私钥必须属于同一套生产应用密钥，不得混用沙箱应用、其他生产应用或另一套重新生成的密钥。
2. 基于项目编程语言确认应用私钥格式：Java 使用 PKCS#8；非 Java（Node.js、Python、PHP、.NET 等）使用 PKCS#1。
3. 如果生产应用私钥格式与当前语言 SDK 要求不一致，请使用支付宝开放平台密钥工具进行私钥格式转换，再将转换后的原始私钥字符串写入项目配置。
4. 不得把私钥输出到日志、对话总结、测试结果或完整支付表单中。

### 连环误判链

遇到 `ERR_OSSL_UNSUPPORTED` 时，agent 容易陷入以下**越错越远**的连环推理，**每一步都是错的**，必须识别并阻断：

```
❌ 连环误判链（严禁陷入）：

第 1 步（误判起点）：看到 ERR_OSSL_UNSUPPORTED → 推断"私钥格式有问题"
    ↓
第 2 步（错误修复）：认为"给 SDK 配置添加 PEM 头尾可以修复私钥"
    → 改写 SDK 配置或原始私钥 → 错误仍在
    ↓
第 3 步（错误归因）：认为"Node.js 18+ 用了 OpenSSL 3.0，旧格式不被支持"
    ↓
第 4 步（越错越远）：认为"应该把原始私钥改成另一种格式"
    → 将私钥转换为 PKCS#8 格式 → 错误更严重（注：Java 语言应直接使用沙箱返回的 `appPrivateKey` 字段，而非手动转换格式）
```

**正确做法**：看到 `ERR_OSSL_UNSUPPORTED`，立即停止改写原始私钥或 SDK 配置，先核对项目读取值与可信配置原值，再按下方「验签失败排查原则」逐项检查。原生密码库所需的临时调用适配仍按上方“调用边界说明”处理，不能用于修复配置。

---

## 验签失败排查原则

遇到 `invalid-signature`（验签出错）时，**严禁凭猜测归因到私钥格式或环境兼容性**，必须按以下顺序逐项排查：

### 正确排查顺序

**原则：先验证实际请求内容，再怀疑密钥配置。**

1. **检查实际发送、签名和验签参数的一致性**
   - 支付宝验签时要求实际请求参数与签名内容匹配。如果 `return_url`、`notify_url` 等参数已传入支付请求但未正确参与签名，或者在签名后被改写，可能导致验签失败。用户明确关闭同步回跳时，未传 `return_url` 本身不应被当作验签失败原因。
   - **排查方法**：比较实际发送给支付宝的请求参数、SDK 参与签名的参数和服务端验签时使用的参数，确认参数名、参数值和编码方式一致。
   - 未关闭同步回跳：确认实际请求、签名内容和验签参数中的 `return_url` 值一致。
   - 已关闭同步回跳：确认实际请求和签名内容均未包含 `return_url`。

2. **检查私钥来源**
   - 确认使用的是沙箱返回的正确私钥字段：Java 使用 `appPrivateKey`，非 Java 使用 `appPrivatePkcsKey`，而非自行生成或其他来源。

3. **检查 signType 是否一致**
   - 请求中的 `sign_type` 必须为 `RSA2`，与配置保持一致。

4. **检查私钥格式完整性**
   - 私钥内容无多余空格、换行符。

5. **检查网关地址**
   - 沙箱环境必须使用沙箱网关 `https://openapi-sandbox.dl.alipaydev.com/gateway.do`，生产环境使用 `https://openapi.alipay.com/gateway.do`。

> **核心教训**：验签失败 ≠ 私钥问题。应先通过日志/输出验证实际请求行为（参数是否完整传入），而非基于技术传言做假设。

---

## 网站支付同步回跳

用户未明确要求关闭同步回跳时，网站支付默认配置 `return_url`。该值不能只是一个格式正确的 URL，还必须对应目标项目内真实存在的 GET 路由和可正常渲染的结果页。

1. 从项目实际运行配置确定协议、主机、端口和路径，禁止使用示例域名、猜测端口或未实现的路由。
2. 结果页不存在时，必须在当前项目中实现，不得只填写 SDK 的 `return_url` 参数。
3. 启动服务后对不带支付参数的精确 `return_url` 发起 GET 请求。该请求只验证路由和页面壳：没有订单上下文时展示安全的中性状态，不得放宽真实回跳请求的验签和订单归属校验。跟随有限次重定向，确认最终响应无 404、500、认证循环或重定向循环。单页应用直接刷新该路由仍必须返回应用页面。
4. 检查 SDK 实际传入的 `return_url` 与已验证地址完全一致，防止域名、端口、协议或路径在配置映射中被替换。
5. 回跳处理必须先验证支付宝同步参数签名，或使用服务端保存且与当前用户会话绑定的订单标识定位待查订单；不得信任可任意修改的订单号。
6. 同步回跳参数不是支付成功的可信依据。页面应先展示“正在确认支付结果”，再通过服务端交易查询或已验签的异步通知状态展示最终结果。
7. 当前 Agent 有浏览器或 UI 验证能力时，必须实际打开回跳页，确认非空白、无可见报错且关键状态内容已渲染。当前无该能力时，向用户提供精确回跳地址请其打开确认；取得确认前不得宣称页面渲染已验证或网站支付集成完成。

回跳地址无法访问、路由不存在、页面无法渲染，或 SDK 实际值与已验证地址不一致时，必须先修正，不得宣称网站支付集成完成。

---

## 时间戳格式化

支付宝要求时间戳格式为 `yyyy-MM-dd HH:mm:ss`（空格分隔，非 ISO 格式）。

### Java
```java
import java.text.SimpleDateFormat;
// ✅ 正确
new SimpleDateFormat("yyyy-MM-dd HH:mm:ss").format(new Date());
// ❌ 错误：Instant.now().toString() → "2026-04-29T14:30:00.000Z"
```

### .NET (C#)
```csharp
// ✅ 正确
DateTime.Now.ToString("yyyy-MM-dd HH:mm:ss");
// ❌ 错误：DateTime.Now.ToString("o") → "2026-04-29T14:30:00.0000000+08:00"
```

### PHP
```php
// ✅ 正确
date("Y-m-d H:i:s");
// ❌ 错误：date("c") → "2026-04-29T14:30:00+08:00"
```

### Node.js
```javascript
function formatAlipayTimestamp(date) {
  const pad = (n) => n.toString().padStart(2, '0');
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`;
}

// ❌ 错误：new Date().toISOString() → "2026-04-29T14:30:00.000Z"
```

### Python
```python
from datetime import datetime

def format_alipay_timestamp(dt: datetime) -> str:
    return dt.strftime("%Y-%m-%d %H:%M:%S")

# ❌ 错误：datetime.isoformat() → "2026-04-29T14:30:00"
```
