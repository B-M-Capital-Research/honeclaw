# 2026-09-06 对话页滚动：一会儿被拉到最顶，一会儿被拉到最底

## 现象

用户：「时不时就会拉到最顶上，时不时就会拉到最下面，感觉很奇怪，不是很丝滑。」

## 复现

本机造一个真实场景：把 dev-login 用户的 `cloud_sessions` 灌成 120 条消息、每条 assistant 约 2,000 字
（含表格与代码块，制造异步渲染带来的高度变化），同一个本地后端（8088）同时跑两份前端——
线上代码（origin/main，4177）与修复版（4176）。测量脚本读 `.public-chat-messages` 的
scrollTop / scrollHeight / 消息数 / 「回到底部」按钮是否出现。

线上代码的结果：

- 冷启动一个长会话，落在 **scrollTop 0**（离最新消息 397,254px），之后再也不动。
- 一路上滑到顶：消息数始终 20，**没有加载更早的历史**；`.public-chat-scroll-down` 从不出现。

后者说明 `handleMessagesScroll` **根本没在跑**：它一进来就被 `suppressScrollUntil / isBottomPinned`
挡掉并 return，所以 awayFromBottom、翻页、`stickToBottom` 的更新全都没发生。

## 三个缺陷

1. **ResizeObserver 从来没装上。** 它建在一个读 `let messagesInnerRef` 的 effect 里；消息列表渲染在
   `authState() === "ready"` 之后，而这个 effect 只跑一次、又没有响应式依赖，那一次 ref 还是 undefined，
   于是直接 return 且永不重跑。首屏 `pinToBottom` 的 6 次跳转在 360ms 内打完，而每个气泡的 Markdown 是
   异步渲染的——长会话此时高度接近 0，`scrollHeight - clientHeight` 就是 0，视口停在顶部，之后没有任何东西
   把它拉下来。短会话在 360ms 内渲染得完，所以只是「时不时」。
2. **钉底窗口吞掉用户手势。** 钉底期间（最长 1.8s）所有 scroll 事件被忽略，`shouldRecoverPinnedBottom`
   还会主动把视口拽回底部，用户此时的上滑没有任何出路。
3. **restore 用发起时捕获的标志决定是否回到底部。** 流式结束后 `restoreSession({keepAtBottom})` 要走一次网络；
   落地时用户往往已经上滑，却仍按发起时的 `true` 重新 `pinToBottom` —— 这就是「读到一半被拉回最新一条」。
   非钉底分支用一次性的 `scrollHeight` 差值补偿，而异步气泡还没长出来，差值系统性偏小，视口落在新插入
   历史的顶端；再一个向上的动作又满足 `scrollTop <= 24`，于是继续翻页——表现为「一直被拉到最顶上」。

## 改法

- `messagesInner` 改成信号 ref，ResizeObserver 在列表挂载时才建立（缺陷 1）。
- 滚轮 / 触摸 / 拖滚动条按 `isLeaveBottomGesture` 判定后立刻解除钉底；`scrollToBottom` 在 `stickToBottom`
  为假时直接返回，钉底排队的延迟跳转不再补刀（缺陷 2）。
- restore 落地时用 `shouldKeepBottomAfterRestore` 按**当时**的滚动状态决定；非钉底分支与「加载更早」都改成
  **锚定某条消息的屏幕位置**，并在 60/150/320/600ms 各校正一次——每次只按当时量到的漂移微调，异步内容
  落多少就补多少（缺陷 3）。
- 其中 `shouldKeepBottomAfterRestore` 与 `isLeaveBottomGesture` 沿用 2026-09-02 那轮未上线的滚动修复
  （owner: Finn-Fengming），按当前文件重新落位。

## 验证

| | 线上 origin/main | 修复版 |
|---|---|---|
| 长会话冷启动落点 | scrollTop 0，离底 397,254px | 离底 **0** |
| 上滑后「回到底部」按钮 | 从不出现 | 出现 |
| 上滑到顶后加载更早 | 不触发（消息数恒为 20） | 触发（20 → 40） |
| 加载更早后视口 | —— | 锚定消息漂移 252px，未跳到顶部 |

`bun test` 586 通过（新增 3 组：落地时决定、手势判定、补偿失败会继续翻页），typecheck 通过。

## 追加：第四个缺陷（浏览器夹取被当成用户上滑）

上面三条修完后再看「发一条消息」这个路径，视口仍然会停在顶部再也不跟随。给容器打标记后确认：
DOM 节点没有被重建，**没有任何脚本写过 `scrollTop`**，是浏览器自己把它夹到了 0——reconcile 让列表
瞬间塌陷，滚动范围随之变小。这个夹取以一次普通的「向上滚动」事件到达 `handleMessagesScroll`，
于是 `stickToBottom` 被清掉；此后所有钉底与 ResizeObserver 都成了空转，视口就一直留在顶部。

判据（`isLayoutDrivenScroll`）：向上的滚动只有在**内容没有变短**、或**1.2 秒内发生过真实手势**
（滚轮 / 触摸 / 拖滚动条）时，才算用户的意图；否则按布局处理，保持原来的跟随状态。
手势入口顺带记录时刻，用户真的在滑时永远优先。

## 环境提醒（这次踩到）

浏览器面板处于隐藏 / 未渲染状态时，**rAF 与 ResizeObserver 都不投递**。本次后半段一度测出
「修复无效」，实为面板挂起：自己新建的 ResizeObserver 同样一次都没触发、`requestAnimationFrame`
回调也没跑。滚动相关的验证必须在面板确实在渲染时做；拿不准就先测一次 rAF 是否回调。

## 未做

- 真实流式回答下的手动复现（本机没有 LLM key）。已用「发送 → 后端返回失败文案」跑通同一条
  时序（发送 → run_finished → 钉底 → restore 落地 → reconcile），第四个缺陷就是这样抓到的；
  但真实长回答的逐字流式仍未复现。
- 第四条修复只有纯函数级测试与机制证据（夹取时无脚本写入 `scrollTop`），没有在活的渲染循环里
  回归——面板挂起后没法再测。上线后值得再看一次「发一条消息 → 回答落地 → 上滑」。
- `pinToBottom` 仍然排 6 次跳转。手势解除后它们会自行失效，这次没有改它的节奏。
- 主 checkout 里 2026-09-02 那轮的未提交改动没有动；它的基线是被重写前的 `chat.tsx`，需要时按本次结果重做。
