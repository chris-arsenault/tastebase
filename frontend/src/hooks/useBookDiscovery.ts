import { useCallback, useMemo, useState, type ChangeEvent } from "react";
import type { BookRecommendation, BookStatus } from "../types";

export type BookFilter = "all" | BookStatus;
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

function textSortValue(book: BookRecommendation, sort: BookSort): string {
  if (sort === "title") return book.title;
  if (sort === "author") return book.author;
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
  left: BookRecommendation,
  right: BookRecommendation,
  sort: BookSort,
  direction: SortDirection,
): number {
  let comparison: number;
  if (sort === "pageCount") {
    comparison = comparePageCounts(left.pageCount, right.pageCount, direction);
  } else {
    comparison = compareTextValues(
      textSortValue(left, sort),
      textSortValue(right, sort),
      direction,
    );
  }
  return comparison || collator.compare(left.title, right.title);
}

function matchesSelectedTags(
  book: BookRecommendation,
  selectedTagValues: Record<string, string>,
): boolean {
  return Object.entries(selectedTagValues).every(([key, value]) =>
    book.tags.some((tag) => tag.key === key && tag.value === value),
  );
}

function collectAvailableTagFacets(
  books: BookRecommendation[],
): BookTagFacet[] {
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

export function useBookDiscovery(
  books: BookRecommendation[],
  isOwnerView: boolean,
) {
  const [statusFilter, setStatusFilter] = useState<BookFilter>("all");
  const [sort, setSort] = useState<BookSort>("recommendedAt");
  const [direction, setDirection] = useState<SortDirection>("desc");
  const [selectedTagValues, setSelectedTagValues] = useState<
    Record<string, string>
  >({});
  const availableTagFacets = useMemo(
    () => collectAvailableTagFacets(books),
    [books],
  );
  const visibleBooks = useMemo(() => {
    const filtered = books.filter(
      (book) =>
        (!isOwnerView ||
          statusFilter === "all" ||
          book.status === statusFilter) &&
        matchesSelectedTags(book, selectedTagValues),
    );
    return filtered.sort((left, right) =>
      compareBooks(left, right, sort, direction),
    );
  }, [books, direction, isOwnerView, selectedTagValues, sort, statusFilter]);
  const handleStatusFilter = useCallback(
    (event: ChangeEvent<HTMLSelectElement>) => {
      setStatusFilter(event.currentTarget.value as BookFilter);
    },
    [],
  );
  const handleSort = useCallback((event: ChangeEvent<HTMLSelectElement>) => {
    const nextSort = event.currentTarget.value as BookSort;
    setSort(nextSort);
    setDirection(nextSort === "recommendedAt" ? "desc" : "asc");
  }, []);
  const toggleDirection = useCallback(() => {
    setDirection((current) => (current === "asc" ? "desc" : "asc"));
  }, []);
  const selectTagValue = useCallback((key: string, value: string) => {
    setSelectedTagValues((current) => {
      const next = { ...current };
      if (value) next[key] = value;
      else delete next[key];
      return next;
    });
  }, []);
  const clearTags = useCallback(() => setSelectedTagValues({}), []);

  return {
    availableTagFacets,
    clearTags,
    direction,
    handleSort,
    handleStatusFilter,
    hasActiveFilters:
      (isOwnerView && statusFilter !== "all") ||
      Object.keys(selectedTagValues).length > 0,
    selectedTagValues,
    selectTagValue,
    sort,
    statusFilter,
    toggleDirection,
    visibleBooks,
  };
}
