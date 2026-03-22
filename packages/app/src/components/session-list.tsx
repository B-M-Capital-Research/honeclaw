import { EmptyState } from "@hone-financial/ui/empty-state";
import { Input } from "@hone-financial/ui/input";
import { Skeleton } from "@hone-financial/ui/skeleton";
import { useNavigate } from "@solidjs/router";
import { For, Show, createMemo } from "solid-js";
import { hasUnread } from "@/lib/filters";
import { actorFromUser, actorLabel } from "@/lib/actors";
import { useConsole } from "@/context/console";
import { useSessions, ME_SESSION_ID } from "@/context/sessions";

function relativeTime(value: string) {
  if (!value) return "刚刚";
  const diff = (Date.now() - new Date(value).getTime()) / 1000;
  if (diff < 60) return "刚刚";
  if (diff < 3600) return `${Math.floor(diff / 60)} 分钟前`;
  if (diff < 86400) return `${Math.floor(diff / 3600)} 小时前`;
  if (diff < 604800) return `${Math.floor(diff / 86400)} 天前`;
  const date = new Date(value);
  return `${date.getMonth() + 1}/${date.getDate()}`;
}

export function SessionList() {
  const navigate = useNavigate();
  const consoleState = useConsole();
  const sessions = useSessions();

  const openUser = async (key: string) => {
    await sessions.selectUser(key);
    navigate(`/sessions/${encodeURIComponent(key)}`);
  };

  const channelLabel = (value: string) => {
    switch (value) {
      case "direct":
        return "iMessage / Web";
      case "imessage":
        return "iMessage";
      case "web":
        return "Web";
      case "discord":
        return "Discord";
      case "telegram":
        return "Telegram";
      case "cli":
        return "CLI";
      default:
        return value || "未知";
    }
  };

  const channelOptions = createMemo(() => {
    const order = ["direct", "imessage", "web", "discord", "telegram", "cli"];
    return sessions
      .availableChannels()
      .slice()
      .sort((left, right) => {
        const leftIndex = order.indexOf(left);
        const rightIndex = order.indexOf(right);
        if (leftIndex === -1 && rightIndex === -1)
          return left.localeCompare(right);
        if (leftIndex === -1) return 1;
        if (rightIndex === -1) return -1;
        return leftIndex - rightIndex;
      });
  });

  return (
    <div class="flex h-full min-h-0 w-[320px] flex-col border-r border-[color:var(--border)] bg-[color:var(--surface)]">
      <div class="border-b border-[color:var(--border)] px-4 py-3">
        <div>
          <div class="text-sm font-semibold tracking-tight">会话</div>
          <div class="mt-1 text-xs text-[color:var(--text-muted)]">
            按渠道查看 session，并打开对应历史
          </div>
        </div>
        {/* 渠道隔离提示 */}
        <div class="mt-2 flex items-center gap-1.5 rounded-md bg-amber-400/10 px-2.5 py-1.5">
          <span class="shrink-0 text-amber-400">ℹ</span>
          <span class="text-[11px] leading-relaxed text-amber-300/80">
            不同渠道的用户 ID 相互独立，无法共享上下文。
          </span>
        </div>
        <div class="mt-3 flex flex-wrap gap-2">
          <For each={consoleState.channels() ?? []}>
            {(channel) => (
              <span class="inline-flex items-center gap-1 rounded-full border border-[color:var(--border)] bg-[color:var(--panel)] px-2.5 py-1 text-[11px] text-[color:var(--text-secondary)]">
                <span
                  class={[
                    "h-1.5 w-1.5 rounded-full",
                    channel.running
                      ? "bg-[color:var(--success)]"
                      : "bg-black/20",
                  ].join(" ")}
                />
                <span>{channel.label}</span>
              </span>
            )}
          </For>
        </div>
        <Input
          class="mt-3 h-8 text-xs"
          value={sessions.query()}
          onInput={(event) => sessions.setQuery(event.currentTarget.value)}
          placeholder="搜索用户名"
        />
        <select
          class="mt-2 flex h-8 w-full rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-1 text-xs text-[color:var(--text-primary)] focus:outline-none focus:ring-2 focus:ring-[color:var(--accent)]"
          value={sessions.channelFilter()}
          onChange={(event) =>
            sessions.setChannelFilter(event.currentTarget.value)
          }
        >
          <option value="all">全部渠道</option>
          <For each={channelOptions()}>
            {(channel) => (
              <option value={channel}>{channelLabel(channel)}</option>
            )}
          </For>
        </select>
      </div>

      <div class="hf-scrollbar min-h-0 flex-1 overflow-y-auto px-3 py-3">
        <Show
          when={!sessions.state.loadingUsers}
          fallback={
            <div class="space-y-3 px-2 py-2">
              <Skeleton class="h-20" />
              <Skeleton class="h-20" />
              <Skeleton class="h-20" />
            </div>
          }
        >
          <Show
            when={sessions.filteredUsers().length > 0}
            fallback={
              <EmptyState
                title="还没有会话"
                description="打开一个用户会话后，历史记录会出现在这里。"
              />
            }
          >
            <div class="space-y-2">
              <For each={sessions.filteredUsers()}>
                {(user) => {
                  const key = user.session_id;
                  const active = () => sessions.state.currentUserId === key;
                  const unread = () =>
                    hasUnread(
                      key,
                      user.last_time,
                      user.last_role,
                      consoleState.state.readAt,
                      sessions.state.currentUserId,
                    );

                  const isMe = () => key === ME_SESSION_ID;

                  return (
                    <button
                      type="button"
                      onClick={() => void openUser(key)}
                      class={[
                        "w-full rounded-md border p-3 text-left transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[color:var(--accent)]",
                        active()
                          ? "border-[color:var(--accent)] bg-[color:var(--accent-soft)]"
                          : isMe()
                            ? "border-[color:var(--accent)]/30 bg-[color:var(--accent-soft)]/30 hover:border-[color:var(--accent)]/60 hover:bg-[color:var(--accent-soft)]/50"
                            : "border-transparent bg-transparent hover:border-[color:var(--border-strong)] hover:bg-black/5",
                      ].join(" ")}
                    >
                      <div class="flex items-start gap-3">
                        {/* ME 用户用特殊图标，其余用首字母 */}
                        <div
                          class={[
                            "flex h-8 w-8 shrink-0 items-center justify-center rounded-md text-xs font-semibold",
                            isMe()
                              ? "bg-[color:var(--accent)] text-white"
                              : "bg-[color:var(--panel-strong)] text-[color:var(--text-secondary)]",
                          ].join(" ")}
                        >
                          {isMe() ? "✦" : user.user_id.slice(0, 1).toUpperCase()}
                        </div>
                        <div class="min-w-0 flex-1">
                          <div class="flex items-center justify-between gap-2">
                            <div class="flex items-center gap-1.5 truncate">
                              <span class="truncate text-sm font-medium text-[color:var(--text-primary)]">
                                {user.session_kind === "group"
                                  ? user.session_label
                                  : actorLabel(actorFromUser(user))}
                              </span>
                              <Show when={isMe()}>
                                <span class="shrink-0 rounded-full bg-[color:var(--accent)] px-1.5 py-0.5 text-[9px] font-semibold uppercase tracking-wide text-white">
                                  默认
                                </span>
                              </Show>
                            </div>
                            <div class="text-[11px] text-[color:var(--text-muted)]">
                              <Show when={!isMe() || user.last_role !== ""}>
                                {relativeTime(user.last_time)}
                              </Show>
                            </div>
                          </div>
                          <div class="mt-0.5 line-clamp-1 text-xs leading-5 text-[color:var(--text-secondary)]">
                            {user.last_message || "暂无消息"}
                          </div>
                          <div class="mt-2 flex items-center justify-between gap-2">
                            <div class="flex items-center gap-2">
                              <span class="rounded-full bg-[color:var(--panel)] px-2 py-0.5 text-[10px] uppercase tracking-wide text-[color:var(--text-secondary)]">
                                {channelLabel(user.channel || "direct")}
                              </span>
                              <Show when={user.session_kind === "group"}>
                                <span class="rounded-full bg-[color:var(--accent-soft)] px-2 py-0.5 text-[10px] uppercase tracking-wide text-[color:var(--accent)]">
                                  群共享
                                </span>
                              </Show>
                              <Show when={!isMe() || user.message_count > 0}>
                                <span class="text-[11px] text-[color:var(--text-muted)]">
                                  {user.message_count} 条历史记录
                                </span>
                              </Show>
                            </div>
                            <Show when={unread()}>
                              <span class="h-2 w-2 rounded-full bg-[color:var(--danger)]" />
                            </Show>
                          </div>
                        </div>
                      </div>
                    </button>
                  );
                }}
              </For>
            </div>
          </Show>
        </Show>
      </div>
    </div>
  );
}
