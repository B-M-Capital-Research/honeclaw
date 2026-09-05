import { Show, createEffect, createSignal, onCleanup } from "solid-js";
import { getDocument, GlobalWorkerOptions, version } from "pdfjs-dist/legacy/build/pdf.mjs";
import type { PDFDocumentLoadingTask, PDFDocumentProxy, PDFPageProxy, RenderTask } from "pdfjs-dist";
import workerUrl from "pdfjs-dist/legacy/build/pdf.worker.min.mjs?url";
import { CONTENT } from "@/lib/public-content";

GlobalWorkerOptions.workerSrc = workerUrl;
const MAX_PDF_BYTES = 128 * 1024 * 1024;
const MAX_CANVAS_PIXELS = 8 * 1024 * 1024;

/** Only the display API: no annotation actions, viewer scripting, or PDF navigation. */
export default function CommunityPdfPreview(props: { blob: Blob | null; loadError: boolean }) {
  const copy = CONTENT.chat_page.community_page;
  const [document, setDocument] = createSignal<PDFDocumentProxy>();
  const [pageNumber, setPageNumber] = createSignal(1);
  const [zoom, setZoom] = createSignal(1);
  const [width, setWidth] = createSignal(0);
  const [state, setState] = createSignal<"loading" | "rendering" | "ready" | "error">("loading");
  const [slow, setSlow] = createSignal(false);
  const [pageText, setPageText] = createSignal("");
  let viewport!: HTMLDivElement;
  let canvas!: HTMLCanvasElement;
  let loadingTask: PDFDocumentLoadingTask | undefined;
  let renderTask: RenderTask | undefined;
  let renderedPage: PDFPageProxy | undefined;
  let renderQueue = Promise.resolve();
  let generation = 0;
  let disposed = false;
  const slowTimer = window.setTimeout(() => setSlow(true), 5_000);
  const pageLabel = () => copy.pdf_page.replace("{page}", String(pageNumber())).replace("{total}", String(document()?.numPages ?? 0));

  createEffect(() => {
    const blob = props.blob;
    if (!blob || loadingTask) return;
    void (async () => {
      try {
        if (blob.size === 0 || blob.size > MAX_PDF_BYTES) throw new Error("PDF size outside preview limit");
        const data = new Uint8Array(await blob.arrayBuffer());
        if (disposed) return;
        const assets = new URL(`${import.meta.env.BASE_URL}pdfjs/${version}/`, window.location.href).href;
        loadingTask = getDocument({
          data,
          cMapUrl: `${assets}cmaps/`, cMapPacked: true,
          standardFontDataUrl: `${assets}standard_fonts/`,
          wasmUrl: `${assets}wasm/`, iccUrl: `${assets}iccs/`,
          enableXfa: false,
          // PDF.js 5.7 removed the eval compiler and its isEvalSupported option.
          // Display API does not instantiate the separate scripting manager.
          canvasMaxAreaInBytes: MAX_CANVAS_PIXELS * 4,
        });
        const loaded = await loadingTask.promise;
        if (!disposed) setDocument(loaded);
      } catch {
        if (!disposed) setState("error");
      }
    })();
  });

  createEffect(() => {
    const pdf = document();
    const current = pageNumber();
    const scale = zoom();
    const availableWidth = width();
    if (!pdf || !availableWidth) return;
    const request = ++generation;
    renderTask?.cancel();
    setState("rendering");
    setPageText("");
    // Cancelled renders must settle before their canvas can be reused.
    renderQueue = renderQueue.then(async () => {
      if (disposed || request !== generation) return;
      renderedPage?.cleanup();
      const page = await pdf.getPage(current);
      if (disposed || request !== generation) { page.cleanup(); return; }
      renderedPage = page;
      const natural = page.getViewport({ scale: 1 });
      const cssScale = Math.min(availableWidth / natural.width, 1.5) * scale;
      const display = page.getViewport({ scale: cssScale });
      const pixelRatio = Math.min(window.devicePixelRatio || 1, 2, 8192 / display.width, 8192 / display.height, Math.sqrt(MAX_CANVAS_PIXELS / (display.width * display.height)));
      const pixels = page.getViewport({ scale: cssScale * pixelRatio });
      canvas.width = Math.max(1, Math.floor(pixels.width));
      canvas.height = Math.max(1, Math.floor(pixels.height));
      canvas.style.width = `${display.width}px`;
      canvas.style.height = `${display.height}px`;
      renderTask = page.render({ canvas, viewport: pixels });
      await renderTask.promise;
      if (disposed || request !== generation) return;
      renderTask = undefined;
      setState("ready");
      // Plain text also makes the current PDF page accessible to screen readers.
      const text = await page.getTextContent().catch(() => null);
      if (!disposed && request === generation) {
        setPageText(text?.items.map((item) => "str" in item ? item.str : "").join(" ") ?? "");
      }
    }).catch(() => {
      if (!disposed && request === generation) setState("error");
    });
  });

  const resizeObserver = new ResizeObserver(() => setWidth(Math.max(1, viewport.clientWidth - 32)));
  createEffect(() => { resizeObserver.observe(viewport); });
  onCleanup(() => {
    disposed = true;
    generation += 1;
    window.clearTimeout(slowTimer);
    resizeObserver.disconnect();
    renderTask?.cancel();
    // destroy() also terminates this document's worker and outstanding decoders.
    void loadingTask?.destroy().catch(() => {});
    void renderQueue.finally(() => { canvas.width = 0; canvas.height = 0; });
  });

  const failed = () => props.loadError || state() === "error";
  return (
    <div class="public-community-document-preview">
      <div class="public-community-pdf-toolbar" aria-label={copy.file_preview}>
        <div class="public-community-pdf-controls">
        <button type="button" disabled={!document() || pageNumber() <= 1} onClick={() => { viewport.scrollTop = 0; setPageNumber(pageNumber() - 1); }}>{copy.pdf_previous}</button>
        <output data-testid="pdf-page-number" aria-live="polite">{document() ? pageLabel() : "—"}</output>
        <button type="button" disabled={!document() || pageNumber() >= document()!.numPages} onClick={() => { viewport.scrollTop = 0; setPageNumber(pageNumber() + 1); }}>{copy.pdf_next}</button>
        </div>
        <div class="public-community-pdf-controls">
        <button type="button" aria-label={copy.zoom_out} disabled={!document() || zoom() <= 0.5} onClick={() => setZoom(Math.max(0.5, zoom() - 0.25))}>−</button>
        <output>{Math.round(zoom() * 100)}%</output>
        <button type="button" aria-label={copy.zoom_in} disabled={!document() || zoom() >= 3} onClick={() => setZoom(Math.min(3, zoom() + 0.25))}>+</button>
        <button type="button" disabled={!document() || zoom() === 1} onClick={() => setZoom(1)}>{copy.fit_screen}</button>
        </div>
      </div>
      <div ref={viewport} class="public-community-pdf-viewport" aria-busy={!failed() && state() !== "ready"}>
        <canvas ref={canvas} hidden={failed() || state() !== "ready"} role="img" aria-label={pageLabel()} data-testid="community-pdf-canvas" />
        <p class="public-community-pdf-text">{pageText()}</p>
        <Show when={failed()} fallback={
          <Show when={state() !== "ready"}><div class="public-community-document-fallback" role="status">{slow() ? copy.pdf_slow : copy.pdf_preparing}</div></Show>
        }>
          <div class="public-community-document-fallback" role="alert"><strong>{copy.pdf_unsupported}</strong><span>{copy.pdf_fallback}</span></div>
        </Show>
      </div>
    </div>
  );
}
