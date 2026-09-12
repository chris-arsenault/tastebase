import { useCallback, useMemo, useState, type ChangeEvent } from "react";
import type { ShelfItem, ShelfStatus } from "../types";
import { creator, filterShelf } from "../utils/shelf";

export type BookFilter = "all" | ShelfStatus;
export type KindFilter = "all" | ShelfItem["kind"];
export type BookSort = "recommendedAt" | "title" | "author" | "pageCount";
export type SortDirection = "asc" | "desc";
export type BookTagFacet = { key: string; values: string[] };

const collator = new Intl.Collator(undefined, {
  numeric: true,
  sensitivity: "base",
});

function comparePageCounts(
  left: number | null,
  right: number | null,
  direction: SortDirection,
): number {
  if (left == null) {
    if (right == null) return 0;
    return 1;
  }
  if (right == null) return -1;
  return direction === "asc" ? left - right : right - left;
}

function textSortValue(book: ShelfItem, sort: BookSort): string {
  if (sort === "title") return book.title;
  if (sort === "author") return creator(book);
  return book.recommendedAt;
}

function compareTextValues(
  left: string,
  right: string,
  direction: SortDirection,
): number {
  const comparison = collator.compare(left, right);
  return direction === "asc" ? comparison : -comparison;
}

function compareBooks(
  left: ShelfItem,
  right: ShelfItem,
  sort: BookSort,
  direction: SortDirection,
): number {
  let comparison: number;
  if (sort === "pageCount") {
    comparison = comparePageCounts(
      left.kind === "book" ? left.pageCount : null,
      right.kind === "book" ? right.pageCount : null,
      direction,
    );
  } else {
    comparison = compareTextValues(
      textSortValue(left, sort),
      textSortValue(right, sort),
      direction,
    );
  }
  return comparison || collator.compare(left.title, right.title);
}

function collectAvailableTagFacets(books: ShelfItem[]): BookTagFacet[] {
  const valuesByKey = new Map<string, Set<string>>();
  for (const book of books) {
    for (const tag of book.tags) {
      const values = valuesByKey.get(tag.key) ?? new Set<string>();
      values.add(tag.value);
      valuesByKey.set(tag.key, values);
    }
  }
  return [...valuesByKey]
    .map(([key, values]) => ({
      key,
      values: [...values].sort(collator.compare),
    }))
    .sort((left, right) => collator.compare(left.key, right.key));
}

function useTagSelection(books: ShelfItem[]) {
  const [selectedTagValues, setSelectedTagValues] = useState<
    Record<string, string>
  >({});
  const availableTagFacets = useMemo(
    () => collectAvailableTagFacets(books),
    [books],
  );
  const selectTagValue = useCallback((key: string, value: string) => {
    setSelectedTagValues((current) => {
      const next = { ...current };
      if (value) next[key] = value;
      else delete next[key];
      return next;
    });
  }, []);
  const clearTags = useCallback(() => setSelectedTagValues({}), []);
  return { selectedTagValues, availableTagFacets, selectTagValue, clearTags };
}

function useShelfSort() {
  const [sort, setSort] = useState<BookSort>("recommendedAt");
  const [direction, setDirection] = useState<SortDirection>("desc");
  const handleSort = useCallback((event: ChangeEvent<HTMLSelectElement>) => {
    const nextSort = event.currentTarget.value as BookSort;
    setSort(nextSort);
    setDirection(nextSort === "recommendedAt" ? "desc" : "asc");
  }, []);
  const toggleDirection = useCallback(() => {
    setDirection((current) => (current === "asc" ? "desc" : "asc"));
  }, []);
  return { sort, direction, handleSort, toggleDirection };
}

export function useBookDiscovery(books: ShelfItem[]) {
  const [statusFilter, setStatusFilter] = useState<BookFilter>("all");
  const [kindFilter, setKindFilter] = useState<KindFilter>("all");
  const [reviewedOnly, setReviewedOnly] = useState(false);
  const { sort, direction, handleSort, toggleDirection } = useShelfSort();
  const { selectedTagValues, availableTagFacets, selectTagValue, clearTags } =
    useTagSelection(books);
  const visibleBooks = useMemo(() => {
    const filtered = filterShelf(books, {
      status: statusFilter,
      kind: kindFilter,
      reviewedOnly,
      tags: selectedTagValues,
    });
    return filtered.sort((left, right) =>
      compareBooks(left, right, sort, direction),
    );
  }, [
    books,
    direction,
    kindFilter,
    reviewedOnly,
    selectedTagValues,
    sort,
    statusFilter,
  ]);
  const handleStatusFilter = useCallback(
    (event: ChangeEvent<HTMLSelectElement>) => {
      setStatusFilter(event.currentTarget.value as BookFilter);
    },
    [],
  );
  const handleKindFilter = useCallback(
    (event: ChangeEvent<HTMLSelectElement>) => {
      setKindFilter(event.currentTarget.value as KindFilter);
    },
    [],
  );
  const handleReviewedOnly = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      setReviewedOnly(event.currentTarget.checked);
    },
    [],
  );

  return {
    availableTagFacets,
    clearTags,
    direction,
    handleSort,
    handleStatusFilter,
    handleKindFilter,
    handleReviewedOnly,
    kindFilter,
    reviewedOnly,
    hasActiveFilters:
      statusFilter !== "all" ||
      kindFilter !== "all" ||
      reviewedOnly ||
      Object.keys(selectedTagValues).length > 0,
    selectedTagValues,
    selectTagValue,
    sort,
    statusFilter,
    toggleDirection,
    visibleBooks,
  };
}
