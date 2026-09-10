import { For, Show, createEffect, createSignal, on } from "solid-js";

import { briefToForm, type IndustryBriefView } from "@/lib/industry-brief";
import { askHoneHref, askNextStepPrompt } from "@/lib/industry-valuation";
import type { Industry } from "@/lib/types";

import { AsOfTag, splitLines, type DetailJump, type Editor } from "./shared";

/**
 * 「当前重点」：每个行业打开先看到的一块。有管理员写的简报就用它（一句问题、为什么是现在、
 * 下一次先看什么）；没有就从底稿派生（典型 State、最新两条变化、第一条关注点），并标明是自动归纳。
 * 编辑态直接改简报本身。
 */
export function IndustryBriefSection(props: {
  industry: Industry;
  view: IndustryBriefView | undefined;
  editMode: boolean;
  editor: Editor;
  onAsk: (href: string) => void;
  jump: DetailJump;
  now?: Date;
}) {
  const askStep = (step: string) =>
    props.onAsk(askHoneHref(askNextStepPrompt(props.industry.name, props.view?.lead ?? "", step)));
  return (
    <Show when={props.editMode || props.view}>
      <section class="industry-brief" id="brief" aria-labelledby="industry-brief-title">
        <div class="industry-section-head">
          <h3 id="industry-brief-title">当前重点</h3>
          <Show when={props.view?.source === "derived"}>
            <span
              class="industry-brief-derived"
              title="底稿没写简报；这一块由典型 State、最新动作与第一条关注点自动归纳"
            >
              底稿自动归纳
            </span>
          </Show>
          <AsOfTag asOf={props.view?.asOf} now={props.now} />
        </div>
        <Show
          when={!props.editMode}
          fallback={<BriefForm industry={props.industry} editor={props.editor} />}
        >
          <Show when={props.view}>
            {(view) => (
              <>
                <Show when={view().source === "brief" && view().lead}>
                  <p class="industry-brief-lead">{view().lead}</p>
                </Show>
                <For each={view().body}>{(paragraph) => <p class="industry-brief-body">{paragraph}</p>}</For>
                <Show when={view().changes.length > 0}>
                  <ul class="industry-brief-changes" aria-label="最新变化">
                    <For each={view().changes}>
                      {(item) => (
                        <li>
                          <strong>
                            {item.kind === "signal" ? item.signal.symbol : item.source.house}
                          </strong>
                          <AsOfTag asOf={item.asOf} now={props.now} />
                          <span class="industry-brief-change-text">
                            {item.kind === "signal" ? item.signal.latest : item.source.takeaway}
                          </span>
                        </li>
                      )}
                    </For>
                  </ul>
                </Show>
                <Show when={view().next.length > 0}>
                  <div class="industry-brief-next">
                    <span class="industry-brief-next-label">下一次先看</span>
                    <ol>
                      <For each={view().next}>
                        {(step) => (
                          <li>
                            <span>{step}</span>
                            <button
                              type="button"
                              class="industry-btn"
                              title="把这一步交给 HONE：给数据、来源与对结论的影响"
                              onClick={() => askStep(step)}
                            >
                              问 HONE
                            </button>
                          </li>
                        )}
                      </For>
                    </ol>
                    <button type="button" class="industry-link" onClick={() => props.jump("watch")}>
                      全部关注点
                    </button>
                  </div>
                </Show>
              </>
            )}
          </Show>
        </Show>
      </section>
    </Show>
  );
}

/** 简报表单：四个字段一起存成一个 set_brief；有简报时可以清空回到自动归纳。 */
function BriefForm(props: { industry: Industry; editor: Editor }) {
  const seed = () => briefToForm(props.industry.brief);
  const [question, setQuestion] = createSignal(seed().question);
  const [body, setBody] = createSignal(seed().body);
  const [next, setNext] = createSignal(seed().next);
  const [asOf, setAsOf] = createSignal(seed().as_of);
  createEffect(
    on(
      () => props.industry.brief,
      () => {
        const value = seed();
        setQuestion(value.question);
        setBody(value.body);
        setNext(value.next);
        setAsOf(value.as_of);
      },
      { defer: true },
    ),
  );
  const dirty = () => {
    const value = seed();
    return (
      question().trim() !== value.question.trim() ||
      body().trim() !== value.body.trim() ||
      splitLines(next()).join("\n") !== value.next ||
      asOf().trim() !== value.as_of.trim()
    );
  };
  const valid = () => question().trim() !== "" && asOf().trim() !== "";
  const save = () =>
    props.editor.submit(props.industry.id, {
      kind: "set_brief",
      brief: {
        question: question().trim(),
        body: body().trim(),
        next: splitLines(next()),
        as_of: asOf().trim(),
      },
    });
  return (
    <div class="industry-brief-form">
      <p class="industry-detail-note">
        简报是读者打开这一行先看到的一块；没写时页面用典型 State、最新动作与第一条关注点自动归纳。截至日期是这份判断的日期，不是今天。
      </p>
      <label class="industry-field">
        <span class="industry-field-label">现在值得研究的问题（一句）</span>
        <textarea
          class="industry-textarea"
          rows={2}
          aria-label="简报问题"
          value={question()}
          disabled={props.editor.busy()}
          onInput={(event) => setQuestion(event.currentTarget.value)}
        />
      </label>
      <label class="industry-field">
        <span class="industry-field-label">为什么是现在（一段，空行分段）</span>
        <textarea
          class="industry-textarea"
          rows={4}
          aria-label="简报正文"
          value={body()}
          disabled={props.editor.busy()}
          onInput={(event) => setBody(event.currentTarget.value)}
        />
      </label>
      <label class="industry-field">
        <span class="industry-field-label">接下来要确认什么（一行一条，建议以日期或事件开头）</span>
        <textarea
          class="industry-textarea"
          rows={3}
          aria-label="简报下一步"
          placeholder="9 月下旬 MU 财报：HBM 是否售罄"
          value={next()}
          disabled={props.editor.busy()}
          onInput={(event) => setNext(event.currentTarget.value)}
        />
      </label>
      <label class="industry-brief-asof">
        截至
        <input
          class="industry-input"
          aria-label="简报截至日期"
          placeholder="2026-09-02"
          value={asOf()}
          disabled={props.editor.busy()}
          onInput={(event) => setAsOf(event.currentTarget.value)}
        />
      </label>
      <div class="industry-field-actions">
        <button
          type="button"
          class="industry-btn is-primary"
          disabled={!props.editor.canSave() || !dirty() || !valid()}
          onClick={() => void save()}
        >
          保存简报
        </button>
        <Show when={props.industry.brief}>
          <button
            type="button"
            class="industry-btn is-danger"
            disabled={!props.editor.canSave()}
            onClick={() => void props.editor.submit(props.industry.id, { kind: "clear_brief" })}
          >
            清空简报
          </button>
        </Show>
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
