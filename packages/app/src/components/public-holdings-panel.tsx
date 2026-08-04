// 「我的 · 自选与持仓」面板：列表 + 点击弹出的四个气泡动作
// （调整 / 问问新闻 / 问问估值 / 问问财报）+ 顶部添加入口。
// 持仓一律以仓位占比展示；不填占比即为自选。

import { createEffect, createMemo, createSignal, For, onCleanup, Show } from "solid-js";
import { CONTENT } from "@/lib/public-content";
import { useNavigate } from "@solidjs/router";
import {
  createPublicHolding,
  deletePublicHolding,
  getPublicPortfolio,
  updatePublicHolding,
  type PublicHolding,
} from "@/lib/api";
import {
  canAddHolding,
  formatHoldingCost,
  formatHoldingWeight,
  holdingAskPrompt,
  totalHoldingWeight,
  validateHoldingForm,
  type HoldingAskKind,
  type HoldingRow,
} from "@/pages/public-holdings-model";

type EditorState =
  | { mode: "create" }
  | { mode: "edit"; row: HoldingRow }
  | null;

const askActions = (): Array<{ kind: HoldingAskKind; label: string }> => [
  { kind: "news", label: CONTENT.chat_page.holdings.ask_news },
  { kind: "valuation", label: CONTENT.chat_page.holdings.ask_valuation },
  { kind: "earnings", label: CONTENT.chat_page.holdings.ask_earnings },
];

function HoldingEditor(props: {
  state: Exclude<EditorState, null>;
  saving: boolean;
  error?: string;
  onCancel: () => void;
  onSubmit: (input: { symbol: string; name: string; weight: string; avgCost: string }) => void;
  onDelete?: () => void;
}) {
  const initial = props.state.mode === "edit" ? props.state.row : undefined;
  const [symbol, setSymbol] = createSignal(initial?.symbol ?? "");
  const [name, setName] = createSignal(initial?.name ?? "");
  const [weight, setWeight] = createSignal(
    initial?.weight != null ? String(Number(initial.weight.toFixed(2))) : "",
  );
  const [avgCost, setAvgCost] = createSignal(
    initial?.avg_cost != null ? String(initial.avg_cost) : "",
  );

  createEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") props.onCancel();
    };
    document.addEventListener("keydown", onKeyDown);
    onCleanup(() => document.removeEventListener("keydown", onKeyDown));
  });

  return (
    <div class="public-holding-modal-overlay" onClick={props.onCancel}>
      <div class="public-holding-modal" onClick={(event) => event.stopPropagation()}>
        <header>
          <strong>{props.state.mode === "create"
              ? CONTENT.chat_page.holdings.add_title
              : CONTENT.chat_page.holdings.edit_title.replace("{symbol}", props.state.row.symbol)}</strong>
          <button type="button" aria-label={CONTENT.chat_page.holdings.close} onClick={props.onCancel}>×</button>
        </header>
        <form
          onSubmit={(event) => {
            event.preventDefault();
            props.onSubmit({ symbol: symbol(), name: name(), weight: weight(), avgCost: avgCost() });
          }}
        >
          <label>
            <span>{CONTENT.chat_page.holdings.ticker}</span>
            <input
              value={symbol()}
              disabled={props.state.mode === "edit"}
              placeholder={CONTENT.chat_page.holdings.ticker_eg}
              onInput={(event) => setSymbol(event.currentTarget.value)}
            />
          </label>
          <label>
            <span>{CONTENT.chat_page.holdings.company}<em>{CONTENT.chat_page.holdings.optional}</em></span>
            <input
              value={name()}
              placeholder={CONTENT.chat_page.holdings.company_eg}
              onInput={(event) => setName(event.currentTarget.value)}
            />
          </label>
          <div class="public-holding-form-row">
            <label>
              <span>{CONTENT.chat_page.holdings.weight}<em>{CONTENT.chat_page.holdings.optional}</em></span>
              <input
                value={weight()}
                inputmode="decimal"
                placeholder={CONTENT.chat_page.holdings.weight_hint}
                onInput={(event) => setWeight(event.currentTarget.value)}
              />
            </label>
            <label>
              <span>{CONTENT.chat_page.holdings.cost}<em>{CONTENT.chat_page.holdings.optional}</em></span>
              <input
                value={avgCost()}
                inputmode="decimal"
                placeholder={CONTENT.chat_page.holdings.cost_eg}
                onInput={(event) => setAvgCost(event.currentTarget.value)}
              />
            </label>
          </div>
          <p class="public-holding-form-hint">{CONTENT.chat_page.holdings.form_hint}</p>
          <Show when={props.error}>
            <p class="public-holding-form-error" role="alert">{props.error}</p>
          </Show>
          <footer>
            <Show when={props.onDelete}>
              {(onDelete) => (
                <button type="button" class="is-danger" onClick={() => onDelete()()}>{CONTENT.chat_page.holdings.delete}</button>
              )}
            </Show>
            <span class="public-holding-form-spacer" />
            <button type="button" onClick={props.onCancel}>{CONTENT.chat_page.holdings.cancel}</button>
            <button type="submit" class="is-primary" disabled={props.saving}>
              {props.saving ? CONTENT.chat_page.holdings.saving : CONTENT.chat_page.holdings.save}
            </button>
          </footer>
        </form>
      </div>
    </div>
  );
}

export function PublicHoldingsPanel() {
  const navigate = useNavigate();
  const [rows, setRows] = createSignal<PublicHolding[]>([]);
  const [limit, setLimit] = createSignal(50);
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string>();
  const [openMenu, setOpenMenu] = createSignal<string>();
  const [editor, setEditor] = createSignal<EditorState>(null);
  const [saving, setSaving] = createSignal(false);
  const [formError, setFormError] = createSignal<string>();

  const load = async () => {
    setLoading(true);
    setError(undefined);
    try {
      const payload = await getPublicPortfolio();
      setRows(payload.holdings ?? []);
      setLimit(payload.limit ?? 50);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : CONTENT.chat_page.holdings.load_failed);
    } finally {
      setLoading(false);
    }
  };

  createEffect(() => {
    void load();
  });

  // 点击空白处收起气泡菜单。
  createEffect(() => {
    if (!openMenu()) return;
    const close = () => setOpenMenu(undefined);
    document.addEventListener("click", close);
    onCleanup(() => document.removeEventListener("click", close));
  });

  const positions = createMemo(() => rows().filter((row) => !row.tracking_only));
  const totalWeight = createMemo(() => totalHoldingWeight(rows() as HoldingRow[]));

  const ask = (row: HoldingRow, kind: HoldingAskKind) => {
    setOpenMenu(undefined);
    navigate(`/chat?q=${encodeURIComponent(holdingAskPrompt(row, kind))}`);
  };

  const submit = async (input: { symbol: string; name: string; weight: string; avgCost: string }) => {
    const state = editor();
    if (!state) return;
    const parsed = validateHoldingForm(input);
    if (!parsed.ok) {
      setFormError(parsed.error);
      return;
    }
    setSaving(true);
    setFormError(undefined);
    try {
      const payload =
        state.mode === "create"
          ? await createPublicHolding(parsed.value)
          : await updatePublicHolding(state.row.symbol, parsed.value);
      setRows(payload.holdings ?? []);
      setLimit(payload.limit ?? 50);
      setEditor(null);
    } catch (cause) {
      setFormError(cause instanceof Error ? cause.message : CONTENT.chat_page.holdings.save_failed);
    } finally {
      setSaving(false);
    }
  };

  const remove = async (symbol: string) => {
    setSaving(true);
    setFormError(undefined);
    try {
      const payload = await deletePublicHolding(symbol);
      setRows(payload.holdings ?? []);
      setEditor(null);
    } catch (cause) {
      setFormError(cause instanceof Error ? cause.message : CONTENT.chat_page.holdings.delete_failed);
    } finally {
      setSaving(false);
    }
  };

  return (
    <section class="public-workspace-panel public-holdings-panel" aria-label={CONTENT.chat_page.holdings.panel_title}>
      <header class="public-holdings-head">
        <div>
          <h2>{CONTENT.chat_page.holdings.panel_title}</h2>
          <p>
            {CONTENT.chat_page.holdings.summary_line
              .replace("{positions}", String(positions().length))
              .replace("{weight}", totalWeight().toFixed(1))
              .replace("{rows}", String(rows().length))
              .replace("{limit}", String(limit()))}
          </p>
        </div>
        <button
          type="button"
          class="public-workspace-primary-action"
          disabled={!canAddHolding(rows().length, limit())}
          title={
            canAddHolding(rows().length, limit())
              ? ""
              : CONTENT.chat_page.holdings.max_hint.replace("{max}", String(limit()))
          }
          onClick={() => {
            setFormError(undefined);
            setEditor({ mode: "create" });
          }}
        >
          ＋ {CONTENT.chat_page.holdings.add}
        </button>
      </header>

      <Show when={!loading()} fallback={<div class="public-workspace-state">{CONTENT.chat_page.holdings.loading}</div>}>
        <Show when={!error()} fallback={
          <div class="public-workspace-state is-error" role="alert">
            <p>{error()}</p>
            <button type="button" onClick={() => void load()}>{CONTENT.chat_page.holdings.reload}</button>
          </div>
        }>
          <Show
            when={rows().length > 0}
            fallback={
              <div class="public-holdings-empty">
                <strong>{CONTENT.chat_page.holdings.empty_title}</strong>
                <p>{CONTENT.chat_page.holdings.empty_hint}</p>
                <button type="button" onClick={() => setEditor({ mode: "create" })}>{CONTENT.chat_page.holdings.add_first}</button>
              </div>
            }
          >
            <ul class="public-holdings-list">
              <For each={rows()}>
                {(row) => (
                  <li classList={{ "is-open": openMenu() === row.symbol }}>
                    <button
                      type="button"
                      class="public-holding-row"
                      aria-expanded={openMenu() === row.symbol}
                      onClick={(event) => {
                        event.stopPropagation();
                        setOpenMenu((current) => (current === row.symbol ? undefined : row.symbol));
                      }}
                    >
                      <span class="public-holding-main">
                        <strong>{row.symbol}</strong>
                        <Show when={row.name}><small>{row.name}</small></Show>
                      </span>
                      <span class="public-holding-meta">
                        <b classList={{ "is-watch": row.tracking_only }}>{formatHoldingWeight(row as HoldingRow)}</b>
                        <Show when={formatHoldingCost(row as HoldingRow)}>
                          {(cost) => <em>{cost()}</em>}
                        </Show>
                      </span>
                    </button>
                    <Show when={openMenu() === row.symbol}>
                      <div class="public-holding-bubble" role="menu" onClick={(event) => event.stopPropagation()}>
                        <button
                          type="button"
                          role="menuitem"
                          onClick={() => {
                            setOpenMenu(undefined);
                            setFormError(undefined);
                            setEditor({ mode: "edit", row: row as HoldingRow });
                          }}
                        >
                          {CONTENT.chat_page.holdings.edit}
                        </button>
                        <For each={askActions()}>
                          {(action) => (
                            <button
                              type="button"
                              role="menuitem"
                              onClick={() => ask(row as HoldingRow, action.kind)}
                            >
                              {action.label}
                            </button>
                          )}
                        </For>
                      </div>
                    </Show>
                  </li>
                )}
              </For>
            </ul>
          </Show>
        </Show>
      </Show>

      <Show when={editor()}>
        {(state) => (
          <HoldingEditor
            state={state()}
            saving={saving()}
            error={formError()}
            onCancel={() => setEditor(null)}
            onSubmit={submit}
            onDelete={
              state().mode === "edit"
                ? () => void remove((state() as { mode: "edit"; row: HoldingRow }).row.symbol)
                : undefined
            }
          />
        )}
      </Show>
    </section>
  );
}
