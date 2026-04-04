import { useEffect, useMemo, useState } from "react";
import Fuse from "fuse.js";
import type { Filters, TastingRecord } from "../types";

const getStoredProductType = () => {
  const stored = localStorage.getItem("productTypeFilter");
  if (stored === "sauce" || stored === "drink" || stored === "all")
    return stored;
  return "all";
};

const defaultFilters: Filters = {
  productType: getStoredProductType(),
  search: "",
  style: "",
  ingredient: "",
  minScore: "",
  minHeat: "",
  date: "",
  sortBy: "date",
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

type SortKey = Filters["sortBy"];

const comparators: Record<
  SortKey,
  (a: TastingRecord, b: TastingRecord) => number
> = {
  name: (a, b) => a.name.localeCompare(b.name),
  score: (a, b) => (b.score ?? -1) - (a.score ?? -1),
  style: (a, b) => a.style.localeCompare(b.style),
  heat: (a, b) => (b.heatUser ?? -1) - (a.heatUser ?? -1),
  date: (a, b) => new Date(b.date).getTime() - new Date(a.date).getTime(),
};

const applyFiltersAndSort = (tastings: TastingRecord[], filters: Filters) => {
  const predicates = buildFilterPredicates(filters);
  let results = tastings.filter((item) => matchesAllFilters(item, predicates));
  results = fuseSearch(results, filters.ingredient.trim());
  return [...results].sort(comparators[filters.sortBy] ?? comparators.date);
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
  const activeFilterCount = [
    filters.minScore,
    filters.minHeat,
    filters.style,
    filters.ingredient,
    filters.date,
  ].filter(Boolean).length;
  const resetFilters = () => setFilters(defaultFilters);

  return {
    filters,
    setFilters,
    filteredTastings,
    activeFilterCount,
    resetFilters,
    defaultFilters,
  };
}
