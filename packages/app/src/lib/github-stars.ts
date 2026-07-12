const HONE_REPO_API_URL =
  "https://api.github.com/repos/B-M-Capital-Research/honeclaw";
const GITHUB_STARS_CACHE_KEY = "hone.github.stars.v2";
const GITHUB_STARS_CACHE_TTL_MS = 6 * 60 * 60 * 1000;

export const GITHUB_STARS_FALLBACK = "742";

type GithubStarsCache = {
  value: string;
  cachedAt: number;
};

let githubStarsRequest: Promise<string> | undefined;

export function formatGithubStars(value: number): string {
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}m`;
  if (value >= 10_000) return `${Math.round(value / 1_000)}k`;
  if (value >= 1_000) return `${(value / 1_000).toFixed(1)}k`;
  return value.toLocaleString("en-US");
}

export function parseGithubStarsCache(
  raw: string | null,
  now = Date.now(),
): string | undefined {
  if (!raw) return undefined;
  try {
    const cached = JSON.parse(raw) as Partial<GithubStarsCache>;
    if (
      typeof cached.value !== "string" ||
      !cached.value ||
      cached.value === "..." ||
      typeof cached.cachedAt !== "number" ||
      now - cached.cachedAt < 0 ||
      now - cached.cachedAt > GITHUB_STARS_CACHE_TTL_MS
    ) {
      return undefined;
    }
    return cached.value;
  } catch {
    return undefined;
  }
}

function readCachedGithubStars(): string | undefined {
  if (typeof window === "undefined") return undefined;
  return parseGithubStarsCache(
    window.localStorage.getItem(GITHUB_STARS_CACHE_KEY),
  );
}

function writeCachedGithubStars(value: string) {
  if (typeof window === "undefined") return;
  const cached: GithubStarsCache = { value, cachedAt: Date.now() };
  window.localStorage.setItem(GITHUB_STARS_CACHE_KEY, JSON.stringify(cached));
}

export function displayGithubStars(value: string | undefined | null): string {
  return value && value !== "..."
    ? value
    : readCachedGithubStars() ?? GITHUB_STARS_FALLBACK;
}

async function requestGithubStars(): Promise<string> {
  try {
    const response = await fetch(HONE_REPO_API_URL, {
      cache: "no-store",
      headers: {
        Accept: "application/vnd.github+json",
      },
    });
    if (!response.ok) return displayGithubStars(undefined);
    const repoPayload = (await response.json()) as { stargazers_count?: unknown };
    if (typeof repoPayload.stargazers_count !== "number") {
      return displayGithubStars(undefined);
    }
    const next = formatGithubStars(repoPayload.stargazers_count);
    writeCachedGithubStars(next);
    return next;
  } catch {
    return displayGithubStars(undefined);
  }
}

export function fetchGithubStars(): Promise<string> {
  if (!githubStarsRequest) {
    githubStarsRequest = requestGithubStars().finally(() => {
      githubStarsRequest = undefined;
    });
  }
  return githubStarsRequest;
}
