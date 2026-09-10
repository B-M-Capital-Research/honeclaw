import { For, Show, createEffect, createMemo, createSignal, on } from "solid-js";

import { rankByMention, sourceText, watchText } from "@/lib/industry-lens";
import { askHoneHref, askWatchPrompt, valuationOf } from "@/lib/industry-valuation";
import type { Industry, IndustryCoreWatch } from "@/lib/types";

import { SourceForm, WatchForm } from "./admin-editors";
import { AsOfTag, FieldEditor, RelatedTag, type Editor, type Lens } from "./shared";

/**
 * 「接下来重点看什么」：关注点按公司视角前置，每条带数字截至日与一个「问 HONE」；
 * 长段的「为什么看它」折起来。编辑态逐条改（含 as_of）、移除、新增。
 */
export function WatchList(props: {
  industry: Industry;
  editMode: boolean;
  editor: Editor;
  lens: Lens;
  onAsk: (href: string) => void;
  now?: Date;
}) {
  const ranked = createMemo(() =>
    rankByMention(props.industry.core_watch ?? [], watchText, props.lens.member()),
  );
  const ask = (watch: IndustryCoreWatch) =>
    props.onAsk(askHoneHref(askWatchPrompt(props.industry.name, watch, props.lens.member())));
  return (
    <section class="industry-watch-section" id="watch" aria-labelledby="industry-watch-title">
      <div class="industry-section-head">
        <h3 id="industry-watch-title">接下来重点看什么</h3>
        <span class="industry-section-sub">下一次验证在哪里、多久看一次</span>
      </div>
      <Show
        when={ranked().length > 0}
        fallback={<p class="industry-detail-note">尚未定稿。</p>}
      >
        <ul class="industry-watch">
          <For each={ranked()}>
            {(row) => (
              <li class="industry-watch-item" classList={{ "is-related": row.related }}>
                <Show
                  when={!props.editMode}
                  fallback={
                    <WatchEditor industry={props.industry.id} watch={row.item} editor={props.editor} />
                  }
                >
                  <div class="industry-watch-head">
                    <strong>{row.item.what}</strong>
                    <Show when={row.item.cadence}>
                      <span class="industry-cadence">{row.item.cadence}</span>
                    </Show>
                    <AsOfTag asOf={row.item.as_of} now={props.now} prefix="数字截至" />
                    <Show when={row.related ? props.lens.member() : undefined}>
                      {(member) => <RelatedTag symbol={member().symbol} />}
                    </Show>
                  </div>
                  <Show when={row.item.why}>
                    <details class="industry-details industry-watch-more">
                      <summary>为什么看它</summary>
                      <p>{row.item.why}</p>
                    </details>
                  </Show>
                  <div class="industry-change-actions">
                    <button
                      type="button"
                      class="industry-btn"
                      title="问最近一期读数、相对上一期的变化，以及要不要改估值结论"
                      onClick={() => ask(row.item)}
                    >
                      问 HONE
                    </button>
                  </div>
                </Show>
              </li>
            )}
          </For>
        </ul>
      </Show>
      <Show when={props.editMode}>
        <WatchForm industry={props.industry.id} editor={props.editor} />
      </Show>
    </section>
  );
}

/** 一条关注点的就地编辑：标题、为什么、频率、数字截至日，整条存成一个 set_watch；按原标题定位。 */
function WatchEditor(props: { industry: string; watch: IndustryCoreWatch; editor: Editor }) {
  const [what, setWhat] = createSignal(props.watch.what);
  const [why, setWhy] = createSignal(props.watch.why);
  const [cadence, setCadence] = createSignal(props.watch.cadence);
  const [asOf, setAsOf] = createSignal(props.watch.as_of ?? "");
  createEffect(
    on(
      () => [props.watch.what, props.watch.why, props.watch.cadence, props.watch.as_of ?? ""],
      ([nextWhat, nextWhy, nextCadence, nextAsOf]) => {
        setWhat(nextWhat);
        setWhy(nextWhy);
        setCadence(nextCadence);
        setAsOf(nextAsOf);
      },
      { defer: true },
    ),
  );
  const dirty = () =>
    what().trim() !== props.watch.what.trim() ||
    why().trim() !== props.watch.why.trim() ||
    cadence().trim() !== props.watch.cadence.trim() ||
    asOf().trim() !== (props.watch.as_of ?? "").trim();
  const save = () =>
    props.editor.submit(props.industry, {
      kind: "set_watch",
      what: props.watch.what,
      watch: { what: what().trim(), why: why().trim(), cadence: cadence().trim(), as_of: asOf().trim() },
    });
  return (
    <div class="industry-field industry-watch-editor">
      <div class="industry-watch-editor-head">
        <input
          class="industry-input"
          aria-label="关注点标题"
          value={what()}
          disabled={props.editor.busy()}
          onInput={(event) => setWhat(event.currentTarget.value)}
        />
        <button
          type="button"
          class="industry-btn is-danger"
          disabled={!props.editor.canSave()}
          onClick={() =>
            void props.editor.submit(props.industry, { kind: "remove_watch", what: props.watch.what })
          }
        >
          移除
        </button>
      </div>
      <textarea
        class="industry-textarea"
        rows={4}
        aria-label={`${props.watch.what} 为什么看它`}
        value={why()}
        disabled={props.editor.busy()}
        onInput={(event) => setWhy(event.currentTarget.value)}
      />
      <div class="industry-watch-editor-meta">
        <label>
          频率
          <input
            class="industry-input"
            aria-label={`${props.watch.what} 频率`}
            value={cadence()}
            disabled={props.editor.busy()}
            onInput={(event) => setCadence(event.currentTarget.value)}
          />
        </label>
        <label>
          数字截至
          <input
            class="industry-input"
            aria-label={`${props.watch.what} 数字截至`}
            placeholder="2026-08-26"
            value={asOf()}
            disabled={props.editor.busy()}
            onInput={(event) => setAsOf(event.currentTarget.value)}
          />
        </label>
      </div>
      <div class="industry-field-actions">
        <button
          type="button"
          class="industry-btn is-primary"
          disabled={!props.editor.canSave() || !dirty() || what().trim() === ""}
          onClick={() => void save()}
        >
          保存
        </button>
        <Show when={dirty()}>
          <span class="industry-field-dirty">未保存</span>
        </Show>
        <Show when={dirty() && props.editor.noteMissing()}>
          <span class="industry-form-hint">先在面板顶部填改动说明</span>
        </Show>
      </div>
    </div>
  );
}

/** 研报与数据来源：按底稿顺序（公司视角下相关的前置），每条带日期；编辑态可移除、新增。 */
export function SourcesList(props: {
  industry: Industry;
  editMode: boolean;
  editor: Editor;
  lens: Lens;
  now?: Date;
}) {
  const ranked = createMemo(() =>
    rankByMention(props.industry.sources ?? [], sourceText, props.editMode ? undefined : props.lens.member()),
  );
  return (
    <section class="industry-sources-section" id="sources" aria-labelledby="industry-sources-title">
      <h3 id="industry-sources-title">研报与数据来源</h3>
      <Show
        when={props.editMode}
        fallback={
          <Show when={valuationOf(props.industry).sources_note}>
            <p class="industry-sources-note">{valuationOf(props.industry).sources_note}</p>
          </Show>
        }
      >
        <FieldEditor
          label="来源说明（这一行看哪些研报与数据、怎么核）"
          value={valuationOf(props.industry).sources_note ?? ""}
          rows={3}
          editor={props.editor}
          onSave={(value) =>
            props.editor.submit(props.industry.id, {
              kind: "set_valuation_field",
              field: "sources_note",
              value,
            })
          }
        />
      </Show>
      <Show when={ranked().length > 0} fallback={<p class="industry-detail-note">尚未定稿。</p>}>
        <ul class="industry-sources">
          <For each={ranked()}>
            {(row) => (
              <li classList={{ "is-related": row.related }}>
                <Show when={props.editMode}>
                  <button
                    type="button"
                    class="industry-btn is-danger industry-item-remove"
                    disabled={!props.editor.canSave()}
                    onClick={() =>
                      void props.editor.submit(props.industry.id, {
                        kind: "remove_source",
                        url: row.item.url,
                      })
                    }
                  >
                    移除
                  </button>
                </Show>
                <a href={row.item.url} target="_blank" rel="noreferrer">
                  {row.item.house}｜{row.item.title}
                </a>
                <AsOfTag asOf={row.item.date} now={props.now} prefix="" staleText="较早" />
                <Show when={row.related ? props.lens.member() : undefined}>
                  {(member) => <RelatedTag symbol={member().symbol} />}
                </Show>
                <p>{row.item.takeaway}</p>
              </li>
            )}
          </For>
        </ul>
      </Show>
      <Show when={props.editMode}>
        <SourceForm industry={props.industry.id} editor={props.editor} />
      </Show>
    </section>
  );
}
