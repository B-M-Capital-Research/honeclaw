import type { PublicPushListItem } from "./types";

export const ALL_PUBLIC_PUSHES = "all" as const;

export type PublicPushCategory = {
  jobId: string;
  title: string;
  count: number;
};

/**
 * Historical pushes own their category label. This deliberately does not join
 * against only-active subscriptions: stopped or deleted jobs must keep their
 * archived messages discoverable.
 */
export function publicPushCategories(
  items: PublicPushListItem[],
): PublicPushCategory[] {
  const categories = new Map<string, PublicPushCategory>();
  for (const item of items) {
    const current = categories.get(item.job_id);
    if (current) {
      current.count += 1;
      continue;
    }
    categories.set(item.job_id, {
      jobId: item.job_id,
      title: item.title,
      count: 1,
    });
  }
  return [...categories.values()];
}

export function filterPublicPushes(
  items: PublicPushListItem[],
  category: string,
): PublicPushListItem[] {
  if (category === ALL_PUBLIC_PUSHES) return items;
  return items.filter((item) => item.job_id === category);
}
