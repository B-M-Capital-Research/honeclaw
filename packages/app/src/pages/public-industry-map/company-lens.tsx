import { For, Show, createMemo } from "solid-js";

import { lensSummary } from "@/lib/industry-lens";
import { INFERRED_MEMBER_TIP, isInferredMember, subtypeOf } from "@/lib/industry-valuation";
import type { Industry, IndustryMember } from "@/lib/types";

import { MemberSubtypeSelect, AddMemberRow } from "./admin-editors";
import {
  FieldEditor,
  changePercent,
  marketCap,
  type DetailJump,
  type Editor,
} from "./shared";

/**
 * 公司卡：选中一家公司后出现在详情顶部——它在这一行赚什么钱、市值现价、本页哪几块提到了它，
 * 以及从它出发的「问 HONE」。退出视角就是把 URL 里的 symbol 去掉。
 */
export function CompanyLensCard(props: {
  industry: Industry;
  member: IndustryMember;
  onExit: () => void;
  onAsk: () => void;
  onLocate: () => void;
  jump: DetailJump;
}) {
  const subtype = createMemo(() => subtypeOf(props.industry, props.member.symbol));
  const summary = createMemo(() => lensSummary(props.industry, props.member));
  const total = () => {
    const counts = summary();
    return counts.signals + counts.watch + counts.variables + counts.sources;
  };
  return (
    <section class="industry-lens-card" id="company-lens" aria-label="公司视角">
      <div class="industry-lens-head">
        <div class="industry-lens-title">
          <span class="industry-lens-symbol">{props.member.symbol}</span>
          <span class="industry-lens-name">{props.member.name}</span>
          <Show when={subtype()}>
            {(item) => (
              <button
                type="button"
                class="industry-subtype-tag"
                classList={{ "is-inferred": isInferredMember(item(), props.member.symbol) }}
                title={
                  isInferredMember(item(), props.member.symbol)
                    ? INFERRED_MEMBER_TIP
                    : `查看子类型「${item().name}」怎么估值`
                }
                onClick={() => props.jump("valuation-card")}
              >
                {item().name}
              </button>
            )}
          </Show>
        </div>
        <button
          type="button"
          class="industry-btn industry-lens-exit"
          aria-label="退出公司视角"
          onClick={props.onExit}
        >
          退出公司视角
        </button>
      </div>
      <p class="industry-lens-role">{props.member.role}</p>
      <dl class="industry-lens-facts">
        <dt>市值</dt>
        <dd>{marketCap(props.member.market_cap)}</dd>
        <dt>现价</dt>
        <dd>
          <Show when={props.member.price != null} fallback="—">
            {props.member.price?.toFixed(2)}
            <span
              class="industry-change"
              classList={{ "is-down": (props.member.change_percent ?? 0) < 0 }}
            >
              {changePercent(props.member.change_percent)}
            </span>
          </Show>
        </dd>
      </dl>
      <Show
        when={total() > 0}
        fallback={
          <p class="industry-detail-note">
            底稿里还没有专门写到 {props.member.symbol} 的条目
            <Show when={subtype()}>{(item) => <>；先看子类型「{item().name}」的执行卡</>}</Show>。
          </p>
        }
      >
        <p class="industry-lens-summary">
          本页与它有关：
          <button type="button" class="industry-link" onClick={() => props.jump("changes")}>
            最近变化 {summary().signals}
          </button>
          <button type="button" class="industry-link" onClick={() => props.jump("watch")}>
            关注点 {summary().watch}
          </button>
          <button type="button" class="industry-link" onClick={() => props.jump("driver-chain")}>
            传导链变量 {summary().variables}
          </button>
          <button type="button" class="industry-link" onClick={() => props.jump("sources")}>
            来源 {summary().sources}
          </button>
        </p>
      </Show>
      <div class="industry-lens-actions">
        <button
          type="button"
          class="industry-btn is-primary"
          title="带着这一行的本体去开一轮前瞻估值"
          onClick={props.onAsk}
        >
          问 HONE：给 {props.member.symbol} 做前瞻估值
        </button>
        <button type="button" class="industry-btn" onClick={props.onLocate}>
          在公司表里定位
        </button>
      </div>
    </section>
  );
}

/** 相关公司表：按市值降序；每行「查看」进入公司视角、「问 HONE」开一轮估值；编辑态改位置、子类型、移出、加入。 */
export function MembersTable(props: {
  industry: Industry;
  editMode: boolean;
  editor: Editor;
  highlight: string | undefined;
  lensSymbol: string | undefined;
  onPickSubtype: (id: string) => void;
  onAsk: (member: IndustryMember) => void;
  onView: (symbol: string) => void;
}) {
  return (
    <table class="industry-members">
      <thead>
        <tr>
          <th>代码</th>
          <th>公司</th>
          <th>子类型</th>
          <th>市值（美元）</th>
          <th>现价</th>
          <th>在这一行的位置</th>
          <th>操作</th>
        </tr>
      </thead>
      <tbody>
        <For each={props.industry.members}>
          {(member) => (
            <tr
              data-symbol={member.symbol}
              classList={{
                "is-hit": props.highlight === member.symbol,
                "is-lens": props.lensSymbol === member.symbol,
              }}
            >
              <td class="industry-symbol">{member.symbol}</td>
              <td>{member.name}</td>
              <td class="industry-subtype-cell">
                <Show
                  when={props.editMode}
                  fallback={
                    <Show
                      when={subtypeOf(props.industry, member.symbol)}
                      fallback={<span class="industry-subtype-none">—</span>}
                    >
                      {(subtype) => (
                        <button
                          type="button"
                          class="industry-subtype-tag"
                          classList={{
                            "is-inferred": isInferredMember(subtype(), member.symbol),
                          }}
                          title={
                            isInferredMember(subtype(), member.symbol)
                              ? INFERRED_MEMBER_TIP
                              : `查看子类型「${subtype().name}」`
                          }
                          onClick={() => props.onPickSubtype(subtype().id)}
                        >
                          {subtype().name}
                        </button>
                      )}
                    </Show>
                  }
                >
                  <MemberSubtypeSelect
                    industry={props.industry}
                    symbol={member.symbol}
                    editor={props.editor}
                  />
                </Show>
              </td>
              <td>
                {marketCap(member.market_cap)}
                <Show when={member.market_cap_basis === "price_x_official_shares"}>
                  <span
                    class="industry-basis"
                    title="提供方的 sharesOutstanding 会整整落后一份申报，这里按最近一期定期报告封面上的官方股本重算；括号里是提供方原样的市值，便于与外部站点对照。"
                  >
                    官方股本口径
                    <Show when={member.provider_market_cap != null}>
                      {" · 提供方 "}
                      {marketCap(member.provider_market_cap)}
                    </Show>
                  </span>
                </Show>
              </td>
              <td>
                <Show when={member.price != null} fallback="—">
                  {member.price?.toFixed(2)}
                  <span
                    class="industry-change"
                    classList={{ "is-down": (member.change_percent ?? 0) < 0 }}
                  >
                    {changePercent(member.change_percent)}
                  </span>
                </Show>
              </td>
              <td class="industry-role">
                <Show when={props.editMode} fallback={member.role}>
                  <FieldEditor
                    ariaLabel={`${member.symbol} 在这一行的位置`}
                    value={member.role}
                    rows={2}
                    editor={props.editor}
                    onSave={(role) =>
                      props.editor.submit(props.industry.id, {
                        kind: "set_member_role",
                        symbol: member.symbol,
                        role,
                      })
                    }
                  />
                </Show>
              </td>
              <td>
                <div class="industry-members-actions">
                  <button
                    type="button"
                    class="industry-btn"
                    title="让整页围绕这家公司重排"
                    aria-label={`查看 ${member.symbol}`}
                    onClick={() => props.onView(member.symbol)}
                  >
                    查看
                  </button>
                  <button
                    type="button"
                    class="industry-btn"
                    title="带着这一行的本体去开一轮前瞻估值"
                    onClick={() => props.onAsk(member)}
                  >
                    问 HONE
                  </button>
                  <Show when={props.editMode}>
                    <button
                      type="button"
                      class="industry-btn is-danger"
                      disabled={!props.editor.canSave()}
                      onClick={() =>
                        void props.editor.submit(props.industry.id, {
                          kind: "remove_member",
                          symbol: member.symbol,
                        })
                      }
                    >
                      移出
                    </button>
                  </Show>
                </div>
              </td>
            </tr>
          )}
        </For>
      </tbody>
      <Show when={props.editMode}>
        <tfoot>
          <AddMemberRow industry={props.industry.id} editor={props.editor} />
        </tfoot>
      </Show>
    </table>
  );
}
