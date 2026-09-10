import { For, Show, createMemo } from "solid-js";

import { recentChanges, type ChangeItem } from "@/lib/industry-brief";
import { rankByMention, signalText, sourceText } from "@/lib/industry-lens";
import {
  askHoneHref,
  askSignalPrompt,
  askSourcePrompt,
  relationLabel,
  valuationOf,
} from "@/lib/industry-valuation";
import type { Industry } from "@/lib/types";

import { SignalForm } from "./admin-editors";
import {
  AsOfTag,
  FieldEditor,
  LatestEditor,
  RelatedTag,
  upstreamSignals,
  type Editor,
  type Lens,
} from "./shared";

const asSignal = (item: ChangeItem) => (item.kind === "signal" ? item.signal : undefined);
const asSource = (item: ChangeItem) => (item.kind === "source" ? item.source : undefined);

/**
 * 「最近变化」：带日期的上游动作与来源按时间倒序排在一起，每条后面接一个带上下文的「问 HONE」。
 * 读模式由底稿派生；编辑态退回逐条上游信号的编辑器（最近动作、截至日期、移除、新增），
 * 因为「最近变化」本身不是一张可以直接改的表。
 */
export function RecentChanges(props: {
  industry: Industry;
  editMode: boolean;
  editor: Editor;
  lens: Lens;
  onAsk: (href: string) => void;
  now?: Date;
}) {
  const changes = createMemo(() => recentChanges(props.industry, props.now));
  const ranked = createMemo(() =>
    rankByMention(
      changes().items,
      (item) => (item.kind === "signal" ? signalText(item.signal) : sourceText(item.source)),
      props.lens.member(),
    ),
  );
  const ask = (item: ChangeItem) => {
    const focus = props.lens.member();
    const prompt =
      item.kind === "signal"
        ? askSignalPrompt(props.industry.name, item.signal, focus)
        : askSourcePrompt(props.industry.name, item.source, focus);
    props.onAsk(askHoneHref(prompt));
  };
  return (
    <section class="industry-changes" id="changes" aria-labelledby="industry-changes-title">
      <div class="industry-section-head">
        <h3 id="industry-changes-title">最近变化</h3>
        <span class="industry-section-sub">哪些外部变化会影响业绩</span>
      </div>
      <Show
        when={!props.editMode}
        fallback={<SignalsEditor industry={props.industry} editor={props.editor} />}
      >
        <Show when={valuationOf(props.industry).upstream_summary}>
          <p class="industry-upstream-summary">{valuationOf(props.industry).upstream_summary}</p>
        </Show>
        <Show
          when={ranked().length > 0}
          fallback={<p class="industry-detail-note">底稿里还没有带日期的上游动作或来源。</p>}
        >
          <ol class="industry-change-list">
            <For each={ranked()}>
              {(row) => (
                <li
                  class="industry-change"
                  classList={{ "is-related": row.related, "is-source": row.item.kind === "source" }}
                >
                  <div class="industry-change-head">
                    <Show when={asSignal(row.item)}>
                      {(signal) => (
                        <>
                          <strong>{signal().symbol}</strong>
                          <Show when={signal().name}>
                            <span class="industry-change-name">{signal().name}</span>
                          </Show>
                          <Show when={signal().relation}>
                            <span class="industry-relation">{relationLabel(signal().relation)}</span>
                          </Show>
                        </>
                      )}
                    </Show>
                    <Show when={asSource(row.item)}>
                      {(source) => (
                        <>
                          <strong>{source().house}</strong>
                          <span class="industry-relation is-source">来源</span>
                        </>
                      )}
                    </Show>
                    <AsOfTag asOf={row.item.asOf} now={props.now} />
                    <Show when={row.related ? props.lens.member() : undefined}>
                      {(member) => <RelatedTag symbol={member().symbol} />}
                    </Show>
                  </div>
                  <p class="industry-change-text">
                    {row.item.kind === "signal" ? row.item.signal.latest : row.item.source.takeaway}
                  </p>
                  <Show when={asSignal(row.item)}>
                    {(signal) => (
                      <Show when={signal().why || (signal().pull ?? []).length > 0}>
                        <details class="industry-details industry-change-more">
                          <summary>为什么重要 · 去取什么</summary>
                          <Show when={signal().why}>
                            <p>{signal().why}</p>
                          </Show>
                          <Show when={(signal().pull ?? []).length > 0}>
                            <ul class="industry-signal-pull">
                              <For each={signal().pull}>{(item) => <li>{item}</li>}</For>
                            </ul>
                          </Show>
                        </details>
                      </Show>
                    )}
                  </Show>
                  <Show when={asSource(row.item)}>
                    {(source) => (
                      <p class="industry-change-link">
                        <a href={source().url} target="_blank" rel="noreferrer">
                          {source().title}
                        </a>
                      </p>
                    )}
                  </Show>
                  <div class="industry-change-actions">
                    <button
                      type="button"
                      class="industry-btn"
                      title="带着这条变化去问：它影响谁、怎么传导、该核什么"
                      onClick={() => ask(row.item)}
                    >
                      问 HONE
                    </button>
                  </div>
                </li>
              )}
            </For>
          </ol>
          <Show when={changes().rest > 0}>
            <p class="industry-detail-note">
              其余 {changes().rest} 条带日期的来源按时间排在下方「研报与数据来源」里。
            </p>
          </Show>
        </Show>
        <Show when={changes().undatedSignals.length > 0}>
          <p class="industry-detail-note">
            还没有带日期动作的上游：
            {changes()
              .undatedSignals.map((signal) => `${signal.symbol}（${relationLabel(signal.relation)}）`)
              .join("、")}
          </p>
        </Show>
      </Show>
    </section>
  );
}

/** 编辑态：逐条上游信号——头部、最近动作编辑器、为什么、去取什么、移除；末尾新增表单。 */
function SignalsEditor(props: { industry: Industry; editor: Editor }) {
  return (
    <>
      <p class="industry-detail-note">
        编辑态按上游信号逐条改：每季财报后更新「最近动作」与截至日期；读者看到的「最近变化」由这里和来源按日期自动排出。
      </p>
      <FieldEditor
        label="上游信号（需求怎么从终端任务传到本行；不放最新一季数字）"
        value={valuationOf(props.industry).upstream_summary ?? ""}
        rows={4}
        editor={props.editor}
        onSave={(value) =>
          props.editor.submit(props.industry.id, {
            kind: "set_valuation_field",
            field: "upstream_summary",
            value,
          })
        }
      />
      <Show
        when={upstreamSignals(props.industry).length > 0}
        fallback={<p class="industry-detail-note">尚未定稿。</p>}
      >
        <ul class="industry-signals">
          <For each={upstreamSignals(props.industry)}>
            {(signal) => (
              <li>
                <div class="industry-signal-head">
                  <strong>{signal.symbol}</strong>
                  <Show when={signal.name}>
                    <span class="industry-signal-name">{signal.name}</span>
                  </Show>
                  <Show when={signal.relation}>
                    <span class="industry-relation">{relationLabel(signal.relation)}</span>
                  </Show>
                  <Show when={signal.cadence}>
                    <span class="industry-cadence">{signal.cadence}</span>
                  </Show>
                  <button
                    type="button"
                    class="industry-btn is-danger"
                    disabled={!props.editor.canSave()}
                    onClick={() =>
                      void props.editor.submit(props.industry.id, {
                        kind: "remove_upstream_signal",
                        symbol: signal.symbol,
                      })
                    }
                  >
                    移除
                  </button>
                </div>
                <LatestEditor
                  symbol={signal.symbol}
                  latest={signal.latest ?? ""}
                  asOf={signal.latest_as_of ?? ""}
                  editor={props.editor}
                  onSave={(latest, asOf) =>
                    props.editor.submit(props.industry.id, {
                      kind: "set_upstream_latest",
                      symbol: signal.symbol,
                      latest,
                      as_of: asOf,
                    })
                  }
                />
                <Show when={signal.why}>
                  <p>{signal.why}</p>
                </Show>
                <Show when={(signal.pull ?? []).length > 0}>
                  <ul class="industry-signal-pull">
                    <For each={signal.pull}>{(item) => <li>{item}</li>}</For>
                  </ul>
                </Show>
              </li>
            )}
          </For>
        </ul>
      </Show>
      <SignalForm industry={props.industry.id} editor={props.editor} />
    </>
  );
}
