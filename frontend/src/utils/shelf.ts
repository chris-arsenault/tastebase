import type { ShelfItem, ShelfStatus } from "../types";

export const bookStatusLabels = {
  recommended: "Want to read",
  reading: "Reading",
  read: "Read",
  did_not_finish: "Did not finish",
};
export const publicationStatusLabels = {
  recommended: "Recommended",
  subscribed: "Subscribed",
  cancelled: "Cancelled",
  not_interested: "Not interested",
};
export const shelfStatusLabels: Record<ShelfStatus, string> = {
  ...bookStatusLabels,
  ...publicationStatusLabels,
};

export function isReviewed(
  item: Pick<ShelfItem, "rating" | "writeup">,
): boolean {
  return item.rating != null && item.writeup.trim().length > 0;
}

export function creator(item: ShelfItem): string {
  return item.kind === "book" ? item.author : (item.publisher ?? "");
}

export type ShelfFilters = {
  status: "all" | ShelfStatus;
  kind: "all" | ShelfItem["kind"];
  reviewedOnly: boolean;
  tags: Record<string, string>;
};

export function filterShelf(
  items: ShelfItem[],
  filters: ShelfFilters,
): ShelfItem[] {
  return items.filter(
    (item) =>
      (filters.status === "all" || item.status === filters.status) &&
      (filters.kind === "all" || item.kind === filters.kind) &&
      (!filters.reviewedOnly || isReviewed(item)) &&
      Object.entries(filters.tags).every(([key, value]) =>
        item.tags.some((tag) => tag.key === key && tag.value === value),
      ),
  );
}
