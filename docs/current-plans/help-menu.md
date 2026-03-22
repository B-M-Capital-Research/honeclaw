# Help Menu Documentation System

## 目标
建立一个本地 Help Menu 帮助系统，支持 Markdown 文档的自动发现与渲染。
用户要求在桌面的“帮助”菜单中，能够选择不同的文档进行阅读，并要求建立 `help` 文件夹存放这些文档，目前包含 `README.md` 和 `启动文档.md`。

## 涉及文件
- [NEW] `help/README.md`: 帮助页面介绍文档。
- [NEW] `help/启动文档.md`: 桌面版使用入门与启动配置指南。
- [MODIFY] `bins/hone-desktop/src/main.rs`: 增加 `get_help_docs` 和 `read_help_doc` Tauri Core IPC Commands。
- [MODIFY] `packages/app/src/lib/backend.ts`: 增加读取上述 Commands 的 TypeScript 接口。
- [MODIFY] `packages/app/src/pages/help.tsx`: 重新设计帮助页面，左侧列表，右侧 Markdown 渲染 (基于 `marked` + `dompurify`)。

## 验证步骤
1. **Rust 编译验证**：`cargo check -p hone-desktop` 确保无编译错误。
2. **前端 TS 验证**：在 `packages/app` 执行 `bun run typecheck`。
3. **功能验证**：本地启动 `./launch.sh` (或 `cargo run -p hone-desktop`) 并在帮助页面查验文件是否正确拉取，并且 Markdown 的标题、高亮、列表、表格均正常渲染。

## 文档同步步骤
- 本次改动涉及前端及架构层细微变更，完成后将输出 `docs/handoffs/YYYY-MM-DD-help-menu.md`，并在 `docs/current-plan.md` 标记完成。
