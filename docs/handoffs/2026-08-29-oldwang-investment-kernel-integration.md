# oldwang 默认投资内核整合交接

## 结果

HONE 默认投资内核升级为 Hari Invest `0.3.0`，已合入并推送 `origin/oldwang`。内核标注提交为 `2ecb072a`；这次整合同时保留此前统一决策大脑、SNDK 深度回答重构和公共搜索兜底。

## 进入内核的内容

- 强规则：`LOG-V0001`—`LOG-V0006`，版本均为 0.1；没有自动新增或修改老王确认逻辑。
- 通用方法：SNDK 重构形成的需求、供给/替代、公司价值捕获、财务兑现、三情景估值、反向隐含要求和条件化动作链。
- 公司基线：命中覆盖公司时加载 `company-thesis-ratings`，只用于商业模式、护城河、产业位置、风险和证伪的历史基线。
- 当前事实：继续由行情、SEC/IR、财报、新闻、产业和持仓工具在每轮重新核实。

## 保持隔离的内容

- `CAND-20260813-06` 仍是候选；`OPEN-20260813-01` 仍是开放冲突；`SCOPE-20260813-02` 仍是长期范围。
- 不把 SNDK 某次价格、财务、预测、目标价或动作固化进内核。
- 没有生产部署、真实训练、RL、自动下单、券商连接或自主操盘授权。
- 原始逐字稿、账户、私有路径和临时 SNDK 下载文件没有进入提交。

## 主要文件

- `skills/hari-invest/references/kernel-manifest.md`
- `skills/hari-invest/CHANGELOG.md`
- `skills/hari-invest/evals/evals.json`
- `crates/hone-channels/src/prompt.rs`
- `tests/regression/ci/test_hari_invest_conversation_contract.sh`
- `docs/decisions.md` 的 D-2026-08-29-199

## 验证

- Hari 对话内核契约、公司研究对话契约、研究资料治理、Skill runtime 阶段一致性与工具测试均通过。
- 财经自动化契约 49/49 通过；默认提示和 SNDK 深度逻辑 Rust 定向测试通过；Rust 全量格式检查通过。
- 推送前对全部 7 个待推送提交执行密钥扫描，约 16.04 MB，未发现泄漏；推送前 Rust 文件格式门禁通过。
- SNDK 491 轮中的代码契约覆盖与 live 模型验收继续分开描述，不把尚未执行的 live 回答写成通过。

## 后续入口

下一步若要把 8 月 13 日决策闭环升级成老王确认逻辑，必须回到蒸馏流程完成本人确认、反例/边界测试和一鸣产品范围确认；不能直接改提示词晋级。生产部署和真实交易仍需单独授权。
