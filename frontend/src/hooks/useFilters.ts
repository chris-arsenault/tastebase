import { useCallback, useEffect, useMemo, useState } from "react";
import Fuse from "fuse.js";
import type { Filters, TastingRecord } from "../types";

const getStoredProductType = () => {
  const stored = localStorage.getItem("productTypeFilter");
  if (stored === "sauce" || stored === "drink" || stored === "all")
    return stored;
  return "all";
};

export type TastingSort = Filters["sortBy"];

export const defaultSortDirection = (sort: TastingSort): Filters["sortDir"] =>
  sort === "name" || sort === "style" ? "asc" : "desc";

const defaultFilters: Filters = {
  productType: getStoredProductType(),
  search: "",
  style: "",
  ingredient: "",
  minScore: "",
  minHeat: "",
  date: "",
  sortBy: "date",
  sortDir: "desc",
};

type FilterPredicate = (item: TastingRecord) => boolean;

const buildFilterPredicates = (filters: Filters): FilterPredicate[] => {
  const predicates: FilterPredicate[] = [];
  if (filters.productType !== "all") {
    predicates.push(
      (item) => (item.productType ?? "sauce") === filters.productType,
    );
  }
  const search = filters.search.trim().toLowerCase();
  if (search) {
    predicates.push((item) =>
      `${item.name} ${item.maker}`.toLowerCase().includes(search),
    );
  }
  const style = filters.style.trim().toLowerCase();
  if (style) {
    predicates.push((item) => item.style.toLowerCase().includes(style));
  }
  if (filters.date) {
    predicates.push((item) => item.date === filters.date);
  }
  const minScore = filters.minScore ? Number(filters.minScore) : null;
  if (minScore !== null) {
    predicates.push((item) => (item.score ?? -1) >= minScore);
  }
  const minHeat = filters.minHeat ? Number(filters.minHeat) : null;
  if (minHeat !== null) {
    predicates.push((item) => (item.heatUser ?? -1) >= minHeat);
  }
  return predicates;
};

const matchesAllFilters = (
  item: TastingRecord,
  predicates: FilterPredicate[],
) => predicates.every((pred) => pred(item));

const fuseSearch = (results: TastingRecord[], query: string) => {
  if (!query || results.length === 0) return results;
  const fuse = new Fuse(results, {
    keys: ["ingredients"],
    threshold: 0.4,
    ignoreLocation: true,
    useExtendedSearch: true,
  });
  return fuse.search(query).map((r) => r.item);
};

/** Ascending comparators; direction is applied once, afterwards. */
const comparators: Record<
  TastingSort,
  (a: TastingRecord, b: TastingRecord) => number
> = {
  name: (a, b) => a.name.localeCompare(b.name),
  score: (a, b) => (a.score ?? -1) - (b.score ?? -1),
  style: (a, b) => a.style.localeCompare(b.style),
  heat: (a, b) => (a.heatUser ?? -1) - (b.heatUser ?? -1),
  date: (a, b) => new Date(a.date).getTime() - new Date(b.date).getTime(),
};

const applyFiltersAndSort = (tastings: TastingRecord[], filters: Filters) => {
  const predicates = buildFilterPredicates(filters);
  let results = tastings.filter((item) => matchesAllFilters(item, predicates));
  results = fuseSearch(results, filters.ingredient.trim());
  const compare = comparators[filters.sortBy] ?? comparators.date;
  const sign = filters.sortDir === "asc" ? 1 : -1;
  return [...results].sort((a, b) => sign * compare(a, b));
};

export type ActiveTastingFilter = {
  key: "minScore" | "minHeat" | "style" | "ingredient" | "date";
  label: string;
};

const describeActiveFilters = (filters: Filters): ActiveTastingFilter[] => {
  const active: ActiveTastingFilter[] = [];
  if (filters.minScore) {
    active.push({ key: "minScore", label: `Score ${filters.minScore}+` });
  }
  if (filters.minHeat) {
    active.push({ key: "minHeat", label: `Heat ${filters.minHeat}+` });
  }
  if (filters.style) active.push({ key: "style", label: filters.style });
  if (filters.ingredient) {
    active.push({ key: "ingredient", label: `with ${filters.ingredient}` });
  }
  if (filters.date) active.push({ key: "date", label: filters.date });
  return active;
};

export function useFilters(tastings: TastingRecord[]) {
  const [filters, setFilters] = useState<Filters>(defaultFilters);

  useEffect(() => {
    localStorage.setItem("productTypeFilter", filters.productType);
  }, [filters.productType]);

  const filteredTastings = useMemo(
    () => applyFiltersAndSort(tastings, filters),
    [filters, tastings],
  );
  const activeFilters = useMemo(
    () => describeActiveFilters(filters),
    [filters],
  );
  const resetFilters = useCallback(
    () =>
      setFilters((current) => ({
        ...defaultFilters,
        productType: current.productType,
        search: current.search,
        sortBy: current.sortBy,
        sortDir: current.sortDir,
      })),
    [],
  );
  const clearFilter = useCallback(
    (key: ActiveTastingFilter["key"]) =>
      setFilters((current) => ({ ...current, [key]: "" })),
    [],
  );

  return {
    filters,
    setFilters,
    filteredTastings,
    activeFilters,
    activeFilterCount: activeFilters.length,
    resetFilters,
    clearFilter,
    defaultFilters,
  };
}
