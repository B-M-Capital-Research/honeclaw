import { Title } from "@solidjs/meta";
import { For, Show, createMemo, createSignal, onCleanup, onMount } from "solid-js";

import { PublicLoginForm } from "@/components/public-login-form";
import { PublicWorkspaceShell } from "@/components/public-workspace-shell";
import { ApiError, getPublicAuthMe, getPublicIndustryMap } from "@/lib/api";
import { cachedPublicUser, setCachedPublicUser } from "@/lib/public-session-cache";
import type { Industry, IndustryMapSnapshot, PublicAuthUserInfo } from "@/lib/types";

import "./public-foundation.css";
import "./public-site.css";
import "./public-polish.css";
import "./public-industry-map.css";

type ViewState = "loading" | "ready" | "login" | "forbidden" | "error";

/** 市值只用于排序与规模感，给到两位有效小数就够，不做币种换算（树里全是美元计价的美股）。 */
function marketCap(value: number | undefined) {
  if (value == null || !Number.isFinite(value)) return "—";
  if (value >= 1e12) return `${(value / 1e12).toFixed(2)} 万亿`;
  if (value >= 1e8) return `${(value / 1e8).toFixed(0)} 亿`;
  return `${(value / 1e8).toFixed(2)} 亿`;
}

/** 改动时间只给到分钟：卡片要的是「什么时候改的」，不是精确时刻。 */
function editedAt(value: string | undefined) {
  if (!value) return "";
  const at = new Date(value);
  if (Number.isNaN(at.getTime())) return value;
  return at.toLocaleString("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function changePercent(value: number | undefined) {
  if (value == null || !Number.isFinite(value)) return "";
  return `${value >= 0 ? "+" : ""}${value.toFixed(2)}%`;
}

export default function PublicIndustryMapPage() {
  const [user, setUser] = createSignal<PublicAuthUserInfo | null>(cachedPublicUser());
  const [view, setView] = createSignal<ViewState>(
    cachedPublicUser()?.is_admin ? "loading" : cachedPublicUser() ? "forbidden" : "loading",
  );
  const [snapshot, setSnapshot] = createSignal<IndustryMapSnapshot>();
  const [selected, setSelected] = createSignal<string>();
  const [error, setError] = createSignal("");
  let controller: AbortController | undefined;

  const bootstrap = async () => {
    try {
      const me = await getPublicAuthMe();
      setUser(me);
      setCachedPublicUser(me);
      if (!me.is_admin) {
        setView("forbidden");
        return;
      }
    } catch (cause) {
      if (cause instanceof Error && cause.name === "AbortError") return;
      setUser(null);
      setCachedPublicUser(null);
      setView("login");
      return;
    }
    await load();
  };

  const load = async () => {
    if (user()?.is_admin !== true) {
      setView("forbidden");
      return;
    }
    controller?.abort();
    controller = new AbortController();
    setError("");
    try {
      const data = await getPublicIndustryMap(controller.signal);
      setSnapshot(data);
      setSelected((current) => current ?? data.industries[0]?.id);
      setView("ready");
    } catch (cause) {
      if (cause instanceof Error && cause.name === "AbortError") return;
      if (cause instanceof ApiError && cause.status === 401) setView("login");
      else if (cause instanceof ApiError && cause.status === 403) setView("forbidden");
      else {
        setError(cause instanceof Error ? cause.message : String(cause));
        setView("error");
      }
    }
  };

  onMount(() => void bootstrap());
  onCleanup(() => controller?.abort());

  const current = createMemo<Industry | undefined>(() =>
    snapshot()?.industries.find((item) => item.id === selected()),
  );

  return (
    <>
      <Title>行业分析 · HONE</Title>
      <Show
        when={view() !== "loading"}
        fallback={<div class="industry-map-loading" role="status">正在读取行业树…</div>}
      >
        <Show
          when={view() !== "login"}
          fallback={
            <PublicLoginForm
              title="登录后查看行业分析"
              subtitle="行业树是研究结构与先验，不是当前事实或买卖建议。"
              onLogin={() => void bootstrap()}
            />
          }
        >
          <PublicWorkspaceShell active="research" topbarLabel="行业分析">
            <Show
              when={view() !== "forbidden"}
              fallback={<p class="industry-map-empty">行业分析目前仅对管理员开放。</p>}
            >
              <Show
                when={view() !== "error"}
                fallback={<p class="industry-map-empty">读取失败：{error()}</p>}
              >
                <Show when={snapshot()}>
                  {(data) => (
                    <div class="industry-map">
                      <header class="industry-map-head">
                        <h1>{data().root.name}</h1>
                        <p>{data().root.summary}</p>
                        <p class="industry-map-meta">
                          研究底稿更新：{data().generated_at}
                          <Show when={!data().market_data_available}>
                            <span class="industry-map-warn">
                              本次未取到行情，公司暂按维护顺序排列
                            </span>
                          </Show>
                        </p>
                      </header>

                      <Show when={data().recent_edits.length > 0}>
                        <section class="industry-edits" aria-label="最近改动">
                          <h2>
                            最近改动
                            <span class="industry-edits-count">
                              共 {data().edit_count} 次
                            </span>
                          </h2>
                          <p class="industry-edits-note">
                            管理员在对话里改的行业内容会记在这里，研究台与后续对话的行业注入同时生效。
                          </p>
                          <ul>
                            <For each={data().recent_edits}>
                              {(edit) => (
                                <li>
                                  <button
                                    type="button"
                                    class="industry-edits-jump"
                                    onClick={() => setSelected(edit.industry)}
                                  >
                                    {edit.industry_name}
                                  </button>
                                  <span class="industry-edits-summary">{edit.summary}</span>
                                  <span class="industry-edits-meta">
                                    <span>{editedAt(edit.at)}</span>
                                    <span class="industry-edits-by">{edit.by}</span>
                                  </span>
                                  <Show when={edit.note}>
                                    <p class="industry-edits-why">{edit.note}</p>
                                  </Show>
                                </li>
                              )}
                            </For>
                          </ul>
                        </section>
                      </Show>

                      <div class="industry-map-body">
                        <nav class="industry-tree" aria-label="行业树">
                          <div class="industry-tree-root">{data().root.name}</div>
                          <ul>
                            <For each={data().industries}>
                              {(industry) => (
                                <li>
                                  <button
                                    type="button"
                                    class="industry-tree-node"
                                    classList={{ "is-active": industry.id === selected() }}
                                    aria-current={industry.id === selected() ? "true" : undefined}
                                    onClick={() => setSelected(industry.id)}
                                  >
                                    <span class="industry-tree-name">
                                      {industry.name}
                                      <Show when={industry.last_edited_at}>
                                        <span class="industry-tree-dot" title="有管理员改动" />
                                      </Show>
                                    </span>
                                    <span class="industry-tree-count">
                                      {industry.members.length}
                                    </span>
                                  </button>
                                </li>
                              )}
                            </For>
                          </ul>
                        </nav>

                        <Show
                          when={current()}
                          fallback={<p class="industry-map-empty">选择左侧的一个行业。</p>}
                        >
                          {(industry) => (
                            <section class="industry-detail">
                              <h2>
                                {industry().name}
                                <Show when={industry().last_edited_at}>
                                  <span class="industry-detail-edited">
                                    最近改动 {editedAt(industry().last_edited_at)}
                                  </span>
                                </Show>
                              </h2>
                              <p class="industry-detail-lead">{industry().one_liner}</p>

                              <h3>相关公司</h3>
                              <p class="industry-detail-note">按市值降序；本轮未取到行情的排在最后。树里只收美股与 ADR。</p>
                              <table class="industry-members">
                                <thead>
                                  <tr>
                                    <th>代码</th>
                                    <th>公司</th>
                                    <th>市值（美元）</th>
                                    <th>现价</th>
                                    <th>在这一行的位置</th>
                                  </tr>
                                </thead>
                                <tbody>
                                  <For each={industry().members}>
                                    {(member) => (
                                      <tr>
                                        <td class="industry-symbol">{member.symbol}</td>
                                        <td>{member.name}</td>
                                        <td>{marketCap(member.market_cap)}</td>
                                        <td>
                                          <Show when={member.price != null} fallback="—">
                                            {member.price?.toFixed(2)}
                                            <span
                                              class="industry-change"
                                              classList={{ "is-down": (member.change_percent ?? 0) < 0 }}
                                            >
                                              {changePercent(member.change_percent)}
                                            </span>
                                          </Show>
                                        </td>
                                        <td class="industry-role">{member.role}</td>
                                      </tr>
                                    )}
                                  </For>
                                </tbody>
                              </table>

                              <h3>底层估值逻辑（结合 AI）</h3>
                              <Show
                                when={industry().ai_valuation_logic.driver_chain}
                                fallback={<p class="industry-detail-note">这一行的传导链尚未定稿。</p>}
                              >
                                <p class="industry-chain">
                                  {industry().ai_valuation_logic.driver_chain}
                                </p>
                                <Show when={industry().ai_valuation_logic.key_variables.length > 0}>
                                  <table class="industry-variables">
                                    <thead>
                                      <tr>
                                        <th>可观测变量</th>
                                        <th>它在链条哪一环</th>
                                        <th>去哪取</th>
                                      </tr>
                                    </thead>
                                    <tbody>
                                      <For each={industry().ai_valuation_logic.key_variables}>
                                        {(variable) => (
                                          <tr>
                                            <td>{variable.name}</td>
                                            <td>{variable.why}</td>
                                            <td class="industry-where">{variable.where}</td>
                                          </tr>
                                        )}
                                      </For>
                                    </tbody>
                                  </table>
                                </Show>
                                <dl class="industry-anchor">
                                  <dt>倍数锚</dt>
                                  <dd>{industry().ai_valuation_logic.multiple_anchor || "—"}</dd>
                                  <dt>这一行最常见的估值错法</dt>
                                  <dd>{industry().ai_valuation_logic.anti_pattern || "—"}</dd>
                                </dl>
                              </Show>

                              <h3>核心关注点</h3>
                              <Show
                                when={industry().core_watch.length > 0}
                                fallback={<p class="industry-detail-note">尚未定稿。</p>}
                              >
                                <ul class="industry-watch">
                                  <For each={industry().core_watch}>
                                    {(watch) => (
                                      <li>
                                        <strong>{watch.what}</strong>
                                        <span class="industry-cadence">{watch.cadence}</span>
                                        <p>{watch.why}</p>
                                      </li>
                                    )}
                                  </For>
                                </ul>
                              </Show>

                              <h3>研报与数据来源</h3>
                              <Show
                                when={industry().sources.length > 0}
                                fallback={<p class="industry-detail-note">尚未定稿。</p>}
                              >
                                <ul class="industry-sources">
                                  <For each={industry().sources}>
                                    {(source) => (
                                      <li>
                                        <a href={source.url} target="_blank" rel="noreferrer">
                                          {source.house}｜{source.title}
                                        </a>
                                        <span class="industry-source-date">{source.date}</span>
                                        <p>{source.takeaway}</p>
                                      </li>
                                    )}
                                  </For>
                                </ul>
                              </Show>
                            </section>
                          )}
                        </Show>
                      </div>
                    </div>
                  )}
                </Show>
              </Show>
            </Show>
          </PublicWorkspaceShell>
        </Show>
      </Show>
    </>
  );
}
