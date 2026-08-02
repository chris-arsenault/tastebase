import { useCallback, useMemo, useState, type ChangeEvent } from "react";
import type { BookRecommendation, BookStatus, BookTag } from "../types";

export type BookFilter = "all" | BookStatus;
export type BookSort = "recommendedAt" | "title" | "author" | "pageCount";
export type SortDirection = "asc" | "desc";

const collator = new Intl.Collator(undefined, {
  numeric: true,
  sensitivity: "base",
});

export const bookTagToken = (tag: BookTag) => `${tag.key}\u0000${tag.value}`;

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

function selectedTagsByKey(
  selectedTags: Set<string>,
): Map<string, Set<string>> {
  const selectedByKey = new Map<string, Set<string>>();
  for (const token of selectedTags) {
    const [key, value] = token.split("\u0000");
    const values = selectedByKey.get(key) ?? new Set<string>();
    values.add(value);
    selectedByKey.set(key, values);
  }
  return selectedByKey;
}

function matchesSelectedTags(
  book: BookRecommendation,
  selectedByKey: Map<string, Set<string>>,
): boolean {
  return [...selectedByKey].every(([key, values]) =>
    book.tags.some((tag) => tag.key === key && values.has(tag.value)),
  );
}

function collectAvailableTags(books: BookRecommendation[]): BookTag[] {
  const tags = new Map<string, BookTag>();
  for (const book of books) {
    for (const tag of book.tags) tags.set(bookTagToken(tag), tag);
  }
  return [...tags.values()].sort(
    (left, right) =>
      collator.compare(left.key, right.key) ||
      collator.compare(left.value, right.value),
  );
}

export function useBookDiscovery(
  books: BookRecommendation[],
  isOwnerView: boolean,
) {
  const [statusFilter, setStatusFilter] = useState<BookFilter>("all");
  const [sort, setSort] = useState<BookSort>("recommendedAt");
  const [direction, setDirection] = useState<SortDirection>("desc");
  const [selectedTagTokens, setSelectedTagTokens] = useState<string[]>([]);
  const selectedTags = useMemo(
    () => new Set(selectedTagTokens),
    [selectedTagTokens],
  );
  const availableTags = useMemo(() => collectAvailableTags(books), [books]);
  const visibleBooks = useMemo(() => {
    const selectedByKey = selectedTagsByKey(selectedTags);
    const filtered = books.filter(
      (book) =>
        (!isOwnerView ||
          statusFilter === "all" ||
          book.status === statusFilter) &&
        matchesSelectedTags(book, selectedByKey),
    );
    return filtered.sort((left, right) =>
      compareBooks(left, right, sort, direction),
    );
  }, [books, direction, isOwnerView, selectedTags, sort, statusFilter]);
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
  const toggleTag = useCallback((tag: BookTag) => {
    const token = bookTagToken(tag);
    setSelectedTagTokens((current) =>
      current.includes(token)
        ? current.filter((value) => value !== token)
        : [...current, token],
    );
  }, []);
  const clearTags = useCallback(() => setSelectedTagTokens([]), []);

  return {
    availableTags,
    clearTags,
    direction,
    handleSort,
    handleStatusFilter,
    hasActiveFilters:
      (isOwnerView && statusFilter !== "all") || selectedTags.size > 0,
    selectedTags,
    sort,
    statusFilter,
    toggleDirection,
    toggleTag,
    visibleBooks,
  };
}
