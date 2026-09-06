import { For, Show, createEffect, createSignal, on } from "solid-js";

import { mentionsMember, variableText } from "@/lib/industry-lens";
import { INFERRED_MEMBER_TIP } from "@/lib/industry-valuation";
import type {
  Industry,
  IndustryEditField,
  IndustryKeyVariable,
  IndustryMember,
  IndustryMethodRule,
  IndustryMethodology,
  IndustrySubtype,
  IndustryValuation,
  IndustryValuationListField,
  IndustryValuationTextField,
} from "@/lib/types";

import { SubtypeForm } from "./admin-editors";
import { AsOfTag, FieldEditor, ListEditor, RelatedTag, type Editor } from "./shared";

/** 研究底稿的几块：方法论条、可观测变量表、估值执行卡、底层估值逻辑。 */

export function VariablesTable(props: {
  variables: IndustryKeyVariable[];
  /** 公司视角下提到它的行打标（不重排：表格顺序就是传导链的顺序）。 */
  member?: Pick<IndustryMember, "symbol" | "name">;
  now?: Date;
}) {
  const dated = () => props.variables.some((variable) => variable.as_of);
  const related = (variable: IndustryKeyVariable) =>
    props.member ? mentionsMember(variableText(variable), props.member) : false;
  return (
    <table class="industry-variables">
      <thead>
        <tr>
          <th>可观测变量</th>
          <th>它在链条哪一环</th>
          <th>去哪取</th>
          <Show when={dated()}>
            <th>数字截至</th>
          </Show>
        </tr>
      </thead>
      <tbody>
        <For each={props.variables}>
          {(variable) => (
            <tr classList={{ "is-related": related(variable) }}>
              <td>
                {variable.name}
                <Show when={related(variable) && props.member}>
                  {(member) => <RelatedTag symbol={member().symbol} />}
                </Show>
              </td>
              <td>{variable.why}</td>
              <td class="industry-where">{variable.where}</td>
              <Show when={dated()}>
                <td class="industry-variable-asof">
                  <AsOfTag asOf={variable.as_of} now={props.now} prefix="" />
                </td>
              </Show>
            </tr>
          )}
        </For>
      </tbody>
    </table>
  );
}

/** 传导链与旧版锚：一段传导链、可观测变量表、倍数锚与常见错法；编辑态逐段就地改。 */
export function DriverChainBlock(props: {
  industry: Industry;
  editMode: boolean;
  editor: Editor;
  setField: (field: IndustryEditField, value: string) => Promise<boolean>;
  member?: Pick<IndustryMember, "symbol" | "name">;
  now?: Date;
}) {
  const logic = () => props.industry.ai_valuation_logic;
  return (
    <Show
      when={props.editMode}
      fallback={
        <Show
          when={logic().driver_chain}
          fallback={<p class="industry-detail-note">这一行的传导链尚未定稿。</p>}
        >
          <p class="industry-chain">{logic().driver_chain}</p>
          <Show when={logic().key_variables.length > 0}>
            <VariablesTable variables={logic().key_variables} member={props.member} now={props.now} />
          </Show>
          <dl class="industry-anchor">
            <dt>倍数锚</dt>
            <dd>{logic().multiple_anchor || "—"}</dd>
            <dt>这一行最常见的估值错法</dt>
            <dd>{logic().anti_pattern || "—"}</dd>
          </dl>
        </Show>
      }
    >
      <FieldEditor
        label="传导链"
        value={logic().driver_chain}
        rows={4}
        editor={props.editor}
        onSave={(value) => props.setField("driver_chain", value)}
      />
      <Show when={logic().key_variables.length > 0}>
        <p class="industry-detail-note">可观测变量表暂不在页面上改。</p>
        <VariablesTable variables={logic().key_variables} now={props.now} />
      </Show>
      <FieldEditor
        label="倍数锚（长版，研究台看）"
        value={logic().multiple_anchor}
        editor={props.editor}
        onSave={(value) => props.setField("multiple_anchor", value)}
      />
      <FieldEditor
        label="倍数锚（短版，每轮注入模型，110 字内）"
        value={logic().multiple_anchor_short ?? ""}
        rows={2}
        editor={props.editor}
        onSave={(value) => props.setField("multiple_anchor_short", value)}
      />
      <FieldEditor
        label="这一行最常见的估值错法（长版，研究台看）"
        value={logic().anti_pattern}
        editor={props.editor}
        onSave={(value) => props.setField("anti_pattern", value)}
      />
      <FieldEditor
        label="估值错法（短版，每轮注入模型，110 字内）"
        value={logic().anti_pattern_short ?? ""}
        rows={2}
        editor={props.editor}
        onSave={(value) => props.setField("anti_pattern_short", value)}
      />
    </Show>
  );
}

export const DOSSIER_IDS = ["dossier", "method", "valuation-card", "valuation-logic", "driver-chain"] as const;

/**
 * 完整研究底稿：方法论、这类公司怎么估值、底层估值逻辑、传导链，折在一个 <details> 里。
 * 用 <details> 而不是条件渲染：里面的编辑器草稿依赖组件实例存活，折叠不能把它们卸载。
 * 编辑态默认展开——管理员进来就是要改这些。
 */
export function ResearchDossier(props: {
  industry: Industry;
  valuation: IndustryValuation;
  methodology: IndustryMethodology;
  activeSubtype: IndustrySubtype | undefined;
  onSelectSubtype: (id: string) => void;
  editMode: boolean;
  editor: Editor;
  setField: (field: IndustryEditField, value: string) => Promise<boolean>;
  open: boolean;
  onToggle: (open: boolean) => void;
  jump: (id: string) => void;
  member?: Pick<IndustryMember, "symbol" | "name">;
  now?: Date;
}) {
  return (
    <details
      class="industry-details industry-dossier"
      id="dossier"
      open={props.open}
      onToggle={(event) => props.onToggle(event.currentTarget.open)}
    >
      <summary>
        完整研究底稿
        <span class="industry-dossier-hint">方法论 · 这类公司怎么估值 · 底层估值逻辑 · 传导链</span>
      </summary>
      <nav class="industry-dossier-nav" aria-label="底稿目录">
        <button type="button" onClick={() => props.jump("method")}>方法论</button>
        <button type="button" onClick={() => props.jump("valuation-card")}>这类公司怎么估值</button>
        <button type="button" onClick={() => props.jump("valuation-logic")}>底层估值逻辑</button>
        <button type="button" onClick={() => props.jump("driver-chain")}>传导链</button>
      </nav>
      <div id="method">
        <MethodologyStrip methodology={props.methodology} />
      </div>
      <h4 class="industry-dossier-title" id="valuation-card">这类公司怎么估值</h4>
      <p class="industry-detail-note">先在这里定位：公司在哪个子类型、用哪个前瞻财年与哪一族倍数、什么被禁止；再去看上游最近做了什么。</p>
      <ValuationCard
        industry={props.industry}
        valuation={props.valuation}
        active={props.activeSubtype}
        onSelect={props.onSelectSubtype}
        editMode={props.editMode}
        editor={props.editor}
      />
      <h4 class="industry-dossier-title" id="valuation-logic">底层估值逻辑</h4>
      <ValuationLogicBlock
        industry={props.industry}
        valuation={props.valuation}
        editMode={props.editMode}
        editor={props.editor}
      />
      <h4 class="industry-dossier-title" id="driver-chain">传导链与旧版锚（带日期的量）</h4>
      <DriverChainBlock
        industry={props.industry}
        editMode={props.editMode}
        editor={props.editor}
        setField={props.setField}
        member={props.member}
        now={props.now}
      />
    </details>
  );
}

/**
 * 方法论条：前瞻估值执行版的通用部分，所有行共用，所以只在详情区顶部放一次。
 * 旧后端没带方法论（执行规则为空）时整条不渲染，页面退回逐行内容。
 */
export function MethodologyStrip(props: { methodology: IndustryMethodology }) {
  const method = () => props.methodology;
  return (
    <Show when={method().execution_rules.length > 0}>
      <section class="industry-method" aria-label="估值方法论">
        <h4>HOne 前瞻估值执行版 {method().version}</h4>
        <Show when={method().positioning}>
          <p class="industry-method-lead">{method().positioning}</p>
        </Show>
        <Show when={method().demand_chain.length > 0}>
          <ol class="industry-method-chain" aria-label="需求链">
            <For each={method().demand_chain}>
              {(step) => (
                <li>
                  <span class="industry-method-step">{step}</span>
                </li>
              )}
            </For>
          </ol>
        </Show>
        <Show when={method().core_principle}>
          <p class="industry-callout">
            <span class="industry-callout-label">核心原则</span>
            {method().core_principle}
          </p>
        </Show>
        <details class="industry-details">
          <summary>通用执行规则</summary>
          <RuleTable head={["规则", "执行要求"]} rows={method().execution_rules} />
        </details>
        <Show when={method().output_fields.length > 0}>
          <details class="industry-details">
            <summary>强制输出字段</summary>
            <RuleTable head={["字段", "要求"]} rows={method().output_fields} />
          </details>
        </Show>
        <Show when={method().hindsight_error}>
          <p class="industry-callout is-warn">
            <span class="industry-callout-label">最常见的后视镜错误</span>
            {method().hindsight_error}
          </p>
        </Show>
      </section>
    </Show>
  );
}

/** 「规则 → 要求」两列表，执行规则与输出字段共用；在自己的容器里横向滚动。 */
export function RuleTable(props: { head: [string, string]; rows: IndustryMethodRule[] }) {
  return (
    <div class="industry-table-scroll">
      <table class="industry-rules">
        <thead>
          <tr>
            <th>{props.head[0]}</th>
            <th>{props.head[1]}</th>
          </tr>
        </thead>
        <tbody>
          <For each={props.rows}>
            {(row) => (
              <tr>
                <td>{row.rule}</td>
                <td>{row.requirement}</td>
              </tr>
            )}
          </For>
        </tbody>
      </table>
    </div>
  );
}

/**
 * 估值执行卡：这一行拆成哪些子类型，选中的那一个用什么主锚 / 次锚、适用什么阶段、
 * 有哪些成员；下面三行是行级锚——倍数上沿由什么决定、盈利上修期权在哪、什么被禁止。
 * 读者先在这里定位公司，再去看上游最近做了什么。
 */
export function ValuationCard(props: {
  industry: Industry;
  valuation: IndustryValuation;
  active: IndustrySubtype | undefined;
  onSelect: (id: string) => void;
  editMode: boolean;
  editor: Editor;
}) {
  // 表单种子：undefined 关着；null 新增；对象 = 预填编辑。
  const [seed, setSeed] = createSignal<IndustrySubtype | null | undefined>(undefined);
  const subtypes = () => props.valuation.subtypes;
  const anchor = () => props.valuation.anchor;
  const isActive = (subtype: IndustrySubtype) => props.active?.id === subtype.id;
  const memberCount = (subtype: IndustrySubtype) =>
    subtype.members.length;
  const setText = (field: IndustryValuationTextField, value: string) =>
    props.editor.submit(props.industry.id, { kind: "set_valuation_field", field, value });
  const setList = (field: IndustryValuationListField, items: string[]) =>
    props.editor.submit(props.industry.id, { kind: "set_valuation_list", field, items });
  const remove = async (subtype: IndustrySubtype) => {
    const confirmed = window.confirm(
      `确定移除子类型「${subtype.name}」？它的成员会回到未归入状态；这次移除会记进改动日志。`,
    );
    if (!confirmed) return;
    const ok = await props.editor.submit(props.industry.id, {
      kind: "remove_subtype",
      id: subtype.id,
    });
    if (ok && seed()?.id === subtype.id) setSeed(undefined);
  };
  // 编辑模式关掉时表单也收起，别把一张半填的表单留到读模式。
  createEffect(
    on(
      () => props.editMode,
      (enabled) => {
        if (!enabled) setSeed(undefined);
      },
      { defer: true },
    ),
  );
  return (
    <div class="industry-valuation-card">
      <Show when={subtypes().length === 0}>
        <p class="industry-detail-note">这一行尚未拆子类型，按下面的行级锚执行。</p>
      </Show>
      <Show when={subtypes().length > 0 || props.editMode}>
        <div class="industry-subtype-chips" aria-label="子类型">
          <For each={subtypes()}>
            {(subtype) => (
              <button
                type="button"
                class="industry-subtype-chip"
                classList={{ "is-active": isActive(subtype) }}
                aria-pressed={isActive(subtype)}
                onClick={() => props.onSelect(subtype.id)}
              >
                {subtype.name}
                <span class="industry-subtype-count">{memberCount(subtype)}</span>
              </button>
            )}
          </For>
          <Show when={props.editMode}>
            <button
              type="button"
              class="industry-btn"
              aria-expanded={seed() === null}
              onClick={() => setSeed((value) => (value === null ? undefined : null))}
            >
              {seed() === null ? "收起" : "新增子类型"}
            </button>
          </Show>
        </div>
      </Show>
      <Show when={props.active}>
        {(subtype) => (
          <div class="industry-subtype-card">
            <div class="industry-subtype-head">
              <strong>{subtype().name}</strong>
              <code class="industry-subtype-id">{subtype().id}</code>
              <Show when={props.editMode}>
                <span class="industry-subtype-actions">
                  <button
                    type="button"
                    class="industry-btn"
                    disabled={props.editor.busy()}
                    onClick={() => setSeed(subtype())}
                  >
                    编辑
                  </button>
                  <button
                    type="button"
                    class="industry-btn is-danger"
                    disabled={!props.editor.canSave()}
                    onClick={() => void remove(subtype())}
                  >
                    移除子类型
                  </button>
                </span>
              </Show>
            </div>
            <dl class="industry-subtype-grid">
              <dt>主锚</dt>
              <dd>{subtype().primary || "—"}</dd>
              <dt>次锚</dt>
              <dd>{subtype().secondary || "—"}</dd>
              <dt>适用阶段</dt>
              <dd>{subtype().when || "—"}</dd>
              <dt>提醒</dt>
              <dd>{subtype().note || "—"}</dd>
            </dl>
            <Show when={memberCount(subtype()) > 0}>
              <div class="industry-subtype-members" aria-label="成员">
                <For each={subtype().members}>
                  {(symbol) => <span class="industry-member-chip">{symbol}</span>}
                </For>
                <For each={subtype().inferred_members}>
                  {(symbol) => (
                    <span class="industry-member-chip is-inferred">
                      {symbol}
                      <span class="industry-inferred-tag" title={INFERRED_MEMBER_TIP}>
                        推断
                      </span>
                    </span>
                  )}
                </For>
              </div>
            </Show>
          </div>
        )}
      </Show>
      <Show when={props.editMode && seed() !== undefined}>
        <SubtypeForm
          industry={props.industry.id}
          seed={seed() ?? null}
          editor={props.editor}
          onSaved={(id) => {
            setSeed(undefined);
            props.onSelect(id);
          }}
          onCancel={() => setSeed(undefined)}
        />
      </Show>
      <Show
        when={props.editMode}
        fallback={
          <dl class="industry-anchor-rows">
            <dt>倍数上沿由</dt>
            <dd>{anchor().upper_range_drivers || "—"}</dd>
            <dt>盈利上修期权</dt>
            <dd>{anchor().revision_optionality || "—"}</dd>
            <dt>禁止</dt>
            <dd>
              <Show when={anchor().forbidden.length > 0} fallback="—">
                <ul class="industry-forbidden">
                  <For each={anchor().forbidden}>{(item) => <li>{item}</li>}</For>
                </ul>
              </Show>
            </dd>
          </dl>
        }
      >
        <FieldEditor
          label="倍数上沿由"
          value={anchor().upper_range_drivers}
          rows={2}
          editor={props.editor}
          onSave={(value) => setText("anchor.upper_range_drivers", value)}
        />
        <FieldEditor
          label="盈利上修期权"
          value={anchor().revision_optionality}
          rows={2}
          editor={props.editor}
          onSave={(value) => setText("anchor.revision_optionality", value)}
        />
        <ListEditor
          label="禁止（一行一条）"
          items={anchor().forbidden}
          rows={3}
          editor={props.editor}
          onSave={(items) => setList("anchor.forbidden", items)}
        />
      </Show>
    </div>
  );
}

/**
 * 底层估值逻辑：一段话、公式、未来 1–3 年先看什么；原文与倍数锚原文折叠在后面，
 * 读者要核出处时再展开。公式没有改动入口，改它要动底稿。
 */
export function ValuationLogicBlock(props: {
  industry: Industry;
  valuation: IndustryValuation;
  editMode: boolean;
  editor: Editor;
}) {
  const logic = () => props.valuation.logic;
  const anchor = () => props.valuation.anchor;
  const setText = (field: IndustryValuationTextField, value: string) =>
    props.editor.submit(props.industry.id, { kind: "set_valuation_field", field, value });
  const setList = (field: IndustryValuationListField, items: string[]) =>
    props.editor.submit(props.industry.id, { kind: "set_valuation_list", field, items });
  return (
    <div class="industry-valuation-logic">
      <Show
        when={props.editMode}
        fallback={
          <Show
            when={logic().summary}
            fallback={<p class="industry-detail-note">这一行的前瞻估值逻辑尚未定稿。</p>}
          >
            <p class="industry-valuation-summary">{logic().summary}</p>
          </Show>
        }
      >
        <FieldEditor
          label="一段话"
          value={logic().summary}
          rows={4}
          editor={props.editor}
          onSave={(value) => setText("logic.summary", value)}
        />
      </Show>
      <Show when={logic().formulas.length > 0}>
        <ul class="industry-formulas" aria-label="公式">
          <For each={logic().formulas}>
            {(item) => (
              <li>
                <code class="industry-formula">{item.formula}</code>
                <Show when={item.note}>
                  <span class="industry-formula-note">{item.note}</span>
                </Show>
              </li>
            )}
          </For>
        </ul>
      </Show>
      <Show
        when={props.editMode}
        fallback={
          <Show when={logic().forward_focus.length > 0}>
            <h4 class="industry-subhead">未来 1–3 年先看</h4>
            <ol class="industry-forward-focus">
              <For each={logic().forward_focus}>{(item) => <li>{item}</li>}</For>
            </ol>
          </Show>
        }
      >
        <ListEditor
          label="未来 1–3 年先看（一行一条）"
          items={logic().forward_focus}
          editor={props.editor}
          onSave={(items) => setList("logic.forward_focus", items)}
        />
      </Show>
      <details class="industry-details">
        <summary>原文</summary>
        <Show
          when={props.editMode}
          fallback={
            <Show
              when={logic().paragraphs.length > 0}
              fallback={<p class="industry-detail-note">尚未定稿。</p>}
            >
              <For each={logic().paragraphs}>
                {(paragraph) => <p class="industry-paragraph">{paragraph}</p>}
              </For>
            </Show>
          }
        >
          <ListEditor
            label="原文（一行一段）"
            items={logic().paragraphs}
            rows={8}
            editor={props.editor}
            onSave={(items) => setList("logic.paragraphs", items)}
          />
        </Show>
      </details>
      <details class="industry-details">
        <summary>倍数锚原文</summary>
        <Show
          when={props.editMode}
          fallback={
            <Show
              when={anchor().paragraphs.length > 0}
              fallback={<p class="industry-detail-note">尚未定稿。</p>}
            >
              <For each={anchor().paragraphs}>
                {(paragraph) => <p class="industry-paragraph">{paragraph}</p>}
              </For>
            </Show>
          }
        >
          <ListEditor
            label="倍数锚原文（一行一段）"
            items={anchor().paragraphs}
            rows={8}
            editor={props.editor}
            onSave={(items) => setList("anchor.paragraphs", items)}
          />
        </Show>
      </details>
    </div>
  );
}
