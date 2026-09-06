# V5 桌面导航审查

## 目标

让桌面端的信息架构明确表达：Agent 是主要研究入口；投资、洞察和跟踪是工作台；“我的”属于账户头像，而不是工作导航。

## 步骤与健康度

1. `01-before-desktop-me.png`：原五入口同权侧栏。健康度：需要调整。
2. `02-after-desktop-me.png`：Agent 独立、工作台分组、“我的”合并头像。健康度：通过。
3. `03-after-desktop-agent.png`：Agent 选中及其内部研究工作区。健康度：通过。
4. `04-mobile-agent-unchanged.png`：移动端五入口回归。健康度：通过。
5. `05-compare-desktop-full.png`：完整页面前后同屏比较。健康度：通过。
6. `06-compare-sidebar-focus.png`：侧栏聚焦比较。健康度：通过。

## 证据边界

截图和 DOM 检查确认了布局、入口数量、路由、选中状态和响应式行为。完整键盘遍历及屏幕阅读器播报仍需专项可访问性测试。
