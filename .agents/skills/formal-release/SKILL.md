---
name: Formal Release
description: 为 honeclaw 仓库执行正式版本发布：补版本号、编写 `docs/releases/vX.Y.Z.md`、完成必要验证、提交变更、推送 `main`、创建并推送 `v*` tag。用户明确要求“发正式版本 / 打 tag / release / release note / 把上次 release 到现在的变更发出去”时使用；不适用于普通 push。
when_to_use: 当用户明确要求正式发版、打版本 tag、补 release notes，或要求把自上次 release 以来的改动整理成新版本并发出去时使用
user-invocable: true
context: inline
aliases:
  - 发版
  - 正式版本
  - release
  - ship release
---

## Formal Release

把正式发版当成一条完整交付链路，而不是“推一次代码”。

先读 [references/release-checklist.md](references/release-checklist.md)。它包含本仓库的版本号位置、release note 约束、验证基线、push/tag 顺序，以及常见失败处理。

如果本次发版触达架构面（poller / EventKind / agent runner / tool / channel / crate 边界 / storage 布局 / frontend surface），还要读 [references/architecture-svg.md](references/architecture-svg.md)，按它把 `resources/architecture.svg` 一并刷新。

## 工作流

1. 先确认这是“正式发版”而不是普通 push。
2. 检查当前分支、工作区、远端分叉、最近 tag，以及上次 release 到现在的变更范围。
3. 选定新版本号，并同步更新：
   - `Cargo.toml`
   - `Cargo.lock`
   - `bins/hone-desktop/tauri.conf.json`
4. 创建 `docs/releases/vX.Y.Z.md`：
   - 从 `docs/templates/release-notes.md` 出发
   - 内容必须覆盖自上次 release 以来的真实用户影响
   - 中文在前，英文在后
5. 如果这次 release 吸收了活跃任务里的重要工作，同步更新对应 plan / handoff / archive 文档，不要只改代码和 release note。
6. 如果本次发版触达架构面，按 [references/architecture-svg.md](references/architecture-svg.md) 刷新 `resources/architecture.svg`（hero 数字、layer 卡片、milestones、版本卡都要核对），并本地渲染确认无字体/越界回退。可跳过的，需在 release notes 里写一句“架构 SVG 不需要更新”。
7. 运行最小必要验证，并额外补改动面定向测试。
8. 原子提交 release 相关改动；不要绕过 hook。
9. 先把 `main` 推上去，再打 annotated tag，再把 tag 推上去。
10. 最后明确汇报 commit、tag、验证结果，以及 release workflow 是否已经被触发。

## 严格规则

- 用户如果只要求 `push`，不要默认执行正式发版。
- `docs/releases/vX.Y.Z.md` 不存在时，禁止打 tag。
- 不要使用 `git worktree`。
- 不要用 `--no-verify` 绕过 hook。
- 遇到 non-fast-forward，先 rebase 再重推；不要强推覆盖远端。
- pre-push 如果卡在 `rustfmt` 或 `gitleaks`，修完再推，不要假装已经发版完成。
