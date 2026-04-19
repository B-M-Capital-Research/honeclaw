# 公司画像包导入导出与傻瓜式导入流

- title: 公司画像包导入导出与傻瓜式导入流
- status: done
- created_at: 2026-04-19
- updated_at: 2026-04-19
- owner: codex
- related_files:
  - `memory/src/company_profile/mod.rs`
  - `memory/src/company_profile/types.rs`
  - `memory/src/company_profile/markdown.rs`
  - `memory/src/company_profile/storage.rs`
  - `memory/src/company_profile/transfer.rs`
  - `memory/src/company_profile/tests.rs`
  - `crates/hone-web-api/src/routes/company_profiles.rs`
  - `packages/app/playwright.config.ts`
  - `packages/app/e2e/company-profile-transfer.spec.ts`
  - `packages/app/src/context/company-profiles.tsx`
  - `packages/app/src/components/company-profile-list.tsx`
  - `packages/app/src/components/company-profile-detail.tsx`
  - `packages/app/src/lib/company-profile-transfer.ts`
- related_docs:
  - `docs/archive/plans/company-profile-transfer.md`
  - `docs/invariants.md`
  - `docs/repo-map.md`
  - `docs/archive/index.md`
- related_prs:
  - N/A

## Summary

公司画像现在支持 Memory 页里的 actor 私有 zip 画像包导入导出。导入继续保持目录级 `skip / replace`，并在扫描、冲突审阅、替换前备份和导入完成反馈之间收敛成一条“先自动扫描，再只在必要时做最少判断”的页面流。

## What Changed

- `hone-memory` 新增画像包 manifest、bundle zip 导出、bundle 预览、冲突识别和 `keep_existing / replace_all / interactive` 三种 apply 语义
- `hone-web-api` 新增 company profile transfer API 和 `company_profile_transfer` capability
- Memory 页面左侧改成“目标用户空间”，来源包括已有画像空间、最近会话用户以及手动指定目标
- Memory 页面右侧新增上传区、自动扫描结果、冲突审阅、导入完成提示和备份下载入口
- 后续修复补上了 legacy plain Markdown 画像兼容：即使历史 `profile.md` / `events/*.md` 没有 frontmatter，transfer 导出和替换前自动备份也会先推断标题、时间和最小 metadata，再导出为标准化 bundle
- 左侧目标区进一步收敛成单一“目标用户空间”列表；当前空间里的公司切换移到右侧详情头部下方，避免“目标人 + 公司列表”同时挤在左边
- 前端新增 Playwright E2E：覆盖手动导出、冲突预览、replace 前自动备份、无冲突直导入，以及 Memory 页基础渲染 smoke
- 后续又把 `memory/src/company_profile.rs` 按职责拆成 `types / markdown / storage / transfer / tests` 五个子模块，保持 `hone-memory` 的 re-export 与 transfer 行为不变，避免公司画像继续堆成单个超大文件

## Verification

- `rtk cargo test -p hone-memory company_profile`
- `rtk cargo test -p hone-web-api`
- `rtk bun run test:web`
- `rtk bun run typecheck:web`
- `rtk bun run build:web`
- `rtk bun run --cwd packages/app test:e2e`
- `rtk cargo check -p hone-memory -p hone-web-api -p hone-channels`

## Risks / Follow-ups

- Memory 页面导入仍然故意停留在目录级 `skip / replace`；如果以后要做更细的 merge，应单开任务重新设计，而不是把当前 UI 扩成编辑器
- 对 legacy plain Markdown 的 transfer metadata 目前是“最小推断”模式：公司名来自标题，更新时间来自文件 mtime，ticker 只会在目录名本身像标准股票代码时才推断
- 若后续要支持“单家公司导出”或“服务端持久化备份”，建议单独再开任务，不要把当前 bundle contract 临时扩写

## Next Entry Point

- `packages/app/src/context/company-profiles.tsx`
- `memory/src/company_profile/mod.rs`
