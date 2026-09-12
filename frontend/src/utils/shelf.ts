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

/** Selected tag values per key. OR within a key, AND across keys. */
export type TagSelection = Record<string, string[]>;

export type ShelfFilters = {
  status: "all" | ShelfStatus;
  kind: "all" | ShelfItem["kind"];
  reviewedOnly: boolean;
  tags: TagSelection;
  search?: string;
};

function matchesTags(item: ShelfItem, tags: TagSelection): boolean {
  return Object.entries(tags).every(
    ([key, values]) =>
      values.length === 0 ||
      item.tags.some((tag) => tag.key === key && values.includes(tag.value)),
  );
}

function matchesSearch(item: ShelfItem, search: string): boolean {
  const query = search.trim().toLowerCase();
  if (!query) return true;
  return `${item.title} ${creator(item)} ${item.summary}`
    .toLowerCase()
    .includes(query);
}

export function filterShelf(
  items: ShelfItem[],
  filters: ShelfFilters,
): ShelfItem[] {
  return items.filter(
    (item) =>
      (filters.status === "all" || item.status === filters.status) &&
      (filters.kind === "all" || item.kind === filters.kind) &&
      (!filters.reviewedOnly || isReviewed(item)) &&
      matchesTags(item, filters.tags) &&
      matchesSearch(item, filters.search ?? ""),
  );
}

export type TagFacetValue = { value: string; count: number };
export type TagFacet = { key: string; values: TagFacetValue[] };

const collator = new Intl.Collator(undefined, {
  numeric: true,
  sensitivity: "base",
});

/**
 * Facet counts that respect every *other* active filter, so each chip shows
 * how many results picking it would yield. Values with zero matches are kept
 * (so a selected value never vanishes) but sort last.
 */
export function collectTagFacets(
  items: ShelfItem[],
  filters: ShelfFilters,
): TagFacet[] {
  const keys = new Set<string>();
  for (const item of items) for (const tag of item.tags) keys.add(tag.key);

  return [...keys].sort(collator.compare).map((key) => {
    const others = { ...filters, tags: { ...filters.tags, [key]: [] } };
    const scope = filterShelf(items, others);
    const counts = new Map<string, number>();
    for (const item of items) {
      for (const tag of item.tags) {
        if (tag.key === key && !counts.has(tag.value)) counts.set(tag.value, 0);
      }
    }
    for (const item of scope) {
      for (const tag of item.tags) {
        if (tag.key === key)
          counts.set(tag.value, (counts.get(tag.value) ?? 0) + 1);
      }
    }
    const values = [...counts]
      .map(([value, count]) => ({ value, count }))
      .sort(
        (left, right) =>
          right.count - left.count || collator.compare(left.value, right.value),
      );
    return { key, values };
  });
}
