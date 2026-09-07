# User Product README Refresh

- title: 中英文 GitHub 首页转为用户端 AI 基础设施投研介绍
- status: done
- created_at: 2026-09-07
- updated_at: 2026-09-07
- owner: Codex
- related_files: README.md, README_EN.md, README_ZH.md, resources/readme/hone-investment-assistant.png, resources/readme/hone-serenity-mobile.jpg
- related_docs: docs/archive/plans/readme-user-product-2026-09-07.md, docs/wiki.md
- related_prs: none; local documentation changes only

## Summary

三份 README 完整更新：默认首页和 README_EN.md 保持英文且逐字一致，README_ZH.md 提供语义对应的中文。产品介绍以用户的 AI 基础设施研究路径为主，旧管理端图与特性列表退出 README，原有图片文件未删除。

## What Changed

- 围绕自然语言问答、约 50 家公司本体研究的能力范围、关注公司与专业金融数据、Serenity 动态、宏观信号对 AI 基础设施的传导介绍产品。
- 本体仅保留覆盖方向与用途，不导出、摘录或展示研究内容；文末说明线上数据服务与公开代码的范围差异。
- 安装、架构和开发说明收敛为文末 Wiki / repo-map / AGENTS / releases 入口。
- `resources/readme/hone-investment-assistant.png`：用户为本次 README 更新提供的截图，原尺寸 3006 × 1604，原图复制，无内容修改。
- `resources/readme/hone-serenity-mobile.jpg`：2026-09-07 从已登录线上 `https://hone-claw.com/research?panel=influencer-digest` 截取，浏览器视口 430 × 932；展示列表顶部 Serenity 卡片、原文配图、来源链接和问 HONE。是移动 Web 视口截图，不声称来自原生 iOS；完成后已恢复浏览器视口。截图 API 返回 JPEG，落盘后按实际格式使用 `.jpg`。

## Verification

- `git diff --check` 通过。
- `cmp README.md README_EN.md` 通过。
- Python 检查三份 README 的 Markdown/HTML 本地链接：每份 8 项全部存在；HTML 段落标签成对。
- 图片文件签名匹配扩展名，`sips -g pixelWidth -g pixelHeight` 确认 3006 × 1604 和 430 × 932；人工查看移动截图确认 Serenity、卡片、原文与来源入口清晰可见。
- 核对用户提供的约 50 家覆盖口径，以及本地 `public-research.tsx` 的 52 家研究基线描述；正文保留约数，不公开本体文件。
- 金融数据能力对照 `crates/hone-tools/src/data_fetch.rs` 的 financials / metrics / ratios / valuation 路径；宏观与 AI 维度对照 `daily-signal-dashboard.tsx`。
- 大 V 同步频率以当前线上可见的“每 15 分钟”及最近同步时间为产品介绍依据；当前本地 checkout 的旧 `public-research.tsx` 文案仍为日报，未在本次文档任务改动运行时代码。
- 无运行时代码变化，未运行 Rust、PostgreSQL、前端单元测试；外部链接未做逐一 HTTP 可达性检查，保留原项目入口并实际查看线上研究页面。

## Risks / Follow-ups

截图是界面样例，未核验其中市场叙述、作者观点与数字；README 图注说明内容与数据以使用时来源为准。未来更新 UI、作者来源或频率时应重截图片并同步双语文案。专业数据覆盖、账户权限与自部署配置影响可用能力。

## Next Entry Point

从 README_ZH.md / README.md 审阅最终文案。本次不涉及运行时行为、模块边界或长期规则，无需更新 repo-map、invariants、decisions。未提交、推送、打 tag 或部署。

## Publication Follow-up · 2026-09-07

用户要求 GitHub 首页实际更新，继续执行提交与推送。发布 todo：保存本次文档变更、同步 origin/main、保留远端新增的历史归档、复查文档链接与英文一致性、执行提交/推送检查、验证远端默认分支的 README 和图片。本阶段为一次性 Git 同步，不新增动态计划或第二份 handoff；不涉及 tag 或正式 release。

同步最新 main 后，线上每 15 分钟的速报频率已与当前源码一致。归档索引冲突保留远端 2026-08-22 至 2026-09-05 的全部新增记录，并在其前追加本任务。发布验证结果以本任务最终交付中的远端 commit 与页面检查为准。
