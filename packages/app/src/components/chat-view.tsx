import { Button } from "@hone-financial/ui/button";
import { EmptyState } from "@hone-financial/ui/empty-state";
import { Textarea } from "@hone-financial/ui/textarea";
import { VList, type VListHandle } from "virtua/solid";
import {
  For,
  Match,
  Show,
  Switch,
  batch,
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
} from "solid-js";
import { useSessions } from "@/context/sessions";
import { parseMessageContent } from "@/lib/messages";
import { resolveSkillSlashCommand } from "@/lib/skill-command";
import type { PendingState, TimelineMessage } from "@/lib/types";
import { useSkills } from "@/context/skills";

type ChatRow =
  | {
      id: string;
      kind: "message";
      message: TimelineMessage;
    }
  | {
      id: string;
      kind: "pending";
      pending: PendingState;
    };

/** 格式化运行时长，如 "5s" / "1m 30s" */
function formatElapsed(ms: number): string {
  const total = Math.floor(ms / 1000);
  if (total < 60) return `${total}s`;
  const m = Math.floor(total / 60);
  const s = total % 60;
  return `${m}m ${s}s`;
}

/** 正在处理的消息气泡 —— 展示完整生命周期状态 */
function PendingBubble(props: {
  pending: PendingState;
  onDismiss: () => void;
  onStop: () => void;
}) {
  const [elapsed, setElapsed] = createSignal(Date.now() - props.pending.startedAt);

  const timer = setInterval(() => {
    setElapsed(Date.now() - props.pending.startedAt);
  }, 1000);
  onCleanup(() => clearInterval(timer));

  const isError = () => props.pending.phase === "error";
  const isTimeout = () => props.pending.phase === "timeout";
  const isTerminal = () => isError() || isTimeout();
  const isStreaming = () => props.pending.phase === "streaming";

  const phaseLabel = () => {
    switch (props.pending.phase) {
      case "queued":
        return "发送中";
      case "thinking":
        return "思考中";
      case "running":
        return "执行中";
      case "streaming":
        return "输出中";
      case "error":
        return "出错了";
      case "timeout":
        return "请求超时";
    }
  };

  const dotClass = () =>
    isError()
      ? "bg-rose-400"
      : isTimeout()
        ? "bg-amber-400"
        : "bg-[color:var(--accent)] animate-pulse";

  const bubbleClass = () =>
    isError()
      ? "border border-rose-400/30 bg-rose-500/10 text-rose-300"
      : isTimeout()
        ? "border border-amber-400/30 bg-amber-500/10 text-amber-300"
        : "bg-[color:var(--surface-strong)] text-[color:var(--text-primary)]";

  return (
    <div class="flex justify-start">
      <div
        class={[
          "max-w-[78%] rounded-2xl px-4 py-3 text-sm leading-7 shadow-sm",
          bubbleClass(),
        ].join(" ")}
      >
        {/* 头部：状态指示 + 耗时 + dismiss 按钮 */}
        <div class="mb-2 flex items-center justify-between gap-3">
          <div class="flex items-center gap-2">
            <span class={["h-2 w-2 rounded-full", dotClass()].join(" ")} />
            <span class="text-xs font-medium uppercase tracking-[0.15em] text-[color:var(--text-muted)]">
              {phaseLabel()}
            </span>
            <span class="text-xs text-[color:var(--text-muted)]">
              {formatElapsed(elapsed())}
            </span>
          </div>
          <Show
            when={!isTerminal()}
            fallback={
              <button
                type="button"
                onClick={props.onDismiss}
                class="text-xs text-[color:var(--text-muted)] hover:text-[color:var(--text-secondary)] transition-colors"
                title="关闭"
              >
                ✕
              </button>
            }
          >
            {/* 活跃阶段：显示内联停止按钮 */}
            <button
              type="button"
              onClick={props.onStop}
              class="inline-flex items-center gap-1 rounded px-1.5 py-0.5 text-xs text-[color:var(--text-muted)] transition-colors hover:bg-rose-500/10 hover:text-rose-400"
              title="停止"
            >
              <span class="h-1.5 w-1.5 rounded-sm bg-current" />
              停止
            </button>
          </Show>
        </div>

        {/* 状态文本（非流式时显示） */}
        <Show when={(!isStreaming() || isTerminal()) && props.pending.statusText && props.pending.statusText !== "..."}>
          <div class="whitespace-pre-wrap text-sm">
            {props.pending.statusText}
          </div>
        </Show>

        {/* 流式内容区域 */}
        <Show when={props.pending.partialContent}>
          <div class="whitespace-pre-wrap text-sm">
            {props.pending.partialContent}
            <Show when={isStreaming()}>
              {/* 光标闪烁效果 */}
              <span class="ml-0.5 inline-block h-[1em] w-[2px] animate-pulse bg-current align-middle opacity-70" />
            </Show>
          </div>
        </Show>

        {/* 工具调用时：非流式、非终态，显示动态点 */}
        <Show
          when={
            !isTerminal() &&
            !isStreaming() &&
            !props.pending.partialContent
          }
        >
          <div class="mt-1 flex gap-1">
            <span
              class="h-1.5 w-1.5 rounded-full bg-[color:var(--text-muted)] animate-bounce"
              style={{ "animation-delay": "0ms" }}
            />
            <span
              class="h-1.5 w-1.5 rounded-full bg-[color:var(--text-muted)] animate-bounce"
              style={{ "animation-delay": "150ms" }}
            />
            <span
              class="h-1.5 w-1.5 rounded-full bg-[color:var(--text-muted)] animate-bounce"
              style={{ "animation-delay": "300ms" }}
            />
          </div>
        </Show>
      </div>
    </div>
  );
}

function MessageBubble(props: { message: TimelineMessage }) {
  const scheduledLabel = () =>
    props.message.kind === "scheduled" ? props.message.jobName : undefined;

  const base =
    props.message.kind === "user"
      ? "ml-auto bg-[color:var(--accent)] text-white"
      : props.message.kind === "scheduled"
        ? "border border-[color:var(--accent)]/30 bg-[color:var(--accent-soft)] text-[color:var(--text-primary)]"
        : props.message.kind === "system"
          ? "mx-auto border border-[color:var(--border)] bg-black/5 text-[color:var(--text-secondary)]"
          : "bg-[color:var(--surface-strong)] text-[color:var(--text-primary)]";

  return (
    <div
      class={[
        "max-w-[78%] rounded-2xl px-4 py-3 text-sm leading-7 shadow-sm",
        base,
      ].join(" ")}
    >
      <Show when={props.message.kind === "scheduled"}>
        <div class="mb-2 text-xs uppercase tracking-[0.2em] text-[color:var(--accent)]">
          定时任务 {scheduledLabel() ? `· ${scheduledLabel()}` : ""}
        </div>
      </Show>
      <For each={parseMessageContent(props.message.content)}>
        {(part) => (
          <Switch>
            <Match when={part.type === "image"}>
              <img
                src={part.value}
                alt=""
                class="mt-2 max-w-full rounded-2xl"
              />
            </Match>
            <Match when={part.type === "text"}>
              <span class="whitespace-pre-wrap">{part.value}</span>
            </Match>
          </Switch>
        )}
      </For>
    </div>
  );
}

function ChatRowView(props: {
  row: ChatRow;
  onDismissPending: () => void;
  onStopPending: () => void;
}) {
  if (props.row.kind === "pending") {
    return (
      <PendingBubble
        pending={props.row.pending}
        onDismiss={props.onDismissPending}
        onStop={props.onStopPending}
      />
    );
  }

  return (
    <div
      class={[
        "flex",
        props.row.message.kind === "user"
          ? "justify-end"
          : props.row.message.kind === "system"
            ? "justify-center"
            : "justify-start",
      ].join(" ")}
    >
      <MessageBubble message={props.row.message} />
    </div>
  );
}

export function ChatView(props: { userId?: string }) {
  const sessions = useSessions();
  const skills = useSkills();
  const [list, setList] = createSignal<VListHandle>();
  const [pinned, setPinned] = createSignal(true);
  const [dismissedPopupDraft, setDismissedPopupDraft] = createSignal("");
  const currentSession = createMemo(() => sessions.currentSession());

  const currentPending = createMemo(() => {
    const key = sessions.state.currentUserId;
    return key ? sessions.state.pendingByKey[key] : undefined;
  });

  const rows = createMemo<ChatRow[]>(() => {
    const items: ChatRow[] = sessions.currentMessages().map((message) => ({
      id: message.id,
      kind: "message" as const,
      message,
    }));

    const pending = currentPending();
    if (pending) {
      // ── UI 防重：如果 pending 的内容已经完整出现在历史记录中，则隐藏气泡 ──
      // 这解决了 clearPending 在刷新竞态中可能存在的短暂延时显示的“幽灵气泡”问题
      const lastMessage = sessions.currentMessages().at(-1);
      const isDuplicate =
        lastMessage &&
        lastMessage.kind === "assistant" &&
        pending.partialContent &&
        lastMessage.content.trim() === pending.partialContent.trim();

      if (!isDuplicate) {
        items.push({
          id: `__pending__${pending.id}`,
          kind: "pending",
          pending,
        });
      }
    }

    return items;
  });

  const syncPinnedState = () => {
    const handle = list();
    if (!handle) {
      setPinned(true);
      return;
    }

    const distance =
      handle.scrollSize - (handle.scrollOffset + handle.viewportSize);
    setPinned(distance <= 24);
  };

  createEffect(() => {
    props.userId;
    setPinned(true);
  });

  createEffect(() => {
    rows().length;
    requestAnimationFrame(() => {
      if (rows().length === 0) return;
      if (!pinned()) return;
      const handle = list();
      if (!handle) return;
      handle.scrollToIndex(rows().length - 1, { align: "end" });
    });
  });

  const slashSkill = createMemo(() =>
    resolveSkillSlashCommand(skills.state.skills, sessions.state.draft),
  );
  const visibleSlashSkill = createMemo(() => {
    const command = slashSkill();
    if (!command) {
      return null;
    }
    if (sessions.state.draft === dismissedPopupDraft()) {
      return null;
    }
    return command;
  });

  const applySkillDraft = (value: string) => {
    batch(() => {
      setDismissedPopupDraft(value);
      sessions.setDraft(value);
    });
  };

  const submitDraft = async () => {
    const slash = slashSkill();
    if (slash?.command.stage === "command") {
      applySkillDraft("/skill ");
      return;
    }
    await sessions.sendCurrentMessage();
  };

  createEffect(() => {
    const draft = sessions.state.draft;
    if (draft !== dismissedPopupDraft()) {
      setDismissedPopupDraft("");
    }
  });

  const isSending = createMemo(() =>
    sessions.isActivePending(sessions.state.currentUserId),
  );

  /** 当前 pending 是否处于活跃（非终态）阶段，可以停止 */
  const isActivelyPending = createMemo(() => {
    const key = sessions.state.currentUserId;
    if (!key) return false;
    const p = sessions.state.pendingByKey[key];
    if (!p) return false;
    return p.phase !== "error" && p.phase !== "timeout";
  });

  const handleDismissPending = () => {
    sessions.dismissPending(sessions.state.currentUserId);
  };

  const handleStop = () => {
    sessions.stopPending(sessions.state.currentUserId);
  };

  return (
    <Show
      when={props.userId}
      fallback={
        <EmptyState
          title="从左侧打开一个会话"
          description="你可以查看共享会话历史，或者新建一个用户会话开始和 Hone 对话。"
        />
      }
    >
      <div class="flex h-full min-h-0 flex-col rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] shadow-sm">
        <div class="flex items-center justify-between border-b border-[color:var(--border)] px-4 py-3">
          <div>
            <div class="text-base font-semibold">
              {currentSession()?.session_label || props.userId}
            </div>
            <div class="mt-0.5 text-xs text-[color:var(--text-muted)]">
              {currentSession()?.session_kind === "group"
                ? "共享群 session 历史（当前为只读浏览）"
                : "实时流式响应与定时消息推送"}
            </div>
          </div>
          <div class="flex items-center gap-2 text-sm text-[color:var(--text-secondary)]">
            <Show
              when={isSending()}
              fallback={
                <>
            <span class="h-2.5 w-2.5 rounded-full bg-[color:var(--success)]" />
            在线
                </>
              }
            >
              <span class="h-2.5 w-2.5 rounded-full bg-[color:var(--accent)] animate-pulse" />
              <span class="text-[color:var(--accent)]">处理中</span>
            </Show>
          </div>
        </div>

        <div class="min-h-0 flex-1 px-4 py-4">
          <Show
            when={rows().length > 0}
            fallback={
              <EmptyState
                title="暂无历史消息"
                description="先发送一条消息，或者等待定时任务推送。"
              />
            }
          >
            <VList
              ref={setList}
              data={rows()}
              class="hf-scrollbar h-full overscroll-contain"
              style={{ height: "100%" }}
              item="div"
              bufferSize={400}
              onScroll={() => syncPinnedState()}
              onScrollEnd={() => syncPinnedState()}
            >
              {(row: ChatRow) => (
                <div class="py-2">
                  <ChatRowView
                    row={row}
                    onDismissPending={handleDismissPending}
                    onStopPending={handleStop}
                  />
                </div>
              )}
            </VList>
          </Show>
        </div>

        <div class="relative border-t border-[color:var(--border)] p-4">
          <div class="flex items-center gap-3">
            <div class="relative flex-1">
              <Show when={visibleSlashSkill()}>
                {(command) => (
                  <div class="absolute bottom-full left-0 right-0 z-20 mb-2 overflow-hidden rounded-2xl border border-[color:var(--border-strong)] bg-[color:var(--surface)] shadow-[0_18px_50px_rgba(15,23,42,0.14)]">
                    <div class="border-b border-[color:var(--border)] px-4 py-3 text-xs font-semibold uppercase tracking-[0.2em] text-[color:var(--accent)]">
                      Slash
                    </div>
                    <Show when={command().command.stage === "command"}>
                      <button
                        type="button"
                        onClick={() => applySkillDraft("/skill ")}
                        class="block w-full border-b border-[color:var(--border)] bg-[color:var(--accent-soft)] px-4 py-3 text-left transition hover:bg-[color:var(--accent-soft)]/80"
                      >
                        <div class="flex items-center justify-between gap-3">
                          <div class="font-medium text-[color:var(--text-primary)]">
                            /skill
                          </div>
                          <div class="text-xs text-[color:var(--text-muted)]">
                            搜索并触发技能
                          </div>
                        </div>
                      </button>
                    </Show>
                    <div class="max-h-72 overflow-y-auto">
                      <Show
                        when={command().matches.length > 0}
                        fallback={
                          <div class="px-4 py-3 text-sm text-[color:var(--text-secondary)]">
                            没有找到匹配技能，继续输入更准确的关键词。
                          </div>
                        }
                      >
                        <For each={command().matches}>
                          {(skill) => (
                            <button
                              type="button"
                              onClick={() =>
                                applySkillDraft(`/skill ${skill.id}`)
                              }
                              class="block w-full border-b border-[color:var(--border)] px-4 py-3 text-left transition last:border-b-0 hover:bg-black/5"
                            >
                              <div class="flex items-center justify-between gap-3">
                                <div class="font-medium text-[color:var(--text-primary)]">
                                  {skill.display_name}
                                </div>
                                <div class="text-xs text-[color:var(--text-muted)]">
                                  {skill.id}
                                </div>
                              </div>
                              <div class="mt-1 text-sm text-[color:var(--text-secondary)]">
                                {skill.description}
                              </div>
                              <Show when={skill.aliases.length > 0}>
                                <div class="mt-2 text-xs text-[color:var(--text-muted)]">
                                  aliases: {skill.aliases.join(", ")}
                                </div>
                              </Show>
                            </button>
                          )}
                        </For>
                      </Show>
                    </div>
                  </div>
                )}
              </Show>
              <Textarea
                rows={1}
                value={sessions.state.draft}
                onInput={(event) =>
                  sessions.setDraft(event.currentTarget.value)
                }
                onKeyDown={(event) => {
                  if (event.isComposing) return;
                  if (event.key === "Enter" && !event.shiftKey) {
                    event.preventDefault();
                    void submitDraft();
                  }
                }}
                placeholder='给 Hone 发消息，或输入 "/skill 关键词" 搜索并触发技能...'
              />
            </div>
            <Show
              when={isActivelyPending()}
              fallback={
                <Button
                  class="whitespace-nowrap shrink-0"
                  onClick={() => void submitDraft()}
                  disabled={
                    currentSession()?.session_kind === "group" ||
                    (slashSkill()?.command.stage === "search" &&
                      !slashSkill()?.command.query)
                  }
                >
                  发送
                </Button>
              }
            >
              {/* 使用 Button 组件保证与「发送」按钮高度/布局完全一致 */}
              <Button
                variant="outline"
                class="shrink-0 whitespace-nowrap border-rose-500/60 text-rose-400 hover:border-rose-400 hover:bg-rose-500/10 hover:text-rose-300"
                onClick={handleStop}
              >
                <span class="h-2 w-2 rounded-sm bg-current" />
                停止
              </Button>
            </Show>
          </div>
        </div>
      </div>
    </Show>
  );
}
