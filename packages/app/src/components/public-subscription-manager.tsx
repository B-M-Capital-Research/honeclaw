import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import { CONTENT } from "@/lib/public-content";
import {
  listPublicSubscriptions,
  unsubscribePublicSubscription,
  updatePublicSubscription,
} from "@/lib/api";
import {
  formatScheduleSummary,
  formatScheduleTime,
  parseScheduleTime,
} from "@/lib/public-subscription-model";
import type { PublicSubscription } from "@/lib/types";

function ManageDialog(props: {
  subscription: PublicSubscription;
  saving: boolean;
  error?: string;
  onCancel: () => void;
  onSave: (patch: { name: string; task_prompt: string; hour: number; minute: number }) => void;
}) {
  const copy = () => CONTENT.chat_page.subscriptions;
  const [name, setName] = createSignal(props.subscription.name);
  const [prompt, setPrompt] = createSignal(props.subscription.task_prompt);
  const [time, setTime] = createSignal(
    formatScheduleTime(props.subscription.schedule.hour, props.subscription.schedule.minute),
  );
  const [localError, setLocalError] = createSignal<string | undefined>();

  const submit = (event: Event) => {
    event.preventDefault();
    const parsed = parseScheduleTime(time());
    if (!parsed) {
      setLocalError(copy().time_invalid);
      return;
    }
    if (!prompt().trim()) {
      setLocalError(copy().prompt_required);
      return;
    }
    setLocalError(undefined);
    props.onSave({
      name: name().trim() || props.subscription.name,
      task_prompt: prompt().trim(),
      ...parsed,
    });
  };

  return (
    <div class="public-subscription-dialog-backdrop" onClick={props.onCancel}>
      <form
        class="public-subscription-dialog"
        onClick={(event) => event.stopPropagation()}
        onSubmit={submit}
      >
        <header>
          <h2>{copy().manage_title}</h2>
          <button type="button" aria-label={copy().close} onClick={props.onCancel}>
            ×
          </button>
        </header>
        <label>
          <span>{copy().field_name}</span>
          <input value={name()} onInput={(event) => setName(event.currentTarget.value)} />
        </label>
        <label>
          <span>{copy().field_prompt}</span>
          {/* The prompt is the whole point of the push: give it room. */}
          <textarea
            rows={5}
            value={prompt()}
            onInput={(event) => setPrompt(event.currentTarget.value)}
          />
        </label>
        <label>
          <span>{copy().field_time}</span>
          <input
            type="time"
            value={time()}
            onInput={(event) => setTime(event.currentTarget.value)}
          />
        </label>
        <Show when={localError() ?? props.error}>
          {(message) => (
            <p class="public-subscription-error" role="alert">
              {message()}
            </p>
          )}
        </Show>
        <footer>
          <button type="button" onClick={props.onCancel}>
            {copy().cancel}
          </button>
          <button type="submit" class="is-primary" disabled={props.saving}>
            {props.saving ? copy().saving : copy().save}
          </button>
        </footer>
      </form>
    </div>
  );
}

export function PublicSubscriptionManager() {
  const copy = () => CONTENT.chat_page.subscriptions;
  const [items, setItems] = createSignal<PublicSubscription[]>([]);
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | undefined>();
  const [editing, setEditing] = createSignal<PublicSubscription | undefined>();
  const [saving, setSaving] = createSignal(false);
  const [busyId, setBusyId] = createSignal<string | undefined>();

  const active = createMemo(() => items().filter((item) => item.enabled));
  const stopped = createMemo(() => items().filter((item) => !item.enabled));

  const load = async () => {
    try {
      setItems(await listPublicSubscriptions());
      setError(undefined);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : copy().load_failed);
    } finally {
      setLoading(false);
    }
  };

  onMount(() => void load());

  const replace = (next: PublicSubscription) =>
    setItems((current) =>
      current.map((item) => (item.job_id === next.job_id ? next : item)),
    );

  const unsubscribe = async (item: PublicSubscription) => {
    setBusyId(item.job_id);
    try {
      const result = await unsubscribePublicSubscription(item.job_id);
      replace(result.subscription);
      setError(undefined);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : copy().action_failed);
    } finally {
      setBusyId(undefined);
    }
  };

  const resume = async (item: PublicSubscription) => {
    setBusyId(item.job_id);
    try {
      replace(await updatePublicSubscription(item.job_id, { enabled: true }));
      setError(undefined);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : copy().action_failed);
    } finally {
      setBusyId(undefined);
    }
  };

  const save = async (patch: {
    name: string;
    task_prompt: string;
    hour: number;
    minute: number;
  }) => {
    const target = editing();
    if (!target) return;
    setSaving(true);
    try {
      replace(await updatePublicSubscription(target.job_id, patch));
      setEditing(undefined);
      setError(undefined);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : copy().action_failed);
    } finally {
      setSaving(false);
    }
  };

  const row = (item: PublicSubscription) => (
    <li classList={{ "is-stopped": !item.enabled }}>
      <div class="public-subscription-main">
        <strong>{item.name}</strong>
        <small>{formatScheduleSummary(item.schedule)}</small>
        <em>{item.task_prompt}</em>
      </div>
      <div class="public-subscription-actions">
        <button type="button" onClick={() => setEditing(item)}>
          {copy().manage}
        </button>
        <Show
          when={item.enabled}
          fallback={
            <button
              type="button"
              disabled={busyId() === item.job_id}
              onClick={() => void resume(item)}
            >
              {copy().resume}
            </button>
          }
        >
          <button
            type="button"
            class="is-danger"
            disabled={busyId() === item.job_id}
            onClick={() => void unsubscribe(item)}
          >
            {copy().unsubscribe}
          </button>
        </Show>
      </div>
    </li>
  );

  return (
    <section class="public-subscription-manager">
      <header class="public-subscription-heading">
        <h2>{copy().title}</h2>
        <p>{copy().subtitle}</p>
      </header>

      <Show when={error()}>
        {(message) => (
          <p class="public-subscription-error" role="alert">
            {message()}
          </p>
        )}
      </Show>

      <Show
        when={!loading()}
        fallback={<p class="public-subscription-empty">{copy().loading}</p>}
      >
        <Show
          when={items().length > 0}
          fallback={<p class="public-subscription-empty">{copy().empty}</p>}
        >
          <ul class="public-subscription-list">
            <For each={active()}>{row}</For>
          </ul>
          {/* Stopped ones stay visible: unsubscribing is reversible, and a
              list that hides them looks like the push was deleted. */}
          <Show when={stopped().length > 0}>
            <h3 class="public-subscription-subheading">{copy().stopped_heading}</h3>
            <ul class="public-subscription-list">
              <For each={stopped()}>{row}</For>
            </ul>
          </Show>
        </Show>
      </Show>

      <Show when={editing()}>
        {(item) => (
          <ManageDialog
            subscription={item()}
            saving={saving()}
            error={error()}
            onCancel={() => setEditing(undefined)}
            onSave={save}
          />
        )}
      </Show>
    </section>
  );
}
