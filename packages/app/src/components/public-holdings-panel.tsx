// 「我的 · 自选与持仓」面板：列表 + 点击弹出的四个气泡动作
// （调整 / 问问新闻 / 问问估值 / 问问财报）+ 顶部添加入口。
// 持仓一律以仓位占比展示；不填占比即为自选。

import { createEffect, createMemo, createSignal, For, onCleanup, Show } from "solid-js";
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

const ASK_ACTIONS: Array<{ kind: HoldingAskKind; label: string }> = [
  { kind: "news", label: "问问新闻" },
  { kind: "valuation", label: "问问估值" },
  { kind: "earnings", label: "问问财报" },
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
          <strong>{props.state.mode === "create" ? "添加自选或持仓" : `调整 ${props.state.row.symbol}`}</strong>
          <button type="button" aria-label="关闭" onClick={props.onCancel}>×</button>
        </header>
        <form
          onSubmit={(event) => {
            event.preventDefault();
            props.onSubmit({ symbol: symbol(), name: name(), weight: weight(), avgCost: avgCost() });
          }}
        >
          <label>
            <span>股票代码</span>
            <input
              value={symbol()}
              disabled={props.state.mode === "edit"}
              placeholder="例如 AAPL"
              onInput={(event) => setSymbol(event.currentTarget.value)}
            />
          </label>
          <label>
            <span>公司名称<em>可选</em></span>
            <input
              value={name()}
              placeholder="例如 苹果"
              onInput={(event) => setName(event.currentTarget.value)}
            />
          </label>
          <div class="public-holding-form-row">
            <label>
              <span>仓位占比 %<em>可选</em></span>
              <input
                value={weight()}
                inputmode="decimal"
                placeholder="留空 = 只加自选"
                onInput={(event) => setWeight(event.currentTarget.value)}
              />
            </label>
            <label>
              <span>成本价<em>可选</em></span>
              <input
                value={avgCost()}
                inputmode="decimal"
                placeholder="例如 180.25"
                onInput={(event) => setAvgCost(event.currentTarget.value)}
              />
            </label>
          </div>
          <p class="public-holding-form-hint">不填仓位占比就只加入观察列表；填了占比才算持仓。</p>
          <Show when={props.error}>
            <p class="public-holding-form-error" role="alert">{props.error}</p>
          </Show>
          <footer>
            <Show when={props.onDelete}>
              {(onDelete) => (
                <button type="button" class="is-danger" onClick={() => onDelete()()}>删除</button>
              )}
            </Show>
            <span class="public-holding-form-spacer" />
            <button type="button" onClick={props.onCancel}>取消</button>
            <button type="submit" class="is-primary" disabled={props.saving}>
              {props.saving ? "保存中…" : "保存"}
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
      setError(cause instanceof Error ? cause.message : "自选与持仓加载失败");
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
      setFormError(cause instanceof Error ? cause.message : "保存失败，请稍后再试");
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
      setFormError(cause instanceof Error ? cause.message : "删除失败，请稍后再试");
    } finally {
      setSaving(false);
    }
  };

  return (
    <section class="public-workspace-panel public-holdings-panel" aria-label="我的自选与持仓">
      <header class="public-holdings-head">
        <div>
          <h2>我的自选与持仓</h2>
          <p>
            {positions().length} 只持仓 · 合计 {totalWeight().toFixed(1)}% · 共 {rows().length}/{limit()} 条
          </p>
        </div>
        <button
          type="button"
          class="public-workspace-primary-action"
          disabled={!canAddHolding(rows().length, limit())}
          title={canAddHolding(rows().length, limit()) ? "" : `最多 ${limit()} 条`}
          onClick={() => {
            setFormError(undefined);
            setEditor({ mode: "create" });
          }}
        >
          ＋ 添加
        </button>
      </header>

      <Show when={!loading()} fallback={<div class="public-workspace-state">正在加载自选与持仓…</div>}>
        <Show when={!error()} fallback={
          <div class="public-workspace-state is-error" role="alert">
            <p>{error()}</p>
            <button type="button" onClick={() => void load()}>重新加载</button>
          </div>
        }>
          <Show
            when={rows().length > 0}
            fallback={
              <div class="public-holdings-empty">
                <strong>还没有自选或持仓</strong>
                <p>添加后，HONE 的新闻、财报与推送都会围绕这些标的展开。</p>
                <button type="button" onClick={() => setEditor({ mode: "create" })}>添加第一只</button>
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
                          调整
                        </button>
                        <For each={ASK_ACTIONS}>
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
