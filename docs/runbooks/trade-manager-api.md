# 交易管理 API v1

同源前缀：`/api/public/trade-manager`。浏览器使用现有 HttpOnly `hone_web_session` cookie，`credentials: same-origin`。所有成功和错误响应都禁止缓存。接口不接受 userId、actor 或其他所有者参数。

| 方法 | 路径 | 请求体 | 成功响应 |
| --- | --- | --- | --- |
| GET | `/state` | 无 | `{state, revision, context}` |
| PUT | `/state` | `{state, revision, label?, requestId?}` | `{state, revision, context}` |
| GET | `/history` | 无 | `{history: [{id, date, label, revision}], context}` |
| GET | `/history/{id}` | 无 | `{id, date, label, revision, state, context}` |
| POST | `/history/{id}/restore` | `{revision, requestId?}` | `{state, revision, context}` |

首次 GET `/state` 可以不带 context。成功但从未保存的账户返回 `state:null, revision:0`；客户端只有在这个成功响应后才建立空白模型，不把请求失败当作新账户。

PUT、restore 和全部 history GET 必须带 `X-Hone-Trade-Context: <首次读取返回值>`。后续 state GET 也应携带它；一旦携带，服务端会严格验证。context 是会话与服务端 actor 的绑定摘要，不是登录凭证，不得存入浏览器长期存储或日志。服务端仍对每次请求检查真实登录 cookie 和既有权限。

`revision` 是客户端读到的版本；每次成功保存或恢复增加 1。所有修改都用当前 revision 做比较；并发竞争只能有一个成功。历史恢复同样生成新版本。服务器保留当前快照及此前最多 8 个保存版本，`history` 不返回当前版本；初次保存前的 `null` 不是可恢复记录。

`requestId` 应传随机 UUID；客户端每个新操作生成一次。相同 requestId、相同原版本及相同内容，只在原结果仍是当前版本时可返回原结果。重用 requestId 写其他内容，或其后已有更新，返回冲突。历史窗口已清理的旧请求没有无限去重保证，但旧 revision 仍阻止覆盖。前端不自动重试有副作用请求；响应丢失后先重新加载核对。

请求 JSON 总体上限 **2 MiB**（包括封包字段）；label 为 1–100 个字符。state 沿用交易台 schemaVersion 1，服务端做字段、数量、日期、数值和已录入账务快照校验。它不能证明用户录入数据真实，也不是券商流水审计系统。

| 状态码 | 含义与客户端处理 |
| --- | --- |
| 200 | 成功，采用返回版本和快照。 |
| 400 / 422 | 请求或快照不符合契约，不得显示已保存。 |
| 401 | 登录缺失/失效，清除页面内所有账本与导出链接。 |
| 402 | 沿用 Hone 产品访问权限限制，清除已加载内容。 |
| 403 | 会话绑定不符或权限拒绝；清除旧会话内存，不得改用新 cookie 盲重试。 |
| 404 | 当前用户没有此历史记录；不泄露其他用户是否存在相同 ID。 |
| 409 | 版本过期或请求冲突。保留本次候选副本，校验身份后供下载；显式重新加载。 |
| 413 | 请求体过大，缩减或分阶段整理后重试。 |
| 503 | 数据库不可用/未配置。保留当前资料，不回退本地存储，不声明写入成功。 |

## 前端父子消息

仅固定同源 iframe 与其父页面之间使用消息，接收方同时验证 `event.origin` 和 `event.source`：

- `hone:trade-status`：只含 `dirty`、`busy` 两个布尔值，供关闭保护。
- `hone:trade-close-request`：返回 Hone 的关闭请求，仍受父页面的未保存/保存中保护。
- `hone:auth-invalidated`：父页面通知子页面立即清除。
- `hone:trade-auth-invalidated`：子页面发现 401/402/403 或 context 不符后，向父页面发送无数据失效通知；父页面撤销下载链接并关闭交易工具，不注销 Hone 全局登录。普通加载/重新加载不发送此通知。
- `hone:trade-export`：用户主动导出并通过身份复核后，把 `{name, content, mime}` 临时交给**同源父弹窗**，由父页面展示原生下载链接，兼容内嵌浏览器的下载限制。仅允许 JSON、CSV、Markdown，UTF-8 不超过 8 MiB；父端校验消息来源、文件名及类型，不把文本插入 HTML。链接 15 分钟失效，替换、关闭或认证失效时销毁。它不是跨标签页广播，也不会上传到第三方。

跨标签页频道 `hone-auth-lifecycle` 只广播 `{type:'invalidate'}`，不传用户、账本或登录信息。退出由 Hone 原入口处理。若浏览器不支持 BroadcastChannel，跨标签页即时通知不可用；接口绑定仍拒绝旧会话，页面恢复焦点及导出前会重新验证身份。
