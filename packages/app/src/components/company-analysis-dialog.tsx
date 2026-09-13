import { createEffect, createMemo, createSignal, For, onCleanup, Show } from "solid-js";
import { Portal } from "solid-js/web";
import { useLocale } from "@/lib/i18n";
import { canResumeCompanyAnalysis, companyAnalysisProgress, downloadCompanyAnalysis,
  getCompanyAnalysis, listCompanyAnalyses, resumeCompanyAnalysis, startCompanyAnalysis,
  type CompanyAnalysisTask } from "@/lib/company-analysis";
import "./company-analysis-dialog.css";

export function CompanyAnalysisDialog(props: { openRequest: number; disabled: boolean }) {
  const [open, setOpen] = createSignal(false);
  const [company, setCompany] = createSignal("");
  const [task, setTask] = createSignal<CompanyAnalysisTask>();
  const [history, setHistory] = createSignal<CompanyAnalysisTask[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [busy, setBusy] = createSignal(false);
  const [downloading, setDownloading] = createSignal(false);
  const [error, setError] = createSignal("");
  const [pollError, setPollError] = createSignal("");
  let handled = props.openRequest;
  let input: HTMLInputElement | undefined;
  const t = (zh: string, en: string) => useLocale() === "en" ? en : zh;
  const stageLabel = (stage: string) => ({
    "公司基本介绍": t("公司概况", "Company overview"),
    "公司财务指标获取": t("财务数据", "Financial data"),
    "产品分析": t("产品分析", "Products"),
    "产品分析附图": t("业务图解", "Business diagrams"),
    "产品竞争力分析": t("竞争力分析", "Competitive position"),
    "管理层分析": t("管理层分析", "Management"),
    "财务分析": t("财务分析", "Financial analysis"),
    "全跑完-美": t("估值分析", "Valuation"),
    "基于报告内容生成名字": t("报告整理", "Report assembly"),
  }[stage] ?? stage);
  const close = () => { if (!busy()) setOpen(false); };
  const running = () => task()?.status === "running";

  createEffect(() => {
    if (props.openRequest <= handled) return;
    handled = props.openRequest;
    if (!props.disabled) { setOpen(true); setError(""); }
  });
  createEffect(() => {
    if (!open()) return;
    const previous = document.documentElement.style.overflow;
    document.documentElement.style.overflow = "hidden";
    const key = (event: KeyboardEvent) => { if (event.key === "Escape") close(); };
    document.addEventListener("keydown", key);
    if (window.matchMedia("(hover: hover) and (pointer: fine)").matches) queueMicrotask(() => input?.focus());
    const controller = new AbortController();
    setLoading(true);
    void listCompanyAnalyses(controller.signal).then((tasks) => {
      if (controller.signal.aborted) return;
      setHistory(tasks);
      const active = tasks.find((item) => item.status === "running" || item.status === "interrupted");
      if (active) { setTask(active); setCompany(active.company); }
    }).catch((cause) => { if (!controller.signal.aborted) setError(String(cause.message || cause)); }).finally(() => { if (!controller.signal.aborted) setLoading(false); });
    onCleanup(() => {
      controller.abort(); document.documentElement.style.overflow = previous;
      document.removeEventListener("keydown", key);
    });
  });

  // Poll only while this task is visible. Closing the dialog never cancels a
  // server task; reopening retrieves its durable status without repeating POST.
  const pollingTaskId = createMemo(() => open() && task()?.status === "running" ? task()?.task_id : undefined);
  createEffect(() => {
    const id = pollingTaskId();
    if (!id) return;
    const controller = new AbortController();
    let timer: ReturnType<typeof setTimeout>;
    const poll = async () => {
      try {
        const latest = await getCompanyAnalysis(id, controller.signal);
        if (controller.signal.aborted || task()?.task_id !== id) return;
        setPollError(""); setTask(latest);
        setHistory((items) => [latest, ...items.filter((item) => item.task_id !== id)]);
      } catch (cause) {
        if (controller.signal.aborted) return;
        setPollError(t("进度连接暂时中断，正在重新连接；任务仍在后台运行。", "Reconnecting to progress. The task continues in the background."));
      }
      if (!controller.signal.aborted) timer = setTimeout(poll, 2500);
    };
    timer = setTimeout(poll, 500);
    onCleanup(() => { controller.abort(); clearTimeout(timer); });
  });

  const selectTask = (value: CompanyAnalysisTask) => { setTask(value); setCompany(value.company); setError(""); setPollError(""); };
  const submit = async () => {
    if (!company().trim() || busy() || running()) return;
    setBusy(true); setError("");
    try { selectTask(await startCompanyAnalysis(company())); }
    catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
      // A lost response can follow an accepted POST. Discover the accepted
      // task rather than sending a duplicate paid research request.
      try {
        const tasks = await listCompanyAnalyses(); setHistory(tasks);
        const active = tasks.find((item) => item.status === "running");
        if (active) selectTask(active);
      } catch { /* Preserve the original error; a later reopen retries GET. */ }
    } finally { setBusy(false); }
  };
  const resume = async () => {
    const current = task(); if (!current || busy()) return;
    setBusy(true); setError("");
    try { selectTask(await resumeCompanyAnalysis(current.task_id)); }
    catch (cause) { setError(cause instanceof Error ? cause.message : String(cause)); }
    finally { setBusy(false); }
  };
  const download = async () => {
    const current = task(); if (!current?.pdf_ready || downloading()) return;
    setDownloading(true); setError("");
    try {
      const blob = await downloadCompanyAnalysis(current.task_id);
      const url = URL.createObjectURL(blob); const anchor = document.createElement("a");
      anchor.href = url; anchor.download = `${current.company.replace(/[\\/:*?"<>|]/g, "-")}-公司完整分析.pdf`;
      document.body.appendChild(anchor); anchor.click(); anchor.remove();
      setTimeout(() => URL.revokeObjectURL(url), 30_000);
    } catch (cause) { setError(cause instanceof Error ? cause.message : String(cause)); }
    finally { setDownloading(false); }
  };

  return <Portal><Show when={open()}>
    <div class="public-chat-proactive-modal-backdrop public-chat-earnings-backdrop" role="presentation" onClick={close}>
      <section class="public-chat-proactive-modal public-chat-earnings-modal company-analysis-dialog" role="dialog" aria-modal="true" aria-labelledby="company-analysis-title" onClick={(event) => event.stopPropagation()}>
        <button type="button" class="public-chat-proactive-close" aria-label={t("关闭", "Close")} onClick={close} disabled={busy()}>×</button>
        <span class="public-chat-earnings-kicker">{t("管理员研究工作流", "Administrator research")}</span>
        <h2 id="company-analysis-title">{t("公司完整分析", "Full company analysis")}</h2>
        <p class="public-chat-proactive-intro">{t("输入公司名称，自动完成公司、产品、管理层、财务和估值分析，生成完整 PDF。", "Enter a company to research its business, products, management, financials and valuation, with a complete PDF.")}</p>
        <form onSubmit={(event) => { event.preventDefault(); void submit(); }}>
          <label class="public-chat-earnings-field"><span>{t("公司名称或股票代码", "Company name or ticker")}</span>
            <input ref={input} value={company()} maxlength={200} disabled={busy() || running()} placeholder="Tempus AI / TEM" data-testid="company-analysis-company" onInput={(event) => setCompany(event.currentTarget.value)} />
          </label>
          <div class="public-chat-earnings-actions"><button class="public-chat-proactive-primary" data-testid="company-analysis-start" type="submit" disabled={busy() || loading() || running() || !company().trim()}>{busy() ? t("正在启动…", "Starting…") : t("开始完整分析", "Start full analysis")}</button></div>
        </form>
        <Show when={task()}>{(current) => <div class="company-analysis-progress" aria-live="polite">
          <div class="company-analysis-progress-heading"><strong>{current().company}</strong><span>{companyAnalysisProgress(current())}%</span></div>
          <progress max={100} value={companyAnalysisProgress(current())} aria-label={t("公司分析进度", "Company analysis progress")} />
          <p>{current().status === "interrupted" ? t("任务已中断，已完成的分析已保存。", "The task was interrupted. Completed stages are saved.") : current().info}</p>
          <small class="company-analysis-task-id">{t("任务 ID：", "Task ID: ")}{current().task_id}</small>
          <Show when={running()}><small>{t("可以关闭弹窗，稍后从公司完整分析查看进度。", "You can close this dialog and return here to check progress.")}</small></Show>
          <Show when={current().completed_stages.filter((stage) => !stage.startsWith("_")).length}><ul class="company-analysis-stages"><For each={current().completed_stages.filter((stage) => !stage.startsWith("_"))}>{(stage) => <li>✓ {stageLabel(stage)}</li>}</For></ul></Show>
          <Show when={canResumeCompanyAnalysis(current())}><button class="public-chat-proactive-primary" type="button" onClick={() => void resume()} disabled={busy()}>{t("继续已保存的分析", "Resume saved analysis")}</button></Show>
          <Show when={current().warnings?.length}><p class="company-analysis-reconnecting">{t("部分图表保留了原始文字，详情见 PDF 中的提示。", "Some diagrams are preserved as source text. See the notes in the PDF.")}</p></Show>
          <Show when={current().pdf_ready}><button class="public-chat-proactive-primary" data-testid="company-analysis-download" type="button" onClick={() => void download()} disabled={downloading()}>{downloading() ? t("正在下载…", "Downloading…") : t("下载完整 PDF", "Download complete PDF")}</button></Show>
        </div>}</Show>
        <Show when={pollError()}><p role="status" class="company-analysis-reconnecting">{pollError()}</p></Show>
        <Show when={error()}><p class="public-chat-earnings-error" role="alert">{error()}</p></Show>
        <Show when={history().length}><details class="company-analysis-history"><summary>{t("最近的分析", "Recent analyses")}</summary><For each={history()}>{(item) => <button type="button" onClick={() => selectTask(item)}>{item.company} · {new Date(item.created_at).toLocaleDateString()} · {companyAnalysisProgress(item)}%</button>}</For></details></Show>
      </section>
    </div>
  </Show></Portal>;
}
