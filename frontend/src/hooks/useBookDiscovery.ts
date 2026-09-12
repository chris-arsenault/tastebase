import { useCallback, useMemo } from "react";
import type { ShelfItem, ShelfStatus } from "../types";
import {
  collectTagFacets,
  creator,
  filterShelf,
  type ShelfFilters,
  type TagSelection,
} from "../utils/shelf";
import { useQueryState } from "./useQueryState";

export type BookFilter = "all" | ShelfStatus;
export type KindFilter = "all" | ShelfItem["kind"];
export type BookSort = "recommendedAt" | "title" | "author" | "pageCount";
export type SortDirection = "asc" | "desc";

export type DiscoveryState = {
  search: string;
  status: BookFilter;
  kind: KindFilter;
  reviewedOnly: boolean;
  tags: TagSelection;
  sort: BookSort;
  direction: SortDirection;
};

const shelfStatuses: ReadonlySet<string> = new Set([
  "recommended",
  "reading",
  "read",
  "did_not_finish",
  "subscribed",
  "cancelled",
  "not_interested",
]);
const sorts: ReadonlySet<string> = new Set([
  "recommendedAt",
  "title",
  "author",
  "pageCount",
]);

export const defaultDirection = (sort: BookSort): SortDirection =>
  sort === "recommendedAt" ? "desc" : "asc";

function parseTags(params: URLSearchParams): TagSelection {
  const tags: TagSelection = {};
  for (const entry of params.getAll("tag")) {
    const separator = entry.indexOf(":");
    if (separator <= 0) continue;
    const key = entry.slice(0, separator);
    const value = entry.slice(separator + 1);
    if (!value) continue;
    tags[key] = [...(tags[key] ?? []), value];
  }
  return tags;
}

function parseSort(params: URLSearchParams) {
  const sort = params.get("sort") ?? "recommendedAt";
  const resolved = (sorts.has(sort) ? sort : "recommendedAt") as BookSort;
  const direction = params.get("dir");
  return {
    sort: resolved,
    direction:
      direction === "asc" || direction === "desc"
        ? direction
        : defaultDirection(resolved),
  };
}

function parseKind(params: URLSearchParams): KindFilter {
  const kind = params.get("kind");
  return kind === "book" || kind === "publication" ? kind : "all";
}

function parseStatus(params: URLSearchParams): BookFilter {
  const status = params.get("status") ?? "all";
  return (shelfStatuses.has(status) ? status : "all") as BookFilter;
}

function parseState(params: URLSearchParams): DiscoveryState {
  return {
    search: params.get("q") ?? "",
    status: parseStatus(params),
    kind: parseKind(params),
    reviewedOnly: params.get("reviewed") === "1",
    tags: parseTags(params),
    ...parseSort(params),
  };
}

function serializeState(state: DiscoveryState, params: URLSearchParams) {
  if (state.search) params.set("q", state.search);
  if (state.status !== "all") params.set("status", state.status);
  if (state.kind !== "all") params.set("kind", state.kind);
  if (state.reviewedOnly) params.set("reviewed", "1");
  for (const [key, values] of Object.entries(state.tags)) {
    for (const value of values) params.append("tag", `${key}:${value}`);
  }
  if (state.sort !== "recommendedAt") params.set("sort", state.sort);
  if (state.direction !== defaultDirection(state.sort)) {
    params.set("dir", state.direction);
  }
}

const collator = new Intl.Collator(undefined, {
  numeric: true,
  sensitivity: "base",
});

function comparePageCounts(
  left: number | null,
  right: number | null,
  direction: SortDirection,
): number {
  if (left == null) return right == null ? 0 : 1;
  if (right == null) return -1;
  return direction === "asc" ? left - right : right - left;
}

function textSortValue(book: ShelfItem, sort: BookSort): string {
  if (sort === "title") return book.title;
  if (sort === "author") return creator(book);
  return book.recommendedAt;
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
    const raw = collator.compare(
      textSortValue(left, sort),
      textSortValue(right, sort),
    );
    comparison = direction === "asc" ? raw : -raw;
  }
  return comparison || collator.compare(left.title, right.title);
}

function toggleValue(values: string[], value: string): string[] {
  return values.includes(value)
    ? values.filter((entry) => entry !== value)
    : [...values, value];
}

type SetDiscoveryState = (
  next: DiscoveryState | ((current: DiscoveryState) => DiscoveryState),
) => void;

function useDiscoveryActions(setState: SetDiscoveryState) {
  const patch = useCallback(
    (changes: Partial<DiscoveryState>) =>
      setState((current) => ({ ...current, ...changes })),
    [setState],
  );
  const setSort = useCallback(
    (sort: BookSort) => patch({ sort, direction: defaultDirection(sort) }),
    [patch],
  );
  const toggleDirection = useCallback(
    () =>
      setState((current) => ({
        ...current,
        direction: current.direction === "asc" ? "desc" : "asc",
      })),
    [setState],
  );
  const toggleTag = useCallback(
    (key: string, value: string) =>
      setState((current) => {
        const values = toggleValue(current.tags[key] ?? [], value);
        const tags = { ...current.tags };
        if (values.length === 0) delete tags[key];
        else tags[key] = values;
        return { ...current, tags };
      }),
    [setState],
  );
  const clearFilters = useCallback(
    () =>
      setState((current) => ({
        ...current,
        status: "all",
        kind: "all",
        reviewedOnly: false,
        tags: {},
      })),
    [setState],
  );
  return { patch, setSort, toggleDirection, toggleTag, clearFilters };
}

export function useBookDiscovery(books: ShelfItem[]) {
  const [state, setState] = useQueryState(parseState, serializeState);
  const actions = useDiscoveryActions(setState);

  const filters: ShelfFilters = useMemo(
    () => ({
      status: state.status,
      kind: state.kind,
      reviewedOnly: state.reviewedOnly,
      tags: state.tags,
      search: state.search,
    }),
    [state.kind, state.reviewedOnly, state.search, state.status, state.tags],
  );

  const visibleBooks = useMemo(
    () =>
      filterShelf(books, filters).sort((left, right) =>
        compareBooks(left, right, state.sort, state.direction),
      ),
    [books, filters, state.direction, state.sort],
  );

  const tagFacets = useMemo(
    () => collectTagFacets(books, filters),
    [books, filters],
  );

  const selectedTagCount = Object.values(state.tags).reduce(
    (sum, values) => sum + values.length,
    0,
  );
  const hasActiveFilters =
    state.status !== "all" ||
    state.kind !== "all" ||
    state.reviewedOnly ||
    selectedTagCount > 0 ||
    state.search.trim().length > 0;

  return {
    state,
    ...actions,
    tagFacets,
    hasActiveFilters,
    visibleBooks,
  };
}
