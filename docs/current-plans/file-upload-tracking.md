# 任务：用户上传文件追踪与 pageIndex 结合评估

## 目标
- 盘点当前项目内“用户上传文件”追踪链路（入口、存储、索引、展示）
- 评估与 `pageIndex` 结合的可行点
- 找出接入最大的 blocker

## 涉及文件
- `memory/src/kb.rs`
- `memory/src/lib.rs`
- `crates/hone-channels/src/attachments.rs`
- `crates/hone-channels/src/core.rs`
- `bins/hone-*/src/main.rs`
- `bins/hone-console-page/src/main.rs`
- `packages/app/src/**`（KB 页面与类型/接口）
- 相关配置：`config.example.yaml`、`config.yaml`

## 计划
1. 代码检索附件/上传/KB/pageIndex 相关入口与数据结构
2. 梳理当前追踪链路（上传 -> 保存 -> 列表/详情 -> 前端展示）
3. 查找 `pageIndex` 的现有定义与使用
4. 评估结合点与技术约束，识别最大 blocker

## 验证
- 本任务为阅读与分析，暂无代码改动；若改动后续产生，按需补测

## 文档同步
- 更新 `docs/current-plan.md` 索引
- 若有中等以上变更或需要交接，再补 `docs/handoffs/*.md`
