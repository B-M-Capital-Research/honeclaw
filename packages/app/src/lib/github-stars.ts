const HONE_REPO_API_URL =
  "https://api.github.com/repos/B-M-Capital-Research/honeclaw";
const GITHUB_STARS_CACHE_KEY = "hone.github.stars";

export const GITHUB_STARS_FALLBACK = "548";

export function formatGithubStars(value: number): string {
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}m`;
  if (value >= 10_000) return `${Math.round(value / 1_000)}k`;
  if (value >= 1_000) return `${(value / 1_000).toFixed(1)}k`;
  return value.toLocaleString("en-US");
}

function readCachedGithubStars(): string | undefined {
  if (typeof window === "undefined") return undefined;
  const cached = window.localStorage.getItem(GITHUB_STARS_CACHE_KEY);
  return cached && cached !== "..." ? cached : undefined;
}

function writeCachedGithubStars(value: string) {
  if (typeof window === "undefined") return;
  window.localStorage.setItem(GITHUB_STARS_CACHE_KEY, value);
}

export function displayGithubStars(value: string | undefined | null): string {
  return value && value !== "..."
    ? value
    : readCachedGithubStars() ?? GITHUB_STARS_FALLBACK;
}

export async function fetchGithubStars(): Promise<string> {
  try {
    const response = await fetch(HONE_REPO_API_URL, {
      headers: {
        Accept: "application/vnd.github+json",
      },
    });
    if (!response.ok) return displayGithubStars(undefined);
    const data = (await response.json()) as { stargazers_count?: unknown };
    if (typeof data.stargazers_count !== "number") {
      return displayGithubStars(undefined);
    }
    const next = formatGithubStars(data.stargazers_count);
    writeCachedGithubStars(next);
    return next;
  } catch {
    return displayGithubStars(undefined);
  }
}
