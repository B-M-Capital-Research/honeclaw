# Stage 129 期初组合来源工件隔离接收器规格登记

## 已完成

- 后端新增 create-once、自哈希、责任链外的隔离接收器规格 registry，并在每次读取时重验当前 Stage 128 独立批准绑定。
- 规格冻结拟议工件身份、代码版本、复现程序、固定非特权 runtime、管理员鉴权流式 PDF/CSV/JSON 输入、未受信 manifest 输出和继承资源上限。
- 管理端、API/types、历史治理页、统一决策大脑与 readiness v126 已接通。

## 明确为空

- 没有上传入口、来源字节、quarantine/artifact、可执行工件、入口、runtime、input mount/read、receipt 或真实 Stage 129 记录。
- 没有 opening snapshot、financial-event allowlist、账本、持仓、现金、NAV/绩效、模型、训练/RL/reward、订单、券商或交易状态。

## 验证

- Stage 129 聚焦 Rust 5/5；HONE Web API 1281 passed、2 ignored；前端 698/698、3471 assertions；金融自动化契约 49/49。
- TypeScript、标准/public 双模式生产构建、workspace all-target check、Rust fmt、diff hygiene 与零真实财务状态审计通过。
- 每次读取除重算完整记录指纹外，还逐项复核 17 项确认，不能只凭 `confirmations_complete` 总开关放行。

## 下一步

- Stage 130 最多只能实现责任链外首次执行授权复核：从服务端派生的只读内容寻址保管区重哈希真实工件与 manifest，生成限时、一次性、尚未执行的授权候选。
- 不得在 Stage 130 上传或读取来源文件、运行接收器、写 receipt 或物化期初组合。
