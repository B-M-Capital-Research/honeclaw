const HONE_REPO_API_URL =
  "https://api.github.com/repos/B-M-Capital-Research/honeclaw";

export function formatGithubStars(value: number): string {
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}m`;
  if (value >= 10_000) return `${Math.round(value / 1_000)}k`;
  if (value >= 1_000) return `${(value / 1_000).toFixed(1)}k`;
  return value.toLocaleString("en-US");
}

export async function fetchGithubStars(): Promise<string> {
  try {
    const response = await fetch(HONE_REPO_API_URL, {
      headers: {
        Accept: "application/vnd.github+json",
      },
    });
    if (!response.ok) return "...";
    const data = (await response.json()) as { stargazers_count?: unknown };
    return typeof data.stargazers_count === "number"
      ? formatGithubStars(data.stargazers_count)
      : "...";
  } catch {
    return "...";
  }
}
