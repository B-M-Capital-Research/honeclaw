import { Title } from "@solidjs/meta";
import { A } from "@solidjs/router";
import {
  For,
  Show,
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
} from "solid-js";
import { PublicWorkspaceShell } from "@/components/public-workspace-shell";
import { DataCenterScene } from "@/components/data-center-scene";
import {
  DATA_CENTER_ZONES,
  industryHref,
  type DataCenterZoneId,
} from "@/lib/data-center-model";
import "./public-data-center.css";

export default function PublicDataCenterPage() {
  const [selected, setSelected] = createSignal<DataCenterZoneId | null>(null);
  const current = createMemo(() =>
    DATA_CENTER_ZONES.find((zone) => zone.id === selected()),
  );
  let dialog!: HTMLDialogElement;
  let trigger: HTMLButtonElement | undefined;
  const close = () => {
    dialog.close();
    setSelected(null);
    trigger?.focus({ preventScroll: true });
  };
  const select = (id: DataCenterZoneId, element: HTMLButtonElement) => {
    trigger = element;
    setSelected(id);
  };
  createEffect(() => {
    if (!current()) return;
    dialog.showModal();
    const previous = document.documentElement.style.overflow;
    document.documentElement.style.overflow = "hidden";
    onCleanup(() => {
      document.documentElement.style.overflow = previous;
    });
  });

  return (
    <>
      <Title>3D 数据中心 · HONE</Title>
      <PublicWorkspaceShell active="research" topbarLabel="AI 基础设施">
        <div class="dc-page">
          <nav class="dc-breadcrumb" aria-label="面包屑">
            <A href="/chat">投资助手</A>
            <span>/</span>
            <span>探索 AI 基础设施</span>
          </nav>
          <header class="dc-header">
            <div>
              <div class="dc-eyebrow">INSIDE THE AI DATA CENTER</div>
              <h1>
                3D 数据中心<span class="dc-title-dot">.</span>
              </h1>
              <p>走进一座 AI 数据中心，看懂算力背后的产业连接。</p>
            </div>
            <A href="/industry-map" class="dc-all-industries">
              完整行业分析 <span aria-hidden="true">↗</span>
            </A>
          </header>
          <DataCenterScene selected={selected()} onSelect={select} />
          <section class="dc-explore" aria-labelledby="dc-explore-title">
            <div class="dc-explore-heading">
              <h2 id="dc-explore-title">一座机房，六个观察切口</h2>
              <span>从物理设施，到 AI 服务</span>
            </div>
            <div class="dc-zone-list">
              <For each={DATA_CENTER_ZONES}>
                {(zone, index) => (
                  <button
                    type="button"
                    class="dc-zone-card"
                    style={{ "--zone-color": zone.color }}
                    aria-haspopup="dialog"
                    aria-expanded={selected() === zone.id}
                    onClick={(event) => select(zone.id, event.currentTarget)}
                  >
                    <span class="dc-zone-number">0{index() + 1}</span>
                    <span class="dc-zone-mark">{zone.shortLabel}</span>
                    <strong>{zone.title}</strong>
                    <span class="dc-zone-subtitle">{zone.subtitle}</span>
                    <span class="dc-zone-arrow" aria-hidden="true">
                      ↗
                    </span>
                  </button>
                )}
              </For>
            </div>
          </section>
          <footer class="dc-footnote">
            <span class="dc-footnote-mark" aria-hidden="true">
              ◇
            </span>
            <p>
              产业结构示意，设备比例与位置经过简化。软件为逻辑服务层；行业详情沿用
              HONE 的研究底稿。
            </p>
            <span>HONE · INDUSTRY EXPLORER</span>
          </footer>
        </div>
      </PublicWorkspaceShell>
      <dialog
        ref={dialog}
        class="dc-detail"
        aria-labelledby="dc-detail-title"
        onCancel={(event) => {
          event.preventDefault();
          close();
        }}
        onClose={() => {
          if (selected()) setSelected(null);
        }}
        onClick={(event) => {
          const rect = dialog.getBoundingClientRect();
          if (
            event.target === dialog &&
            (event.clientX < rect.left ||
              event.clientX > rect.right ||
              event.clientY < rect.top ||
              event.clientY > rect.bottom)
          )
            close();
        }}
      >
        <Show when={current()}>
          {(zone) => (
            <div style={{ "--zone-color": zone().color }}>
              <div class="dc-detail-top">
                <span>探索基础设施 / {zone().shortLabel}</span>
                <button
                  type="button"
                  class="dc-detail-close"
                  aria-label="关闭行业浮窗"
                  onClick={close}
                  autofocus
                >
                  ×
                </button>
              </div>
              <span class="dc-detail-mark">{zone().shortLabel}</span>
              <h2 id="dc-detail-title">{zone().title}</h2>
              <p class="dc-detail-subtitle">{zone().subtitle}</p>
              <div class="dc-location">
                <span aria-hidden="true">⌖</span>
                {zone().location}
              </div>
              <p class="dc-description">{zone().description}</p>
              <div class="dc-components">
                <For each={zone().components}>
                  {(component) => <span>{component}</span>}
                </For>
              </div>
              <h3>研究时，关注什么</h3>
              <ul class="dc-focus-list">
                <For each={zone().focus}>{(item) => <li>{item}</li>}</For>
              </ul>
              <div class="dc-detail-links">
                <h3>继续看完整行业分析</h3>
                <For each={zone().industries}>
                  {(industry) => (
                    <A
                      href={industryHref(industry.id)}
                      onClick={() => dialog.close()}
                    >
                      <span>{industry.name}</span>
                      <span aria-hidden="true">↗</span>
                    </A>
                  )}
                </For>
                <p>登录后可阅读行业逻辑、关键变量和相关公司。</p>
              </div>
            </div>
          )}
        </Show>
      </dialog>
    </>
  );
}
