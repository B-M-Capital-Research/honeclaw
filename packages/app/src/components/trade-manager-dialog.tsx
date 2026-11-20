import { createEffect, createMemo, createSignal, onCleanup, Show } from "solid-js";
import { Portal } from "solid-js/web";
import { getPublicAuthMe } from "@/lib/api";
import { useLocale } from "@/lib/i18n";
import { onPublicAuthInvalidated } from "@/lib/public-auth-lifecycle";
import { createTradeManagerDownload, type TradeManagerDownload, createTradeManagerSession, readTradeManagerMessage, tradeManagerCloseAction, type TradeManagerSessionState } from "@/lib/trade-manager-session";
import "./trade-manager-dialog.css";

export const tradeManagerLabel = () => useLocale() === "en" ? "Trade manager" : "交易管理";
export const tradeManagerHint = () => useLocale() === "en" ? "Your portfolio, trade plans and review journal" : "个人持仓、交易计划与复盘记录";

/** Static UI is public; every opening and every data operation uses server auth. */
export function TradeManagerDialog(props: { openRequest: number; userId?: string }) {
  const t = (zh: string, en: string) => useLocale() === "en" ? en : zh;
  const [state, setState] = createSignal<TradeManagerSessionState>({ status: "closed" });
  const [childStatus, setChildStatus] = createSignal({ dirty: false, busy: false });
  const [closeNotice, setCloseNotice] = createSignal<"wait" | "confirm">();
  const [download, setDownload] = createSignal<TradeManagerDownload>();
  const [downloadClicked, setDownloadClicked] = createSignal(false);
  const [downloadError, setDownloadError] = createSignal(false);
  const downloadLease = createTradeManagerDownload(setDownload);
  const clearDownload = () => { downloadLease.clear(); setDownloadClicked(false); setDownloadError(false); };
  const session = createTradeManagerSession(getPublicAuthMe, setState);
  const isOpen = createMemo(() => state().status !== "closed");
  let handled = props.openRequest;
  let owner = props.userId;
  let frame: HTMLIFrameElement | undefined;
  const close = () => { clearDownload(); setCloseNotice(undefined); setChildStatus({ dirty: false, busy: false }); frame = undefined; session.close(); };
  const requestClose = () => {
    const action = tradeManagerCloseAction(childStatus());
    if (action === "close") close();
    else setCloseNotice(action);
  };
  const invalidate = () => {
    // Authentication changes bypass dirty-form confirmation. Clear before unmount.
    frame?.contentWindow?.postMessage({ type: "hone:auth-invalidated" }, window.location.origin);
    close();
  };
  const receive = (event: MessageEvent) => {
    const message = readTradeManagerMessage(event, frame?.contentWindow, window.location.origin);
    // A server-expired/revoked child session must also destroy parent exports.
    // This closes this tool only; it does not mutate Hone's global login state.
    if (message?.type === "auth-invalidated") { invalidate(); return; }
    if (message?.type === "export" && state().status === "ready") {
      clearDownload();
      try { downloadLease.prepare(message); } catch { setDownloadError(true); }
    }
    if (message?.type === "close") requestClose();
    if (message?.type === "status") {
      setChildStatus({ dirty: message.dirty, busy: message.busy });
      if (closeNotice() === "wait" && !message.busy) setCloseNotice(undefined);
    }
  };
  const unsubscribe = onPublicAuthInvalidated(invalidate);
  window.addEventListener("message", receive);
  onCleanup(() => { unsubscribe(); window.removeEventListener("message", receive); invalidate(); });

  createEffect(() => {
    const userId = props.userId;
    if (owner !== userId) { owner = userId; invalidate(); }
    if (props.openRequest <= handled) return;
    handled = props.openRequest;
    if (userId) { close(); void session.open(userId); }
  });
  createEffect(() => {
    if (!isOpen()) return;
    const previousOverflow = document.documentElement.style.overflow;
    const previousFocus = document.activeElement instanceof HTMLElement ? document.activeElement : undefined;
    document.documentElement.style.overflow = "hidden";
    onCleanup(() => {
      document.documentElement.style.overflow = previousOverflow;
      if (previousFocus?.isConnected) previousFocus.focus();
    });
  });

  return <Portal><Show when={isOpen()}>
    <dialog class="trade-manager-dialog" aria-label={tradeManagerLabel()}
      ref={(dialog) => queueMicrotask(() => { if (dialog.isConnected && isOpen() && !dialog.open) dialog.showModal(); })}
      onCancel={(event) => { event.preventDefault(); requestClose(); }}>
      <header class="trade-manager-dialog__header">
        <div><h2>{tradeManagerLabel()}</h2><p>{t("当前 Hone 账户的独立交易记录", "Private trading records for your current Hone account")}</p></div>
        <button type="button" onClick={requestClose} aria-label={t("关闭交易管理", "Close trade manager")}>{t("关闭", "Close")} ×</button>
      </header>
      <Show when={closeNotice()}>
        <div class="trade-manager-dialog__notice" role="alert">
          <p>{closeNotice() === "wait"
            ? t("正在保存，请等待结果后再关闭。", "Saving. Please wait for the result before closing.")
            : t("还有未提交的修改。关闭会放弃这些修改。", "You have unsaved changes. Closing will discard them.")}</p>
          <button type="button" onClick={() => setCloseNotice(undefined)}>{t("继续编辑", "Keep editing")}</button>
          <Show when={closeNotice() === "confirm"}>
            <button type="button" onClick={() => childStatus().busy ? setCloseNotice("wait") : close()}>{t("放弃修改并关闭", "Discard and close")}</button>
          </Show>
        </div>
      </Show>
      <Show when={download()}>{(file) =>
        <div class="trade-manager-dialog__download" role="status">
          <div><strong>{file().name}</strong><p>{downloadClicked()
            ? t("已点击下载，请检查浏览器下载列表。", "Download clicked. Check your browser's download list.")
            : t("文件已准备，尚未下载。请点击链接；链接在 15 分钟后失效。", "File prepared, not yet downloaded. Click the link; it expires in 15 minutes.")}</p></div>
          <a href={file().url} download={file().name} onClick={() => setDownloadClicked(true)}>{t("点击下载文件", "Click to download file")}</a>
          <button type="button" onClick={clearDownload}>{t("清除", "Clear")}</button>
        </div>
      }</Show>
      <Show when={downloadError()}><p class="trade-manager-dialog__download-error" role="alert">{t("无法准备下载，请重新导出。", "Unable to prepare the download. Export again.")}</p></Show>
      <Show when={state().status === "ready"} fallback={
        <div class="trade-manager-dialog__status" role="status">
          <Show when={state().status === "checking"} fallback={<>
            <p>{t("无法确认当前账户。请关闭后重新登录或重试。", "Unable to verify this account. Close and sign in again, or retry.")}</p>
            <button type="button" onClick={() => props.userId && void session.open(props.userId)}>{t("重试", "Retry")}</button>
          </>}><p>{t("正在验证账户…", "Verifying your account…")}</p></Show>
        </div>
      }>
        <iframe ref={frame} class="trade-manager-dialog__frame" title={tradeManagerLabel()}
          src="/trade-manager/?embedded=1"
          sandbox="allow-scripts allow-same-origin allow-forms allow-downloads allow-popups allow-modals"
          referrerpolicy="no-referrer"
          onLoad={(event) => {
            try {
              const location = event.currentTarget.contentWindow?.location;
              const allowedPath = location?.pathname === "/trade-manager/" || location?.pathname === "/trade-manager/index.html";
              if (!location || location.origin !== window.location.origin || !allowedPath) invalidate();
            } catch { invalidate(); }
          }} />
      </Show>
    </dialog>
  </Show></Portal>;
}
