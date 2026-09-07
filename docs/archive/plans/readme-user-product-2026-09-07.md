# User Product README Refresh

- title: 中英文 GitHub 首页转为用户端 AI 基础设施投研介绍
- status: archived
- created_at: 2026-09-07
- updated_at: 2026-09-07
- owner: Codex
- related_files: README.md, README_EN.md, README_ZH.md, resources/readme/
- related_docs: docs/current-plan.md, docs/handoffs/2026-09-07-readme-user-product.md

## Goal

完整重写中英文 README，围绕用户端问答、约 50 家 AI 基础设施公司研究、关注公司与专业金融数据、Serenity 动态、宏观红绿灯介绍产品，不公开本体研究内容。

## Scope

- [x] 核对现有 README、客户端入口和线上大 V 速报能力。
- [x] 保存用户提供的问答截图，截取线上手机视口 Serenity 动态。
- [x] 重写 README.md / README_EN.md / README_ZH.md，保持英文副本一致。
- [x] 验证：图片可读、链接与相对路径有效、双语语义一致、git diff --check。
- [x] 更新上下文文档：记录图片来源与能力边界到 docs/handoffs/2026-09-07-readme-user-product.md。
- [x] 完成归档：计划移入 docs/archive/plans/，从 docs/current-plan.md 移除，更新 docs/archive/index.md。

## Validation

文档与静态图片变更；核对代码/线上 UI、Markdown 链接、截图与中英文一致性，不运行无关 Rust/数据库测试。

## Documentation Sync

本任务包含截图交付与明确的文档留存，单独建计划并归档。无模块边界、运行时行为、测试契约变化，无需修改 repo-map、invariants 或 decisions。

## Risks / Open Questions

专业数据库和研究资产属于线上服务能力，不表示公开仓库包含授权数据或本体内容。截图只是界面示例，不是对其中市场事实的核验。同步频率按线上 UI 描述。

## Completion

2026-09-07：README 与两张图片交付完成；验证与 handoff 已完成，任务移出活跃索引并归档。仅文档/图片改动，未提交或推送。
