import { Title } from "@solidjs/meta";
import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import { PublicLoginForm } from "@/components/public-login-form";
import { PublicWorkspaceShell } from "@/components/public-workspace-shell";
import {
  deletePublicResearchLibrary,
  getPublicResearchLibrary,
  isUnauthorizedApiError,
  reviewPublicResearchLibraryCandidate,
  submitPublicResearchLibraryCandidate,
  updatePublicResearchLibrary,
  uploadPublicResearchLibrary,
} from "@/lib/api";
import type {
  ResearchLibraryBundle,
  ResearchLibraryItem,
  ResearchLibraryScope,
  ResearchLibrarySourceType,
  ResearchLibraryUse,
} from "@/lib/types";

import "./public-foundation.css";
import "./public-site.css";
import "./public-polish.css";
import "./public-research-library.css";

type ViewState = "loading" | "ready" | "login" | "error";
type ScopeFilter = "all" | ResearchLibraryScope;

const USE_LABELS: Array<{
  id: ResearchLibraryUse;
  label: string;
  hint: string;
}> = [
  { id: "chat", label: "HONE 问答", hint: "提问相关公司或主题时检索" },
  { id: "portfolio_news", label: "持仓新闻", hint: "按股票代码进入个人日报" },
  {
    id: "key_event_chain",
    label: "关键事件链",
    hint: "全局资料才会进入公共事件链",
  },
];

const SOURCE_LABELS: Record<ResearchLibrarySourceType, string> = {
  manual_upload: "本地文件",
  zsxq_export: "知识星球导出",
  ima_export: "IMA 导出",
  authorized_connector: "官方授权连接器",
};

function today() {
  const now = new Date();
  return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}-${String(now.getDate()).padStart(2, "0")}`;
}

function formatBytes(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

export default function PublicResearchLibraryPage() {
  const [view, setView] = createSignal<ViewState>("loading");
  const [bundle, setBundle] = createSignal<ResearchLibraryBundle>();
  const [filter, setFilter] = createSignal<ScopeFilter>("all");
  const [file, setFile] = createSignal<File>();
  const [sourceType, setSourceType] =
    createSignal<ResearchLibrarySourceType>("manual_upload");
  const [sourceName, setSourceName] = createSignal("");
  const [sourceUrl, setSourceUrl] = createSignal("");
  const [sourceDate, setSourceDate] = createSignal(today());
  const [tickers, setTickers] = createSignal("");
  const [topics, setTopics] = createSignal("");
  const [scope, setScope] = createSignal<ResearchLibraryScope>("personal");
  const [uses, setUses] = createSignal<ResearchLibraryUse[]>([
    "chat",
    "portfolio_news",
  ]);
  const [uploading, setUploading] = createSignal(false);
  const [workingItem, setWorkingItem] = createSignal("");
  const [notice, setNotice] = createSignal("");
  const [error, setError] = createSignal("");
  const [confirmingRemove, setConfirmingRemove] = createSignal("");
  const [confirmingSubmit, setConfirmingSubmit] = createSignal("");
  const [reviewDraft, setReviewDraft] = createSignal<{
    id: string;
    decision: "approve" | "reject";
  } | null>(null);
  const [reviewNote, setReviewNote] = createSignal("");
  let fileInput: HTMLInputElement | undefined;

  const load = async () => {
    setError("");
    try {
      const next = await getPublicResearchLibrary();
      setBundle(next);
      setView("ready");
    } catch (cause) {
      if (isUnauthorizedApiError(cause)) {
        setView("login");
      } else {
        setError(cause instanceof Error ? cause.message : String(cause));
        setView("error");
      }
    }
  };

  onMount(() => void load());

  /** In-place merge (same spirit as community-forum's mergePost): replace by
      id when present, otherwise prepend, so成功操作不需要全量重拉。 */
  const mergeItems = (
    ...incoming: Array<ResearchLibraryItem | null | undefined>
  ) => {
    setBundle((current) => {
      if (!current) return current;
      let items = current.items;
      for (const item of incoming) {
        if (!item) continue;
        items = items.some((value) => value.id === item.id)
          ? items.map((value) => (value.id === item.id ? item : value))
          : [item, ...items];
      }
      return { ...current, items };
    });
  };

  const filteredItems = createMemo(() => {
    const current = filter();
    return (bundle()?.items ?? []).filter(
      (item) => current === "all" || item.scope === current,
    );
  });

  const toggleDraftUse = (usage: ResearchLibraryUse) => {
    setUses((current) =>
      current.includes(usage)
        ? current.filter((item) => item !== usage)
        : [...current, usage],
    );
  };

  const submit = async (event: SubmitEvent) => {
    event.preventDefault();
    const selected = file();
    if (!selected) {
      setError("请先选择一个研究文件。");
      return;
    }
    setUploading(true);
    setError("");
    setNotice("");
    try {
      const form = new FormData();
      form.append("file", selected);
      form.append("scope", scope());
      form.append("source_type", sourceType());
      form.append("source_name", sourceName());
      form.append("source_url", sourceUrl());
      form.append("source_date", sourceDate());
      form.append("tickers", tickers());
      form.append("topics", topics());
      form.append("uses", uses().join(","));
      const result = await uploadPublicResearchLibrary(form);
      setNotice(
        result.deduplicated
          ? "相同文件已存在，未重复保存。"
          : "资料已归档并加入获授权的产品流程。 ",
      );
      setFile(undefined);
      setSourceName("");
      setSourceUrl("");
      setTickers("");
      setTopics("");
      if (fileInput) fileInput.value = "";
      mergeItems(result.item);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setUploading(false);
    }
  };

  const toggleItemUse = async (
    item: ResearchLibraryItem,
    usage: ResearchLibraryUse,
  ) => {
    const next = item.uses.includes(usage)
      ? item.uses.filter((value) => value !== usage)
      : [...item.uses, usage];
    setError("");
    try {
      const updated = await updatePublicResearchLibrary(item.id, {
        uses: next,
      });
      setBundle((current) =>
        current
          ? {
              ...current,
              items: current.items.map((value) =>
                value.id === item.id ? updated : value,
              ),
            }
          : current,
      );
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  };

  const remove = async (item: ResearchLibraryItem) => {
    setError("");
    try {
      await deletePublicResearchLibrary(item.id);
      setConfirmingRemove("");
      setBundle((current) =>
        current
          ? {
              ...current,
              items: current.items.filter((value) => value.id !== item.id),
            }
          : current,
      );
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  };

  const chooseConnectorImport = (type: ResearchLibrarySourceType) => {
    setSourceType(type);
    fileInput?.click();
  };

  const submitCandidate = async (item: ResearchLibraryItem) => {
    setWorkingItem(item.id);
    setError("");
    try {
      const result = await submitPublicResearchLibraryCandidate(item.id);
      setNotice(
        result.deduplicated
          ? "这份资料已经在审核队列中。"
          : "已提交审核；批准前仍只属于候选资料。",
      );
      setConfirmingSubmit("");
      mergeItems(result.item);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setWorkingItem("");
    }
  };

  const reviewCandidate = async (
    item: ResearchLibraryItem,
    decision: "approve" | "reject",
  ) => {
    const note = reviewNote().trim();
    setWorkingItem(item.id);
    setError("");
    try {
      const result = await reviewPublicResearchLibraryCandidate(
        item.id,
        decision,
        note,
      );
      setNotice(
        decision === "approve"
          ? "已核验并采纳到 HONE 官方研究库。"
          : "已驳回；候选资料不会进入任何 Agent 或每日产品。 ",
      );
      setReviewDraft(null);
      setReviewNote("");
      mergeItems(result.item, result.promoted_item);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setWorkingItem("");
    }
  };

  return (
    <>
      <Title>我的知识源 · HONE</Title>
      <Show
        when={view() !== "loading"}
        fallback={
          <div class="research-library-loading">正在读取研究资料库…</div>
        }
      >
        <Show
          when={view() !== "login"}
          fallback={
            <PublicLoginForm
              title="登录后管理研究资料"
              subtitle="个人资料只会用于你的 HONE 问答与持仓日报。"
              onLogin={() => void load()}
            />
          }
        >
          <PublicWorkspaceShell active="research" topbarLabel="我的知识源">
            <div class="research-library-page">
              <header class="research-library-hero">
                <div>
                  <span class="research-library-eyebrow">
                    MY KNOWLEDGE SOURCES
                  </span>
                  <h1>我的知识源</h1>
                  <p>
                    连接你自己的研究材料。个人资料只服务你的问答与持仓；只有经过管理员核验的投稿，才会进入
                    HONE 官方研究库。
                  </p>
                </div>
                <div class="research-library-hero__stats">
                  <strong>{bundle()?.items.length ?? 0}</strong>
                  <span>份已归档资料</span>
                </div>
              </header>

              <section
                class="research-library-connectors"
                aria-label="外部知识库接入方式"
              >
                <article class="research-library-connector">
                  <span>知识星球</span>
                  <strong>官方 Skill · 只读导入</strong>
                  <p>
                    {bundle()?.connector_status.zsxq.note ??
                      "使用官方授权工具导出后导入，不模拟登录。"}
                  </p>
                  <div>
                    <button
                      type="button"
                      onClick={() => chooseConnectorImport("zsxq_export")}
                    >
                      导入同步包
                    </button>
                    <a
                      href={bundle()?.connector_status.zsxq.guide_url}
                      target="_blank"
                      rel="noreferrer"
                    >
                      官方说明
                    </a>
                  </div>
                </article>
                <article class="research-library-connector">
                  <span>iMA</span>
                  <strong>个人导出 · 只读导入</strong>
                  <p>
                    {bundle()?.connector_status.ima.note ??
                      "当前导入用户主动导出的文件。"}
                  </p>
                  <div>
                    <button
                      type="button"
                      onClick={() => chooseConnectorImport("ima_export")}
                    >
                      选择 iMA 文件
                    </button>
                    <small>不读取私人会话</small>
                  </div>
                </article>
                <article>
                  <span>信任升级</span>
                  <strong>个人 → 候选 → 官方</strong>
                  <p>
                    社区投稿先隔离审核。管理员核对来源、日期和事实后，才能采纳进
                    投资助手与每日产品。
                  </p>
                </article>
              </section>

              <section class="research-library-upload">
                <div class="research-library-section-title">
                  <div>
                    <span>DAILY IMPORT</span>
                    <h2>上传今天的材料</h2>
                  </div>
                  <small>单文件最大 20 MB；相同内容自动去重</small>
                </div>
                <form onSubmit={submit}>
                  <label class="research-library-file">
                    <input
                      ref={fileInput}
                      type="file"
                      accept=".pdf,.txt,.md,.csv,.tsv,.json,.yaml,.yml,.doc,.docx,.xls,.xlsx,.ppt,.pptx"
                      onChange={(event) =>
                        setFile(event.currentTarget.files?.[0])
                      }
                    />
                    <span>
                      {file() ? file()!.name : "选择报告、纪要或平台导出文件"}
                    </span>
                    <b>{file() ? formatBytes(file()!.size) : "浏览文件"}</b>
                  </label>
                  <div class="research-library-form-grid">
                    <label>
                      来源类型
                      <select
                        value={sourceType()}
                        onChange={(event) =>
                          setSourceType(
                            event.currentTarget
                              .value as ResearchLibrarySourceType,
                          )
                        }
                      >
                        <option value="manual_upload">本地文件</option>
                        <option value="zsxq_export">知识星球导出</option>
                        <option value="ima_export">IMA 导出</option>
                        <option value="authorized_connector">
                          官方授权连接器
                        </option>
                      </select>
                    </label>
                    <label>
                      资料日期
                      <input
                        type="date"
                        value={sourceDate()}
                        onInput={(event) =>
                          setSourceDate(event.currentTarget.value)
                        }
                      />
                    </label>
                    <label>
                      来源名称
                      <input
                        value={sourceName()}
                        placeholder="例如：老王知识星球 / 公司 IR"
                        onInput={(event) =>
                          setSourceName(event.currentTarget.value)
                        }
                      />
                    </label>
                    <label>
                      原始链接
                      <input
                        type="url"
                        value={sourceUrl()}
                        placeholder="https://（可选）"
                        onInput={(event) =>
                          setSourceUrl(event.currentTarget.value)
                        }
                      />
                    </label>
                    <label>
                      股票代码
                      <input
                        value={tickers()}
                        placeholder="NVDA, TSM, MU"
                        onInput={(event) =>
                          setTickers(event.currentTarget.value)
                        }
                      />
                    </label>
                    <label>
                      主题标签
                      <input
                        value={topics()}
                        placeholder="Rubin, HBM, AI Capex"
                        onInput={(event) =>
                          setTopics(event.currentTarget.value)
                        }
                      />
                    </label>
                  </div>
                  <Show when={bundle()?.is_admin}>
                    <div class="research-library-scope">
                      <button
                        type="button"
                        classList={{ "is-active": scope() === "personal" }}
                        onClick={() => setScope("personal")}
                      >
                        个人资料
                      </button>
                      <button
                        type="button"
                        classList={{ "is-active": scope() === "hone_global" }}
                        onClick={() => setScope("hone_global")}
                      >
                        HONE 全局资料
                      </button>
                    </div>
                  </Show>
                  <div class="research-library-use-grid">
                    <For each={USE_LABELS}>
                      {(usage) => (
                        <label
                          classList={{ "is-active": uses().includes(usage.id) }}
                        >
                          <input
                            type="checkbox"
                            checked={uses().includes(usage.id)}
                            onChange={() => toggleDraftUse(usage.id)}
                          />
                          <span>
                            <strong>{usage.label}</strong>
                            <small>{usage.hint}</small>
                          </span>
                        </label>
                      )}
                    </For>
                  </div>
                  <div class="research-library-submit-row">
                    <Show when={notice()}>
                      <p class="is-notice">{notice()}</p>
                    </Show>
                    <Show when={error()}>
                      <p class="is-error">{error()}</p>
                    </Show>
                    <button type="submit" disabled={uploading()}>
                      {uploading() ? "正在解析…" : "归档到 HONE"}
                    </button>
                  </div>
                </form>
              </section>

              <section class="research-library-list">
                <div class="research-library-section-title">
                  <div>
                    <span>LIBRARY</span>
                    <h2>资料与授权</h2>
                  </div>
                  <div class="research-library-filters">
                    <For
                      each={
                        [
                          { id: "all", label: "全部" },
                          { id: "personal", label: "个人" },
                          { id: "community_candidate", label: "审核候选" },
                          { id: "hone_global", label: "HONE 官方" },
                        ] as const
                      }
                    >
                      {(option) => (
                        <button
                          type="button"
                          classList={{ "is-active": filter() === option.id }}
                          onClick={() => setFilter(option.id)}
                        >
                          {option.label}
                        </button>
                      )}
                    </For>
                  </div>
                </div>
                <Show
                  when={view() !== "error"}
                  fallback={
                    <div class="research-library-empty">
                      <strong>暂时无法读取资料库</strong>
                      <span>{error()}</span>
                      <button type="button" onClick={() => void load()}>
                        重新读取
                      </button>
                    </div>
                  }
                >
                  <Show
                    when={filteredItems().length > 0}
                    fallback={
                      <div class="research-library-empty">
                        <strong>还没有资料</strong>
                        <span>
                          先上传一份今天的报告或纪要，HONE
                          就会开始建立可追溯的研究底座。
                        </span>
                      </div>
                    }
                  >
                    <div class="research-library-items">
                      <For each={filteredItems()}>
                        {(item) => (
                          <article>
                            <header>
                              <div>
                                <span
                                  class={`parse-status is-${item.parse_status}`}
                                >
                                  {item.parse_status === "ready"
                                    ? "可检索"
                                    : item.parse_status === "stored"
                                      ? "已归档"
                                      : "解析异常"}
                                </span>
                                <span>
                                  {item.scope === "hone_global"
                                    ? "HONE 官方"
                                    : item.scope === "community_candidate"
                                      ? "审核候选"
                                      : "个人"}
                                </span>
                                <Show
                                  when={item.scope === "community_candidate"}
                                >
                                  <span
                                    class={`review-status is-${item.review_status}`}
                                  >
                                    {item.review_status === "pending"
                                      ? "待审核"
                                      : item.review_status === "approved"
                                        ? "已采纳"
                                        : "已驳回"}
                                  </span>
                                </Show>
                              </div>
                              <Show
                                when={confirmingRemove() === item.id}
                                fallback={
                                  <button
                                    type="button"
                                    onClick={() =>
                                      setConfirmingRemove(item.id)
                                    }
                                  >
                                    删除
                                  </button>
                                }
                              >
                                <div
                                  class="research-library-confirm"
                                  role="group"
                                  aria-label={`确认删除“${item.title}”`}
                                >
                                  <span>删除后下游产品将不再读取这份资料</span>
                                  <button
                                    type="button"
                                    class="is-danger"
                                    onClick={() => void remove(item)}
                                  >
                                    确认删除
                                  </button>
                                  <button
                                    type="button"
                                    onClick={() => setConfirmingRemove("")}
                                  >
                                    取消
                                  </button>
                                </div>
                              </Show>
                            </header>
                            <h3>{item.title}</h3>
                            <p class="research-library-item__meta">
                              {SOURCE_LABELS[item.source_type]} ·{" "}
                              {item.source_name} · 资料日 {item.source_date} ·{" "}
                              {formatBytes(item.size)}
                            </p>
                            <Show
                              when={bundle()?.is_admin && item.submitted_by}
                            >
                              <p class="research-library-item__meta">
                                投稿账号：{item.submitted_by}
                              </p>
                            </Show>
                            <Show when={item.excerpt}>
                              <p class="research-library-item__excerpt">
                                {item.excerpt}
                              </p>
                            </Show>
                            <Show when={item.review_note}>
                              <p class="research-library-item__review">
                                审核备注：{item.review_note}
                              </p>
                            </Show>
                            <div class="research-library-tags">
                              <For each={[...item.tickers, ...item.topics]}>
                                {(tag) => <span>{tag}</span>}
                              </For>
                            </div>
                            <footer>
                              <div class="research-library-item__uses">
                                <Show
                                  when={item.scope !== "community_candidate"}
                                >
                                  <For each={USE_LABELS}>
                                    {(usage) => (
                                      <button
                                        type="button"
                                        classList={{
                                          "is-active": item.uses.includes(
                                            usage.id,
                                          ),
                                        }}
                                        onClick={() =>
                                          void toggleItemUse(item, usage.id)
                                        }
                                      >
                                        {usage.label}
                                      </button>
                                    )}
                                  </For>
                                </Show>
                                <Show when={item.scope === "personal"}>
                                  <Show
                                    when={confirmingSubmit() === item.id}
                                    fallback={
                                      <button
                                        type="button"
                                        class="is-submit"
                                        disabled={
                                          workingItem() === item.id ||
                                          item.parse_status !== "ready"
                                        }
                                        onClick={() =>
                                          setConfirmingSubmit(item.id)
                                        }
                                      >
                                        投稿给 HONE
                                      </button>
                                    }
                                  >
                                    <div
                                      class="research-library-confirm"
                                      role="group"
                                      aria-label={`确认把“${item.title}”提交给管理员审核`}
                                    >
                                      <span>
                                        提交给管理员审核；审核前不会影响其他用户或官方评级
                                      </span>
                                      <button
                                        type="button"
                                        class="is-submit"
                                        disabled={workingItem() === item.id}
                                        onClick={() =>
                                          void submitCandidate(item)
                                        }
                                      >
                                        确认投稿
                                      </button>
                                      <button
                                        type="button"
                                        onClick={() => setConfirmingSubmit("")}
                                      >
                                        取消
                                      </button>
                                    </div>
                                  </Show>
                                </Show>
                                <Show
                                  when={
                                    bundle()?.is_admin &&
                                    item.scope === "community_candidate" &&
                                    item.review_status === "pending"
                                  }
                                >
                                  <button
                                    type="button"
                                    class="is-approve"
                                    disabled={workingItem() === item.id}
                                    classList={{
                                      "is-active":
                                        reviewDraft()?.id === item.id &&
                                        reviewDraft()?.decision === "approve",
                                    }}
                                    onClick={() => {
                                      setReviewNote("");
                                      setReviewDraft({
                                        id: item.id,
                                        decision: "approve",
                                      });
                                    }}
                                  >
                                    核验并采纳
                                  </button>
                                  <button
                                    type="button"
                                    class="is-reject"
                                    disabled={workingItem() === item.id}
                                    classList={{
                                      "is-active":
                                        reviewDraft()?.id === item.id &&
                                        reviewDraft()?.decision === "reject",
                                    }}
                                    onClick={() => {
                                      setReviewNote("");
                                      setReviewDraft({
                                        id: item.id,
                                        decision: "reject",
                                      });
                                    }}
                                  >
                                    驳回
                                  </button>
                                </Show>
                              </div>
                              <a
                                href={item.download_url}
                                download={item.filename}
                              >
                                下载原文件
                              </a>
                            </footer>
                            <Show when={reviewDraft()?.id === item.id}>
                              <div class="research-library-review-form">
                                <label>
                                  {reviewDraft()?.decision === "approve"
                                    ? "核验依据或采纳备注（可留空）"
                                    : "驳回原因（建议填写）"}
                                  <textarea
                                    rows={3}
                                    value={reviewNote()}
                                    onInput={(event) =>
                                      setReviewNote(event.currentTarget.value)
                                    }
                                    placeholder={
                                      reviewDraft()?.decision === "approve"
                                        ? "记录你核对过的来源、日期与事实…"
                                        : "说明驳回原因，方便投稿人修订…"
                                    }
                                  />
                                </label>
                                <div class="research-library-review-form__actions">
                                  <button
                                    type="button"
                                    classList={{
                                      "is-danger":
                                        reviewDraft()?.decision === "reject",
                                      "is-primary":
                                        reviewDraft()?.decision === "approve",
                                    }}
                                    disabled={workingItem() === item.id}
                                    onClick={() =>
                                      void reviewCandidate(
                                        item,
                                        reviewDraft()!.decision,
                                      )
                                    }
                                  >
                                    {reviewDraft()?.decision === "approve"
                                      ? "确认采纳"
                                      : "确认驳回"}
                                  </button>
                                  <button
                                    type="button"
                                    onClick={() => {
                                      setReviewDraft(null);
                                      setReviewNote("");
                                    }}
                                  >
                                    取消
                                  </button>
                                </div>
                              </div>
                            </Show>
                          </article>
                        )}
                      </For>
                    </div>
                  </Show>
                </Show>
              </section>
            </div>
          </PublicWorkspaceShell>
        </Show>
      </Show>
    </>
  );
}
