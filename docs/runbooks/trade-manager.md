# Hone 个人交易管理工具接入

## 交付范围

在 `/chat` 的工作流工具菜单中新增“交易管理”，普通已登录且拥有现有使用权限的 Hone 用户可打开。工具沿用现有企业逻辑、事件证据、风险试算、计划审查、手工成交、资金流水、拆股与复盘功能。所有证券动作都是登记；没有券商订单接口。

本补丁基于公开 Hone 源码 `f7d87aff034507a529ccf34b84de2aff7879f5ec`，在独立副本开发，不包含本机 Hone 工作树的其他未提交修改。工程人员应在当前目标分支执行 `git apply --check`；有差异时按对应接口合并，不能用旧版完整 `chat.tsx` 覆盖新版本。

## 架构和身份边界

- UI：`packages/app/src/components/trade-manager-dialog.tsx`；同源固定路径 `/trade-manager/index.html`。iframe 用于隔离样式和生命周期，**不是用户数据隔离边界**。
- 静态业务模块：`packages/app/public/trade-manager/`。模型及导出沿用原交易台；数据加载与保存改成异步服务端接口。没有 demo/personal 切换，不把财务数据放入 localStorage、sessionStorage 或 IndexedDB。
- API：`/api/public/trade-manager/*`，由 Hone `require_public_user` 验证现有 HttpOnly 登录会话及访问权限。服务端根据认证用户构造 actor；客户端不能指定被操作用户。
- 存储：独立 PostgreSQL 表 `cloud_trade_workspaces` 与 `cloud_trade_revisions`，每次查询、保存和历史恢复均带认证 actor。与 Hone 已有 portfolio、研究画像、对话和订阅表分开；不会自动导入或覆盖这些数据。
- 会话绑定：首个认证加载返回当前会话的 `context`，之后请求携带 `X-Hone-Trade-Context`。切换账号后，即使新账号的 revision 相同，旧页面也不能写入新账号。
- 并发：客户端等待服务端确认后才更新界面；服务端 revision 比较和写入在一条原子 SQL 内完成。过期版本返回 409，不能静默覆盖。requestId 用于受限幂等去重，不能作为授权依据。
- 导出：身份校验后，在工具上方显示可点击下载链接，需用户再次点击保存；链接仅在同源父弹窗内临时保留，15 分钟或关闭/身份失效后销毁。
- 退出：父页面、同源 iframe 与跨标签页仅发送“不再有效”的无身份通知；立即清空页面内账户数据，并丢弃旧请求的晚到结果。通知不包含账本、cookie 或用户标识。
- 所有个人数据响应均 `Cache-Control: no-store`。静态 HTML/JS 不含用户信息；API 必须走同源认证后端，禁止 CDN 缓存账户响应。

## 接入步骤

1. 在目标 Hone 分支备份并创建工程分支，检查补丁适用性；合并新增文件和共享入口的小范围改动。
2. 沿用现有 Node/Bun、Rust 与数据库配置。没有新增外部 SaaS、模型 key 或券商依赖。
3. 确认 Hone 的 PostgreSQL 已配置并可用。本功能在缺少数据库时拒绝服务，不回退到本地文件或浏览器保存。
4. 先对测试数据库执行附带的加表迁移；生产由现有数据库变更流程执行（`docs/sql/trade-manager-v1.sql`）。只新增表，不修改现有持仓/账户表。运行时也有一次性兼容建表逻辑；若生产应用角色没有 DDL 权限，应由迁移角色事先建表，初始化会探测已存在的两表并跳过 DDL。
5. 构建后端和公共前端，使用原部署管线发布二者。`public/trade-manager/` 必须进入公共前端产物，`/api/public/trade-manager/*` 必须转发到认证后端。
6. 若使用现有 Pages `_worker.js`，合并本补丁的精确路径 CSP 例外：仅工具入口允许同源嵌入，其余页面维持禁止嵌入。若使用其他反向代理，配置等价策略；不能全站移除 `X-Frame-Options` / `frame-ancestors`。
7. 在测试环境完成下列真实账号验收，再按现有发布流程上线。

## 上线前验收

- 未登录访问账户 API 被拒绝；新登录用户首次打开是空白独立账本。
- 用户 A 保存现金、持仓和计划；用户 B 在同一浏览器、另一浏览器分别打开，均看不到 A 的数据。A 重新登录仍可恢复自己账本。
- A 退出或切换为 B 后，旧 iframe / 标签页不能继续显示或保存 A 的信息；旧 context 与 B cookie 组合必须拒绝。
- 同账号两个标签页读取同一 revision，先保存的一方成功，另一方得到冲突提示，下载自己的待保存副本后可重新加载。
- 模拟网络断开、数据库故障、写入后响应丢失，不得提示已保存；重新加载核对结果，不要重录成交。
- A 猜测 B 历史版本 ID 不能读取或恢复；历史恢复增加新版本，不更改历史版本的所属用户。
- 原版 JSON 可显式导入当前登录用户的账本；导入前下载备份，不自动读取旧网站 localStorage。
- 报价、计划审查、部分成交、拆股、资金流水、复盘筛选和导出，与原有核算口径一致。
- 桌面与手机可打开和关闭工具，保存中与未保存修改得到明确处理。真实生产账号、反向代理和会员策略需由工程测试环境验证。

## 验证命令

```sh
bun install --frozen-lockfile
node --test tests/trade-manager/*.test.mjs
bun test --preload ./packages/app/happydom.ts ./packages/app/src/lib/public-auth-lifecycle.test.ts ./packages/app/src/lib/trade-manager-session.test.ts ./packages/app/src/lib/public-session-cache.test.ts ./packages/app/public/_worker.test.js
bun run typecheck:web
bun run build:web:public
cargo check -p hone-web-api --lib --locked
# 测试 PostgreSQL 的连接参数由工程环境提供，绝不可指向生产数据库。
cargo test -p hone-web-api --lib trade_manager -- --test-threads=1
```

完整接口契约见 [API 说明](trade-manager-api.md)。最终实际执行结果及限制见 handoff，不能把静态编译或模拟界面测试当作生产登录验收。

## 回滚与运维

先关闭新入口或回退本补丁的前后端版本，保留两张交易表。**不要在回滚时 DROP 表**，避免删除用户账本。数据库备份需同时覆盖当前账本与历史版本；浏览器 JSON 导出是用户可携带副本，不能代替服务端备份。

静态版本和 API 契约必须一起更新。没有必要改动登录 cookie、付费策略、其他工具权限或用户研究数据。生产分支若已有交易管理接口，先做命名和数据源协调，避免双写两个账本。
