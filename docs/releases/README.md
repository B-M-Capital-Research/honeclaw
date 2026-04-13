# Release Notes

- 每个发布 tag 都应有一份对应的用户向 release notes：`docs/releases/vX.Y.Z.md`
- 先复制 `docs/templates/release-notes.md`，再按真实用户影响填写
- 所有 release notes 默认使用中英文双语：中文在前，英文在后，并在开头显式提示英语读者直接往下翻
- 重点写“用户应该知道什么”，不要把 changelog 写成内部提交流水账
- 至少覆盖：
  - 用户可感知的新能力 / 修复
  - 谁需要更新
  - 升级后行为变化
  - 安装 / 更新方式
  - 已知注意事项
- release workflow 会读取 `docs/releases/<tag>.md`；文件缺失时发布会失败
