/**
 * ResearchPreview
 *
 * SolidJS 版 Markdown 渲染组件，参考 BamangResearch/bamang-markdown/src/components/Preview.tsx
 *
 * 渲染管线：
 *   markdown string
 *     → marked()         Markdown → HTML，自定义 code/blockquote/text renderer
 *     → DOMPurify.sanitize()  XSS 净化
 *     → innerHTML        写入隐藏 div
 *     → paginateContent()    按 A4 高度分页
 *     → mermaid.render()     渲染流程图（分页后）
 *
 * PDF 导出：
 *   html2canvas(每页 .rp-page) → jsPDF.addImage() → pdf.save()
 */

import { Show, createEffect, createSignal, onCleanup, type Component } from "solid-js"
import { marked, type Renderer } from "marked"
import DOMPurify from "dompurify"
import hljs from "highlight.js"
import mermaid from "mermaid"
import "highlight.js/styles/github.css"
import "./research-preview.css"
import { RESEARCH } from "@/lib/admin-content/research"
import { useLocale } from "@/lib/i18n"

// ── marked 配置 ───────────────────────────────────────────────────────────────

marked.setOptions({ breaks: true, gfm: true })

const renderer: Partial<Renderer> = {
  // 代码块：mermaid 特殊处理，其余用 hljs 高亮
  code({ text, lang }) {
    if (lang === "mermaid") {
      const id = "mermaid-" + Math.random().toString(36).slice(2, 9)
      return `<div class="mermaid" id="${id}">${text}</div>`
    }
    if (lang && hljs.getLanguage(lang)) {
      try {
        const highlighted = hljs.highlight(text, { language: lang }).value
        return `<pre><code class="hljs language-${lang}">${highlighted}</code></pre>`
      } catch {
        // fallthrough
      }
    }
    return `<pre><code class="hljs">${hljs.highlightAuto(text).value}</code></pre>`
  },

  // 引用块：单换行转双换行
  blockquote({ text }) {
    const processed = text.replace(/<br>/g, "<br><br>")
    return `<blockquote>\n${processed}</blockquote>\n`
  },
}

marked.use({ renderer })

// ── Mermaid 初始化 ────────────────────────────────────────────────────────────

mermaid.initialize({
  startOnLoad: false,
  theme: "default",
  securityLevel: "loose",
})

// ── A4 分页常量 ────────────────────────────────────────────────────────────────

// 297mm 总高 - 20mm 页眉 - 20mm 页脚 = 257mm 内容区
// 内容区 padding 上 12mm + 下 20mm；实际可用 ≈ 225mm
// 1mm ≈ 3.7795px；保守取 200mm ≈ 756px
const PAGE_CONTENT_HEIGHT_PX = 200 * 3.7795

// ── PDF 导出 ──────────────────────────────────────────────────────────────────

async function exportToPdf(companyName: string) {
  // 动态 import 避免首屏加载 jspdf/html2canvas
  const [{ default: jsPDF }, { default: html2canvas }] = await Promise.all([
    import("jspdf"),
    import("html2canvas"),
  ])

  const pages = document.querySelectorAll<HTMLElement>(".rp-page")
  if (pages.length === 0) return

  const pdf = new jsPDF({ orientation: "portrait", unit: "mm", format: "a4" })

  for (let i = 0; i < pages.length; i++) {
    const page = pages[i]
    const canvas = await html2canvas(page, {
      scale: 2,
      useCORS: true,
      logging: false,
      backgroundColor: "#ffffff",
    })
    const imgData = canvas.toDataURL("image/jpeg", 0.88)
    if (i > 0) pdf.addPage()
    pdf.addImage(imgData, "JPEG", 0, 0, 210, 297, undefined, "FAST")
  }

  pdf.save(`${companyName}${RESEARCH.preview.pdf_filename_suffix}.pdf`)
}

// ── 组件 Props ────────────────────────────────────────────────────────────────

interface ResearchPreviewProps {
  markdown: string
  companyName: string
}

// ── 主组件 ───────────────────────────────────────────────────────────────────

const ResearchPreview: Component<ResearchPreviewProps> = (props) => {
  // 隐藏 div：用于测量 DOM 高度后分页
  let measureRef: HTMLDivElement | undefined
  const [pages, setPages] = createSignal<string[]>([])
  const [exporting, setExporting] = createSignal(false)

  // Tab 状态：预览 / 源代码
  const [tab, setTab] = createSignal<"preview" | "source">("preview")

  // 本地可编辑 Markdown，初始值来自 props
  const [editMarkdown, setEditMarkdown] = createSignal(props.markdown)

  // 当外部 props.markdown 更新时（首次加载）同步到本地
  createEffect(() => {
    setEditMarkdown(props.markdown)
  })

  // 保证每次 render 使用唯一 counter，避免 mermaid ID 冲突
  let mermaidCounter = 0

  createEffect(() => {
    // 同时追踪 editMarkdown 和 tab，这样切换到预览时自动重新渲染
    const md = editMarkdown()
    const currentTab = tab()
    if (currentTab !== "preview" || !md || !measureRef) return

    // 1. Markdown → HTML → XSS 净化
    const rawHtml = marked(md) as string
    const cleanHtml = DOMPurify.sanitize(rawHtml, {
      ADD_TAGS: ["iframe"],
      ADD_ATTR: ["allow", "allowfullscreen", "frameborder", "scrolling"],
    })
    measureRef.innerHTML = cleanHtml

    // 2. 分页
    const pageList = paginateContent(measureRef)

    // 3. 用分页结果更新 state
    setPages(pageList)

    // 4. 分页渲染完成后，渲染 mermaid（DOM 已更新时触发）
    //    使用 queueMicrotask 等待 SolidJS DOM 更新完成
    queueMicrotask(() => {
      renderMermaidInPages()
    })
  })

  onCleanup(() => {
    setPages([])
  })

  // ── 分页算法（参考 BamangResearch/Preview.tsx paginateContent） ────────────

  function paginateContent(container: HTMLDivElement): string[] {
    const children = Array.from(container.children) as HTMLElement[]
    const newPages: string[] = []
    let currentPage: HTMLElement[] = []
    let currentHeight = 0

    for (const element of children) {
      // 临时挂载到 document.body 来测量真实高度
      const tempWrap = document.createElement("div")
      tempWrap.style.cssText =
        "position:absolute;visibility:hidden;width:170mm;padding:0;margin:0;line-height:1.8;font-size:18px;"
      const clone = element.cloneNode(true) as HTMLElement
      tempWrap.appendChild(clone)
      document.body.appendChild(tempWrap)

      const style = window.getComputedStyle(clone)
      const marginTop = parseFloat(style.marginTop) || 0
      const marginBottom = parseFloat(style.marginBottom) || 0
      const elementHeight = tempWrap.offsetHeight + marginTop + marginBottom

      document.body.removeChild(tempWrap)

      // 根据元素类型决定安全阈值
      const tag = element.tagName
      const isMermaid = element.classList.contains("mermaid")
      let safeH = PAGE_CONTENT_HEIGHT_PX * 0.88
      if (tag === "TABLE") safeH = PAGE_CONTENT_HEIGHT_PX * 0.70
      else if (isMermaid || tag === "PRE") safeH = PAGE_CONTENT_HEIGHT_PX * 0.75
      else if (/^H[1-6]$/.test(tag)) safeH = PAGE_CONTENT_HEIGHT_PX * 0.92

      if (currentHeight + elementHeight > safeH && currentPage.length > 0) {
        // 保存当前页
        const pageDiv = document.createElement("div")
        currentPage.forEach((el) => pageDiv.appendChild(el.cloneNode(true)))
        newPages.push(pageDiv.innerHTML)
        currentPage = [element]
        currentHeight = elementHeight
      } else {
        currentPage.push(element)
        currentHeight += elementHeight
      }
    }

    // 最后一页
    if (currentPage.length > 0) {
      const pageDiv = document.createElement("div")
      currentPage.forEach((el) => pageDiv.appendChild(el.cloneNode(true)))
      newPages.push(pageDiv.innerHTML)
    }

    return newPages.length > 0 ? newPages : [container.innerHTML]
  }

  // ── Mermaid 渲染 ───────────────────────────────────────────────────────────

  function renderMermaidInPages() {
    const mermaidDivs = document.querySelectorAll<HTMLElement>(".rp-page .mermaid")
    mermaidDivs.forEach((el) => {
      mermaidCounter++
      const id = `rp-mermaid-${Date.now()}-${mermaidCounter}`
      el.id = id
      const code = el.textContent?.trim() ?? ""
      if (!code) return

      mermaid
        .render(`${id}-svg`, code)
        .then(({ svg }) => {
          el.innerHTML = svg
        })
        .catch((err) => {
          console.warn("Mermaid render error:", err)
          el.innerHTML = `<pre style="color:#f44336;padding:12px;background:#ffebee;border-left:4px solid #f44336;border-radius:6px;font-size:13px;">${RESEARCH.preview.mermaid_error_prefix}${String(err)}</pre>`
        })
    })
  }

  // ── 今日日期 ───────────────────────────────────────────────────────────────
  const today = () => {
    const loc = useLocale() === "zh" ? "zh-CN" : "en-US"
    return new Date().toLocaleDateString(loc, {
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
    })
  }

  // ── 渲染 ──────────────────────────────────────────────────────────────────

  return (
    <div class="research-preview-wrap">
      {/* 工具栏 */}
      <div class="research-preview-toolbar">
        <span class="research-preview-toolbar-title">
          {props.companyName}{RESEARCH.preview.title_suffix}
        </span>

        {/* Tab 切换 */}
        <div class="rp-tab-bar">
          <button
            type="button"
            class={`rp-tab${tab() === "preview" ? " rp-tab-active" : ""}`}
            onClick={() => setTab("preview")}
          >
            {RESEARCH.preview.tab_preview}
          </button>
          <button
            type="button"
            class={`rp-tab${tab() === "source" ? " rp-tab-active" : ""}`}
            onClick={() => setTab("source")}
          >
            {RESEARCH.preview.tab_source}
          </button>
        </div>

        <div class="research-preview-toolbar-actions">
          <Show when={tab() === "preview"}>
            <button
              type="button"
              disabled={exporting()}
              onClick={async () => {
                setExporting(true)
                try {
                  await exportToPdf(props.companyName)
                } finally {
                  setExporting(false)
                }
              }}
              style={{
                "font-size": "12px",
                "padding": "4px 12px",
                "border-radius": "5px",
                "border": "1px solid #d0d0d0",
                "background": exporting() ? "#e0e0e0" : "#fff",
                "color": exporting() ? "#999" : "#333",
                "cursor": exporting() ? "not-allowed" : "pointer",
                "transition": "all 0.15s",
              }}
            >
              {exporting() ? RESEARCH.preview.exporting_button : RESEARCH.preview.export_button}
            </button>
          </Show>
        </div>
      </div>

      {/* 隐藏的 measure div，用于分页计算 */}
      <div ref={measureRef} style={{ display: "none" }} />

      {/* 源代码编辑器 */}
      <Show when={tab() === "source"}>
        <div class="rp-source-editor">
          <textarea
            class="rp-source-textarea"
            value={editMarkdown()}
            onInput={(e) => setEditMarkdown(e.currentTarget.value)}
            spellcheck={false}
            placeholder={RESEARCH.preview.source_placeholder}
          />
        </div>
      </Show>

      {/* 预览滚动区：每页一张 A4 卡片 */}
      <Show when={tab() === "preview"}>
        <div class="research-preview-scroll">
          {pages().length === 0 ? (
            <div class="research-preview-loading">{RESEARCH.preview.rendering}</div>
          ) : (
            pages().map((pageHtml, i) => (
              <div class="rp-page" data-page={i + 1}>
                {/* 水印 */}
                <div class="rp-watermark">
                  <span class="rp-watermark-text">Hone Financial Research</span>
                </div>
                {/* 页眉 */}
                <div class="rp-header">
                  <span class="rp-header-left">{props.companyName}{RESEARCH.preview.header_suffix}</span>
                  <span class="rp-header-right">
                    <span class="rp-header-brand">Hone Financial</span>
                    <span class="rp-header-date">{today()}</span>
                  </span>
                </div>
                {/* 内容 */}
                <div
                  class="rp-content"
                  // eslint-disable-next-line solid/no-innerhtml
                  innerHTML={pageHtml}
                />
                {/* 页脚 */}
                <div class="rp-footer">
                  <span class="rp-footer-left">{RESEARCH.preview.footer_disclaimer}</span>
                  <span class="rp-footer-right">
                    {i + 1} / {pages().length}
                  </span>
                </div>
              </div>
            ))
          )}
        </div>
      </Show>
    </div>
  )
}

export default ResearchPreview
