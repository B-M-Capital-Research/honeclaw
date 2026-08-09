import { CONTENT } from "@/lib/public-content";
import type { PublicSubscription } from "@/lib/types";

/** `06:55`, not `6:55` — a schedule people scan should line up in a column. */
export function formatScheduleTime(hour: number, minute: number): string {
  const pad = (value: number) => String(Math.max(0, Math.floor(value))).padStart(2, "0");
  return `${pad(hour)}:${pad(minute)}`;
}

const WEEKDAY_LABELS = () => CONTENT.chat_page.subscriptions.weekdays;

export function formatScheduleSummary(schedule: PublicSubscription["schedule"]): string {
  const time = formatScheduleTime(schedule.hour, schedule.minute);
  const repeat = (schedule.repeat ?? "").trim().toLowerCase();
  if (repeat === "weekly") {
    const labels = WEEKDAY_LABELS();
    const index = typeof schedule.weekday === "number" ? schedule.weekday : 0;
    return `${labels[index % labels.length]} ${time}`;
  }
  if (repeat === "once") {
    return schedule.date ? `${schedule.date} ${time}` : time;
  }
  if (repeat === "workday" || repeat === "weekday") {
    return `${CONTENT.chat_page.subscriptions.weekdays_only} ${time}`;
  }
  if (repeat === "trading_day") {
    return `${CONTENT.chat_page.subscriptions.trading_days_only} ${time}`;
  }
  if (repeat === "holiday") {
    return `${CONTENT.chat_page.subscriptions.market_holidays} ${time}`;
  }
  if (repeat === "heartbeat") {
    return CONTENT.chat_page.subscriptions.continuous_monitoring;
  }
  return `${CONTENT.chat_page.subscriptions.every_day} ${time}`;
}

/** Parses `HH:MM` back into fields. Returns undefined rather than guessing. */
export function parseScheduleTime(raw: string): { hour: number; minute: number } | undefined {
  const match = /^\s*(\d{1,2})\s*:\s*(\d{1,2})\s*$/.exec(raw);
  if (!match) return undefined;
  const hour = Number(match[1]);
  const minute = Number(match[2]);
  if (!Number.isInteger(hour) || !Number.isInteger(minute)) return undefined;
  if (hour < 0 || hour > 23 || minute < 0 || minute > 59) return undefined;
  return { hour, minute };
}
