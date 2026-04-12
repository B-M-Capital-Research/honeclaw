import {
  createContext,
  createEffect,
  createSignal,
  onCleanup,
  untrack,
  useContext,
  type ParentProps,
} from "solid-js";
import { createStore, reconcile } from "solid-js/store";
import { connectEvents, getHistory, getUsers, sendChat } from "@/lib/api";
import { actorFromUser, type ActorRef } from "@/lib/actors";
import { filterUsers } from "@/lib/filters";
import { historyToTimeline, messageId } from "@/lib/messages";
import { parseSseChunks } from "@/lib/stream";
import type {
  PendingPhase,
  PendingState,
  TimelineMessage,
  UserInfo,
} from "@/lib/types";
import { useConsole } from "./console";
import { useBackend } from "./backend";

/** 所有渠道历史消息轮询间隔（毫秒）— 兜底保证 5s 内更新 */
const HISTORY_POLL_INTERVAL_MS = 5000;
/** 用户列表轮询间隔（毫秒）— 及时发现新会话 */
const USERS_POLL_INTERVAL_MS = 5000;
/** 请求超时时间（毫秒） */
const REQUEST_TIMEOUT_MS = 120_000;

/**
 * 默认对话用户：ME
 * channel=web / user_id=ME — 始终存在于会话列表顶部，无需已有历史记录也可发起对话。
 * session_id 遵循后端 ActorIdentity::session_id() 的生成规则：
 *   encode("web") + "__direct__" + encode("ME") = "Actor_web__direct__ME"
 */
export const ME_SESSION_ID = "Actor_web__direct__ME";

function isPendingPhaseActive(phase: PendingPhase | undefined): boolean {
  return (
    phase === "queued" ||
    phase === "thinking" ||
    phase === "running" ||
    phase === "streaming"
  );
}

const ME_SYNTHETIC_USER: UserInfo = {
  channel: "web",
  user_id: "ME",
  channel_scope: undefined,
  session_id: ME_SESSION_ID,
  session_kind: "direct",
  session_label: "ME",
  actor_user_id: "ME",
  last_message: "点击开始对话",
  last_role: "",
  last_time: "",
  message_count: 0,
};

type SessionsContextValue = ReturnType<typeof createSessionsState>;

const SessionsContext = createContext<SessionsContextValue>();

function createSessionsState() {
  const backend = useBackend();
  const consoleState = useConsole();
  const [state, setState] = createStore({
    users: [] as UserInfo[],
    loadingUsers: false,
    loadingHistory: false,
    /** 每个会话独立的消息处理状态，key = session_id */
    pendingByKey: {} as Record<string, PendingState>,
    currentUserId: consoleState.state.lastUserId ?? "",
    histories: {} as Record<string, TimelineMessage[]>,
    draft: "",
    pendingPrefill: "",
    error: "",
  });
  const [query, setQuery] = createSignal("");
  const [channelFilter, setChannelFilter] = createSignal("all");
  let eventSource: EventSource | undefined;
  let pollTimer: ReturnType<typeof setInterval> | undefined;
  let usersTimer: ReturnType<typeof setInterval> | undefined;
  let sseReconnectTimer: ReturnType<typeof setTimeout> | undefined;
  /** 响应式 store 外部的 AbortController 映射，key → 当前请求的控制器 */
  const abortControllers = new Map<string, AbortController>();
  /** 正在执行 refreshHistoryForKey 的 key 集合，用于防止并发刷新导致重复追加消息 */
  const refreshingKeys = new Set<string>();
  /** 非响应式状态，用于严格阻止同一个会话的并发发送请求 */
  const activeSendingKeys = new Set<string>();
  /** 记录已经处理过的 prefill，防止在快速跳转中被触发多次 */
  const processedPrefills = new Set<string>();
  /** 记录各会话历史轮询的连续失败次数，用于在持续失败时输出警告日志 */
  const pollFailureCount = new Map<string, number>();

  const findUser = (key: string) =>
    state.users.find((user) => user.session_id === key);

  const append = (key: string, message: TimelineMessage) => {
    setState("histories", key, (current = []) => {
      // 1. ID 查重：防止同一个前端生成 ID 的消息被重复追加（SSE 循环中多次调用）
      if (current.some((m) => m.id === message.id)) return current;

      // 2. 内容查重：防止后端刷新（refreshHistoryForKey）与前端乐观更新（sendDraft）由于 ID 不一致导致的重复
      const last = current[current.length - 1];
      if (
        last &&
        last.kind === message.kind &&
        last.content.trim() === message.content.trim()
      ) {
        return current;
      }

      return [...current, message];
    });
  };

  /** 设置某个会话的 pending 状态 */
  const setPending = (key: string, pending: PendingState) => {
    setState("pendingByKey", key, pending);
  };

  /** 清除某个会话的 pending 状态 */
  const clearPending = (key: string) => {
    setState("pendingByKey", (prev) => {
      const next = { ...prev };
      delete next[key];
      return next;
    });
  };

  /** 更新某个会话 pending 状态的部分字段 */
  const updatePending = (key: string, patch: Partial<PendingState>) => {
    setState("pendingByKey", key, (prev) =>
      prev ? { ...prev, ...patch } : prev,
    );
  };

  const sendDraft = async (actor: ActorRef, key: string, draft: string) => {
    // 1. 严格非响应式锁：阻止极短时间内的多次并发触发（如导航重复触发）
    if (activeSendingKeys.has(key)) return;
    activeSendingKeys.add(key);

    // 2. 响应式锁：阻止 UI 上的重复操作
    const existing = state.pendingByKey[key];
    if (existing && existing.phase !== "error" && existing.phase !== "timeout") {
      activeSendingKeys.delete(key);
      return;
    }

    const timeoutHandleRef = { current: undefined as ReturnType<typeof setTimeout> | undefined };
    const controller = new AbortController();

    try {
      append(key, { id: messageId(), kind: "user", content: draft });

      const pendingId = messageId();
      const startedAt = Date.now();

      setPending(key, {
        id: pendingId,
        startedAt,
        phase: "queued",
        statusText: "正在发送…",
        partialContent: "",
      });

      // AbortController 用于超时中止和手动停止
      abortControllers.set(key, controller);
      timeoutHandleRef.current = setTimeout(() => {
        controller.abort();
        updatePending(key, {
          phase: "timeout",
          statusText: `请求超时（已等待 ${Math.round(REQUEST_TIMEOUT_MS / 1000)} 秒）`,
        });
      }, REQUEST_TIMEOUT_MS);

      if (!backend.hasCapability("chat")) {
        throw new Error("当前 backend 不支持聊天能力");
      }
      const stream = await sendChat(actor, draft, controller.signal);
      const reader = stream.getReader();
      const decoder = new TextDecoder();
      let pending = "";

      outer: while (true) {
        const chunk = await reader.read();
        if (chunk.done) break;
        pending += decoder.decode(chunk.value, { stream: true });
        const parsed = parseSseChunks(pending);
        pending = parsed.pending;

        for (const event of parsed.events) {
          if (event.event === "run_started") {
            const text = event.data.text;
            updatePending(key, {
              phase: "thinking",
              statusText: text || "正在思考…",
            });
          }

          if (event.event === "tool_call") {
            const tool = event.data.tool;
            const reasoning = event.data.reasoning ?? event.data.text;
            updatePending(key, {
              phase: "running",
              statusText: tool
                ? `调用工具：${tool}`
                : reasoning
                  ? reasoning
                  : "处理中…",
            });
          }

          if (event.event === "assistant_delta") {
            const content = event.data.content ?? "";
            const currentPending = state.pendingByKey[key];
            if (currentPending) {
              updatePending(key, {
                phase: "streaming",
                statusText: "输出中…",
                partialContent: (currentPending.partialContent ?? "") + content,
              });
            }
          }

          if (event.event === "error") {
            const msg = event.data.text?.trim() || "请求失败";
            updatePending(key, {
              phase: "error",
              statusText: msg,
            });
            continue;
          }

          if (event.event === "run_error") {
            updatePending(key, {
              phase: "error",
              statusText: event.data.message ?? "发生未知错误",
            });
            continue;
          }

          if (event.event === "run_finished") {
            if (timeoutHandleRef.current) clearTimeout(timeoutHandleRef.current);
            const currentPending = state.pendingByKey[key];
            if (currentPending?.phase === "error") {
              break outer;
            }
            if (!currentPending) {
              break outer;
            }
            clearPending(key);
            if (currentPending.partialContent) {
              append(key, {
                id: currentPending.id,
                kind: "assistant",
                content: currentPending.partialContent,
              });
            } else if (event.data.success === false) {
              setPending(key, {
                id: currentPending.id,
                startedAt: currentPending.startedAt,
                phase: "error",
                statusText: "处理失败，请重试",
                partialContent: "",
              });
            }
            break outer;
          }

          if (event.event === "done") {
            if (timeoutHandleRef.current) clearTimeout(timeoutHandleRef.current);
            const cur = state.pendingByKey[key];
            if (
              cur &&
              cur.phase !== "error" &&
              cur.phase !== "timeout"
            ) {
              if (cur.partialContent) {
                append(key, {
                  id: cur.id,
                  kind: "assistant",
                  content: cur.partialContent,
                });
                clearPending(key);
              } else {
                updatePending(key, {
                  phase: "error",
                  statusText: "连接已断开，请重试",
                });
              }
            }
            break outer;
          }
        }
      }
    } catch (error) {
      if (timeoutHandleRef.current) clearTimeout(timeoutHandleRef.current);
      const currentPending = state.pendingByKey[key];
      // 只有在超时状态以外才覆盖（超时已经更新过状态）
      if (currentPending && currentPending.phase !== "timeout") {
        const isAbort = error instanceof Error && error.name === "AbortError";
        if (!isAbort) {
          updatePending(key, {
            phase: "error",
            statusText: error instanceof Error ? error.message : String(error),
          });
        }
      }
    } finally {
      if (timeoutHandleRef.current) clearTimeout(timeoutHandleRef.current);
      abortControllers.delete(key);

      // ── 防御性清理：若 pending 仍处于"活跃"阶段（流意外关闭，未收到 run_finished）──
      // 此情形包括网络断开、服务端崩溃等，防止气泡永久停留、阻塞后续发送
      const lingering = state.pendingByKey[key];
      if (
        lingering &&
        lingering.phase !== "error" &&
        lingering.phase !== "timeout"
      ) {
        if (lingering.partialContent) {
          // 有部分流式内容 → 提交为正式消息后清除
          append(key, {
            id: lingering.id,
            kind: "assistant",
            content: lingering.partialContent,
          });
          clearPending(key);
        } else {
          // 无内容 → 显示连接断开错误（用户可 dismiss）
          updatePending(key, {
            phase: "error",
            statusText: "连接已断开，请重试",
          });
        }
      }

      // ── 释放发送锁；error/timeout 终态保留 pending 供用户 dismiss ──
      const tail = state.pendingByKey[key];
      if (!tail || isPendingPhaseActive(tail.phase)) {
        clearPending(key);
      }
      activeSendingKeys.delete(key);

      // 2. 后续刷新改为异步，不阻塞主逻辑返回
      void (async () => {
        if (backend.hasCapability("history")) {
          await refreshHistoryForKey(key, false, /* force= */ true);
        }
        consoleState.markRead(key);
        await refreshUsers();
      })();
    }
  };

  const ensureHistory = async (key: string) => {
    if (!key) return;
    if (!backend.state.connected || !backend.hasCapability("history")) return;
    if (state.histories[key]) return;
    const user = findUser(key);
    if (!user) return;
    if (!backend.hasCapability("history")) return;
    setState("loadingHistory", true);
    try {
      const history = await getHistory(user.session_id);
      // reconcile 让 SolidJS 按 id 做精准 diff，只更新真正变化的消息节点
      setState("histories", key, reconcile(historyToTimeline(history), { key: "id" }));
      setState("error", "");
    } catch (error) {
      setState("error", error instanceof Error ? error.message : String(error));
    } finally {
      setState("loadingHistory", false);
    }
  };

  /**
   * 刷新指定 key 的历史消息（用于 iMessage 实时更新）。
   * - 只在非 pending 状态下刷新，避免干扰 web console 的消息流
   * - 与现有 timeline 对比，有新消息才更新
   * - 若只是末尾追加新消息，则只 push 增量而不整体替换，
   *   避免 <For> 重新挂载已有消息气泡导致卡顿
   * - 自动推断思考状态（最后一条是 user → 显示 thinking pending）
   * @param force 若为 true，跳过 pending 检查（用于 send 完成后强制同步）
   */
  const refreshHistoryForKey = async (
    key: string,
    updatePendingState = false,
    force = false,
  ) => {
    // 防止对同一 key 的并发刷新（否则两次刷新会各自计算增量并重复追加相同消息）
    if (refreshingKeys.has(key)) return;
    // 若当前会话正在流式请求中，不干扰（除非强制刷新）；error/timeout 允许刷新
    if (!force && isPendingPhaseActive(state.pendingByKey[key]?.phase)) return;
    refreshingKeys.add(key);
    const user = findUser(key);
    if (!user) {
      refreshingKeys.delete(key);
      return;
    }
    if (!backend.hasCapability("history")) {
      refreshingKeys.delete(key);
      return;
    }
    try {
      const history = await getHistory(user.session_id);
      const newTimeline = historyToTimeline(history);
      const current = state.histories[key] ?? [];

      // 只在内容有变化时才更新（避免不必要的重渲染）
      const lastNew = newTimeline[newTimeline.length - 1];
      const lastCurrent = current[current.length - 1];
      const hasChanges =
        newTimeline.length !== current.length ||
        (lastNew !== undefined &&
          lastNew.content !== (lastCurrent?.content ?? ""));

      if (hasChanges) {
        // 若服务端消息数 >= 本地且前缀内容一致，只追加增量（最快路径）
        if (
          newTimeline.length > current.length &&
          current.length > 0 &&
          newTimeline[current.length - 1]?.content === lastCurrent?.content
        ) {
          const added = newTimeline.slice(current.length);
          setState("histories", key, (prev = []) => [...prev, ...added]);
        } else {
          // 全量替换时使用 reconcile 做精准 diff：
          // 配合 stableHistoryId，相同内容的消息 ID 不变，SolidJS 只更新真正变化的节点，
          // 避免 VList 触发全量重渲染导致的闪烁
          setState("histories", key, reconcile(newTimeline, { key: "id" }));
        }
      }

      // iMessage：根据最后一条消息推断 pending 状态
      if (updatePendingState && !state.pendingByKey[key]) {
        const last = newTimeline[newTimeline.length - 1];
        if (last?.kind === "user") {
          // 最后是用户消息，说明 bot 还没回复，显示 thinking
          setPending(key, {
            id: messageId(),
            startedAt: Date.now(),
            phase: "thinking",
            statusText: "正在思考…",
            partialContent: "",
          });
        } else if (last?.kind === "assistant") {
          // 已有回复，清除 pending
          clearPending(key);
        }
      }
      // 轮询成功，重置失败计数
      pollFailureCount.delete(key);
    } catch {
      // 累计连续失败次数；连续 3 次后输出警告，但不中断轮询（瞬时网络抖动可自愈）
      const failures = (pollFailureCount.get(key) ?? 0) + 1;
      pollFailureCount.set(key, failures);
      if (failures >= 3) {
        console.warn(
          `[sessions] 历史轮询连续失败 ${failures} 次 (key=${key})，后端可能暂时不可达`,
        );
      }
    } finally {
      refreshingKeys.delete(key);
    }
  };

  const reconnect = (key: string) => {
    // 清理旧的 SSE 连接、轮询定时器与待重连定时器
    if (eventSource) {
      eventSource.close();
      eventSource = undefined;
    }
    if (pollTimer) {
      clearInterval(pollTimer);
      pollTimer = undefined;
    }
    if (sseReconnectTimer) {
      clearTimeout(sseReconnectTimer);
      sseReconnectTimer = undefined;
    }
    if (!key) return;
    // untrack: 阻止 state.users 被当前 createEffect 追踪为依赖，
    // 避免每次 refreshUsers() 后都重建 SSE 连接导致连接池耗尽
    const user = untrack(() => findUser(key));
    if (!user) return;
    if (user.session_kind === "group") return;
    if (!backend.hasCapability("sse.events")) return;
    void connectEvents(actorFromUser(user)).then((source) => {
      eventSource = source;

      // ── SSE 断线自动重连 ──────────────────────────────────────────────────
      source.onerror = () => {
        source.close();
        eventSource = undefined;
        if (pollTimer) {
          clearInterval(pollTimer);
          pollTimer = undefined;
        }
        // 3 秒后重连（避免断线瞬间暴风重试）
        sseReconnectTimer = setTimeout(() => {
          sseReconnectTimer = undefined;
          const currentKey = untrack(() => state.currentUserId);
          if (currentKey && backend.state.connected) {
            reconnect(currentKey);
          }
        }, 3000);
      };

      // ── 定时任务消息 ──────────────────────────────────────────────────────────
      source.addEventListener("scheduled_message", (event) => {
        const data = JSON.parse(event.data || "{}") as {
          text?: string;
          job_name?: string;
        };
        append(key, {
          id: messageId(),
          kind: "scheduled",
          content: data.text ?? "",
          jobName: data.job_name,
        });
        void refreshUsers();
      });

      // ── 主动推送消息 ──────────────────────────────────────────────────────────
      source.addEventListener("push_message", (event) => {
        const data = JSON.parse(event.data || "{}") as { text?: string };
        append(key, {
          id: messageId(),
          kind: "assistant",
          content: data.text ?? "",
        });
        void refreshUsers();
      });

      // ── iMessage 实时事件（仅在 iMessage 渠道下生效，防止污染 web 消息流） ──
      const isImessage = user.channel === "imessage";

      source.addEventListener("imessage_user_message", () => {
        if (!isImessage) return;
        void refreshHistoryForKey(key, false);
        if (!state.pendingByKey[key]) {
          setPending(key, {
            id: messageId(),
            startedAt: Date.now(),
            phase: "thinking",
            statusText: "正在思考…",
            partialContent: "",
          });
        }
      });

      source.addEventListener("imessage_progress", (event) => {
        if (!isImessage) return;
        const data = JSON.parse(event.data || "{}") as {
          stage?: string;
          tool?: string;
          iteration?: number;
        };
        const text = data.tool
          ? `调用工具：${data.tool}`
          : data.stage === "gemini.spawn"
            ? `Gemini 正在思考… (轮次 ${data.iteration ?? 1})`
            : `处理中 (${data.stage ?? ""})`;
        // 更新 pending 状态而不是 append 系统消息（减少气泡噪音）
        if (state.pendingByKey[key]) {
          updatePending(key, { phase: "running", statusText: text });
        } else {
          setPending(key, {
            id: messageId(),
            startedAt: Date.now(),
            phase: "running",
            statusText: text,
            partialContent: "",
          });
        }
      });

      source.addEventListener("imessage_processing_start", () => {
        if (!isImessage) return;
        if (!state.pendingByKey[key]) {
          setPending(key, {
            id: messageId(),
            startedAt: Date.now(),
            phase: "thinking",
            statusText: "正在处理…",
            partialContent: "",
          });
        }
      });

      source.addEventListener("imessage_assistant_message", () => {
        if (!isImessage) return;
        void refreshHistoryForKey(key, false);
        clearPending(key);
      });

      source.addEventListener("imessage_processing_error", () => {
        if (!isImessage) return;
        clearPending(key);
      });
    });

    // ── 所有渠道历史轮询兜底（5s，保证消息在 5s 内同步）──────────────────────
    if (backend.hasCapability("history")) {
      const isImessage = user.channel === "imessage";
      pollTimer = setInterval(() => {
        void refreshHistoryForKey(key, isImessage);
      }, HISTORY_POLL_INTERVAL_MS);
    }
  };

  const refreshUsers = async () => {
    if (!backend.state.connected || !backend.hasCapability("users")) {
      setState("users", []);
      return;
    }
    // 只在首次加载（列表为空）时显示 loading 状态；
    // 后续背景轮询静默更新，不触发骨架屏闪烁
    if (state.users.length === 0) {
      setState("loadingUsers", true);
    }
    try {
      const users = await getUsers();
      // 若后端尚未存在 ME 的真实 session，始终在列表顶部注入合成 ME 用户，
      // 确保用户无需先有历史记录也能随时发起对话。
      // 一旦 ME 发送过消息并有了真实 session 文件，后端会返回真实数据将其覆盖。
      const hasME = users.some((u) => u.session_id === ME_SESSION_ID);
      const usersWithMe = hasME ? users : [ME_SYNTHETIC_USER, ...users];
      setState("users", reconcile(usersWithMe, { key: "session_id" }));
      setState("error", "");
    } catch (error) {
      setState("error", error instanceof Error ? error.message : String(error));
    } finally {
      setState("loadingUsers", false);
    }
  };

  const selectUser = async (key?: string) => {
    setState("currentUserId", key ?? "");
    consoleState.setLastUserId(key);
    if (!key) return;
    consoleState.markRead(key);
    // 若有从开始页带来的预填消息，直接自动发送而不是仅写入 draft
    if (state.pendingPrefill) {
      const prefill = state.pendingPrefill;
      const prefillKey = `${key}:${prefill}`;

      if (!processedPrefills.has(prefillKey)) {
        processedPrefills.add(prefillKey);
        setState("pendingPrefill", "");
        setState("draft", "");
        const user = findUser(key);
        if (user && user.session_kind !== "group") {
          const existing = state.pendingByKey[key];
          if (!existing || existing.phase === "error" || existing.phase === "timeout") {
            void sendDraft(actorFromUser(user), key, prefill);
            return;
          }
        }
        // 群会话或找不到用户时，回退到只填入草稿
        setState("draft", prefill);
      }
    }
    await ensureHistory(key);
  };

  createEffect(() => {
    const userId = state.currentUserId;
    if (backend.state.connected) {
      reconnect(userId);
    }
  });

  createEffect(() => {
    if (backend.state.connected) {
      void refreshUsers();
      // 清除旧定时器（effect 重跑时）
      if (usersTimer) clearInterval(usersTimer);
      // 每 5s 轮询用户列表，及时发现新会话
      usersTimer = setInterval(() => {
        void refreshUsers();
      }, USERS_POLL_INTERVAL_MS);
      onCleanup(() => {
        if (usersTimer) {
          clearInterval(usersTimer);
          usersTimer = undefined;
        }
      });
    }
  });

  onCleanup(() => {
    eventSource?.close();
    if (pollTimer) clearInterval(pollTimer);
    if (usersTimer) clearInterval(usersTimer);
    if (sseReconnectTimer) clearTimeout(sseReconnectTimer);
  });

  return {
    state,
    query,
    setQuery,
    channelFilter,
    setChannelFilter,
    async refreshUsers() {
      await refreshUsers();
    },
    filteredUsers() {
      return filterUsers(state.users, query(), channelFilter());
    },
    availableChannels() {
      return Array.from(
        new Set(state.users.map((user) => user.channel || "direct")),
      );
    },
    async selectUser(key?: string) {
      await selectUser(key);
    },
    currentMessages() {
      return state.currentUserId
        ? (state.histories[state.currentUserId] ?? [])
        : [];
    },
    /** 是否存在进行中的流式请求（不含 error/timeout）— 标题栏「处理中」等 */
    isActivePending(key: string) {
      return isPendingPhaseActive(state.pendingByKey[key]?.phase);
    },
    /** @deprecated 请用 isActivePending，语义已改为仅「活跃」阶段 */
    isPending(key: string) {
      return isPendingPhaseActive(state.pendingByKey[key]?.phase);
    },
    /** 关闭指定会话的错误/超时 pending 状态（用户主动 dismiss） */
    dismissPending(key: string) {
      const p = state.pendingByKey[key];
      if (p && (p.phase === "error" || p.phase === "timeout")) {
        clearPending(key);
      }
    },
    setDraft(value: string) {
      setState("draft", value);
    },
    prefillDraft(value: string) {
      if (state.currentUserId) {
        setState("draft", value);
        return;
      }
      setState("pendingPrefill", value);
    },
    /** 从开始页跳转到 ME 会话时使用：清空当前 draft，写入 pendingPrefill，
     *  待 selectUser(ME_SESSION_ID) 被调用时自动提取为 draft 并发送 */
    setPendingPrefill(text: string) {
      setState("draft", "");
      setState("pendingPrefill", text);
    },
    async sendCurrentMessage() {
      const key = state.currentUserId;
      const draft = state.draft.trim();
      const user = key ? findUser(key) : undefined;
      if (!key || !user || !draft) return;
      // 若处于可重试的终态（error / timeout），自动 dismiss 以允许立即重发
      const existing = state.pendingByKey[key];
      if (existing && (existing.phase === "error" || existing.phase === "timeout")) {
        clearPending(key);
      }
      if (state.pendingByKey[key]) return; // 仍在活跃处理中，阻止重复发送
      if (user.session_kind === "group") {
        append(key, {
          id: messageId(),
          kind: "system",
          content: "群共享会话当前仅支持浏览历史；请在对应 IM 群里继续发言。",
        });
        return;
      }

      setState("draft", "");
      await sendDraft(actorFromUser(user), key, draft);
    },
    async sendPrefilledMessage(message: string) {
      const key = state.currentUserId;
      const draft = message.trim();
      const user = key ? findUser(key) : undefined;
      if (!key || !user || !draft) return;
      // 若处于可重试的终态（error / timeout），自动 dismiss 以允许立即重发
      const existing = state.pendingByKey[key];
      if (existing && (existing.phase === "error" || existing.phase === "timeout")) {
        clearPending(key);
      }
      if (state.pendingByKey[key]) return;
      if (user.session_kind === "group") {
        append(key, {
          id: messageId(),
          kind: "system",
          content: "群共享会话当前仅支持浏览历史；请在对应 IM 群里继续发言。",
        });
        return;
      }

      setState("draft", "");
      await sendDraft(actorFromUser(user), key, draft);
    },
    /** 主动停止当前进行中的请求（中止 HTTP 流、设置"已停止"状态） */
    stopPending(key: string) {
      const controller = abortControllers.get(key);
      if (controller) {
        controller.abort();
        // abort 触发 catch 块，catch 块会将状态设为 error 并显示 AbortError；
        // 覆盖为更友好的"已停止"提示
        setTimeout(() => {
          const p = state.pendingByKey[key];
          if (p && (p.phase === "error" || p.phase === "timeout")) {
            updatePending(key, { phase: "error", statusText: "已停止，可重新发送" });
          }
        }, 50);
      } else {
        // 若无活跃请求（如 iMessage pending），直接清除
        clearPending(key);
      }
    },
    currentSession() {
      return state.currentUserId ? findUser(state.currentUserId) : undefined;
    },
  };
}

export function SessionsProvider(props: ParentProps) {
  const value = createSessionsState();
  return (
    <SessionsContext.Provider value={value}>
      {props.children}
    </SessionsContext.Provider>
  );
}

export function useSessions() {
  const value = useContext(SessionsContext);
  if (!value) {
    throw new Error("SessionsProvider missing");
  }
  return value;
}
