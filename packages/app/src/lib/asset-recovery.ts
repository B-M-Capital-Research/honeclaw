const RECOVERY_STORAGE_KEY = "hone.asset-recovery.reload";
const DEFAULT_RELOAD_INTERVAL_MS = 60_000;

type RecoveryRecord = {
  href: string;
  at: number;
};

type RecoveryOptions = {
  now?: () => number;
  href?: string;
  storage?: Pick<Storage, "getItem" | "setItem">;
  reload?: () => void;
  reloadDelayMs?: number;
  minReloadIntervalMs?: number;
};

const RECOVERABLE_ERROR_PATTERNS = [
  /text\/html.*valid javascript mime type/i,
  /failed to fetch dynamically imported module/i,
  /error loading dynamically imported module/i,
  /importing a module script failed/i,
  /chunkloaderror/i,
  /loading chunk \d+ failed/i,
  /unable to preload css/i,
  /vite:preloaderror/i,
];

function getErrorText(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) {
    return `${error.name}: ${error.message}`;
  }
  if (typeof error === "object" && error !== null) {
    const maybe = error as { message?: unknown; reason?: unknown; error?: unknown; type?: unknown };
    return [
      typeof maybe.type === "string" ? maybe.type : "",
      getErrorText(maybe.error),
      getErrorText(maybe.reason),
      typeof maybe.message === "string" ? maybe.message : "",
    ].join(" ");
  }
  return "";
}

export function isRecoverableAssetLoadError(error: unknown): boolean {
  const text = getErrorText(error);
  return RECOVERABLE_ERROR_PATTERNS.some((pattern) => pattern.test(text));
}

function readRecoveryRecord(storage: RecoveryOptions["storage"]): RecoveryRecord | null {
  if (!storage) return null;
  try {
    const raw = storage.getItem(RECOVERY_STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as Partial<RecoveryRecord>;
    if (typeof parsed.href !== "string" || typeof parsed.at !== "number") return null;
    return { href: parsed.href, at: parsed.at };
  } catch {
    return null;
  }
}

function writeRecoveryRecord(
  storage: RecoveryOptions["storage"],
  record: RecoveryRecord,
) {
  if (!storage) return;
  try {
    storage.setItem(RECOVERY_STORAGE_KEY, JSON.stringify(record));
  } catch {
    // Storage may be unavailable in private mode; reload recovery should still run.
  }
}

export function recoverFromAssetLoadError(
  error: unknown,
  options: RecoveryOptions = {},
): boolean {
  if (!isRecoverableAssetLoadError(error)) return false;
  if (typeof window === "undefined" && !options.reload) return false;

  const now = options.now?.() ?? Date.now();
  const href = options.href ?? window.location.href;
  const storage = options.storage ?? window.sessionStorage;
  const minReloadIntervalMs =
    options.minReloadIntervalMs ?? DEFAULT_RELOAD_INTERVAL_MS;
  const previous = readRecoveryRecord(storage);
  if (previous?.href === href && now - previous.at < minReloadIntervalMs) {
    return false;
  }

  writeRecoveryRecord(storage, { href, at: now });
  const reload = options.reload ?? (() => window.location.reload());
  globalThis.setTimeout(reload, options.reloadDelayMs ?? 120);
  return true;
}

function eventTargetLooksLikeScriptAsset(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  if (target instanceof HTMLScriptElement) {
    return /\.m?js($|\?)/.test(target.src);
  }
  if (target instanceof HTMLLinkElement) {
    return target.rel === "modulepreload" || /\.css($|\?)/.test(target.href);
  }
  return false;
}

export function installAssetLoadRecovery(win: Window = window) {
  win.addEventListener("vite:preloadError", (event) => {
    if (recoverFromAssetLoadError(event)) {
      event.preventDefault();
    }
  });

  win.addEventListener("unhandledrejection", (event) => {
    if (recoverFromAssetLoadError(event.reason)) {
      event.preventDefault();
    }
  });

  win.addEventListener(
    "error",
    (event) => {
      const error = eventTargetLooksLikeScriptAsset(event.target)
        ? "Failed to fetch dynamically imported module"
        : event.error ?? event.message;
      if (recoverFromAssetLoadError(error)) {
        event.preventDefault();
      }
    },
    true,
  );
}
