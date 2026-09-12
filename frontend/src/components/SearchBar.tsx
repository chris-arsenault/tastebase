import { useCallback, useMemo } from "react";
import { HeatSlider, ScoreSlider } from "./display";
import { FilterBar, type ActiveFilter } from "./FilterBar";
import { Legend } from "./signals";
import type { Filters } from "../types";
import {
  defaultSortDirection,
  type ActiveTastingFilter,
  type TastingSort,
} from "../hooks/useFilters";

const sortOptions: { value: TastingSort; label: string }[] = [
  { value: "date", label: "Date" },
  { value: "name", label: "Name" },
  { value: "score", label: "Score" },
  { value: "heat", label: "Heat" },
  { value: "style", label: "Style" },
];

const tastingLegend = [
  { glyph: "🌶️", label: "heat 1–5" },
  { glyph: <span className="legend-bar" />, label: "score /10" },
  { glyph: <span className="status-dot state-info" />, label: "AI working" },
  { glyph: "!", label: "needs attention" },
];

type SetFilters = React.Dispatch<React.SetStateAction<Filters>>;

function FilterPanel({
  filters,
  setFilters,
}: Readonly<{ filters: Filters; setFilters: SetFilters }>) {
  const updateFilter = useCallback(
    <K extends keyof Filters>(key: K, value: Filters[K]) =>
      setFilters((prev) => ({ ...prev, [key]: value })),
    [setFilters],
  );
  const onMinHeatChange = useCallback(
    (val: string) => updateFilter("minHeat", val),
    [updateFilter],
  );
  const onMinScoreChange = useCallback(
    (val: string) => updateFilter("minScore", val),
    [updateFilter],
  );

  return (
    <>
      <div className="filter-row">
        <HeatSlider
          value={filters.minHeat}
          onChange={onMinHeatChange}
          label="Min heat"
        />
        <ScoreSlider
          value={filters.minScore}
          onChange={onMinScoreChange}
          label="Min score"
        />
      </div>
      <div className="filter-row">
        <label className="filter-field">
          <span>Style</span>
          <input
            placeholder="e.g. Habanero"
            value={filters.style}
            onChange={(e) => updateFilter("style", e.target.value)}
          />
        </label>
        <label className="filter-field">
          <span>Ingredient</span>
          <input
            placeholder="e.g. Garlic"
            value={filters.ingredient}
            onChange={(e) => updateFilter("ingredient", e.target.value)}
          />
        </label>
        <label className="filter-field">
          <span>Date</span>
          <input
            type="date"
            value={filters.date}
            onChange={(e) => updateFilter("date", e.target.value)}
          />
        </label>
      </div>
    </>
  );
}

type SearchBarProps = {
  filters: Filters;
  setFilters: SetFilters;
  activeFilters: ActiveTastingFilter[];
  resultCount: number;
  itemLabel: string;
  searchPlaceholder: string;
  onReset: () => void;
  onClearFilter: (key: ActiveTastingFilter["key"]) => void;
};

export function SearchBar({
  filters,
  setFilters,
  activeFilters,
  resultCount,
  itemLabel,
  searchPlaceholder,
  onReset,
  onClearFilter,
}: Readonly<SearchBarProps>) {
  const search = useMemo(
    () => ({
      value: filters.search,
      placeholder: searchPlaceholder,
      onChange: (value: string) =>
        setFilters((prev) => ({ ...prev, search: value })),
    }),
    [filters.search, searchPlaceholder, setFilters],
  );
  const sort = useMemo(
    () => ({
      options: sortOptions,
      value: filters.sortBy,
      direction: filters.sortDir,
      onChange: (value: TastingSort) =>
        setFilters((prev) => ({
          ...prev,
          sortBy: value,
          sortDir: defaultSortDirection(value),
        })),
      onToggleDirection: () =>
        setFilters((prev) => ({
          ...prev,
          sortDir: prev.sortDir === "asc" ? "desc" : "asc",
        })),
    }),
    [filters.sortBy, filters.sortDir, setFilters],
  );
  const chips: ActiveFilter[] = useMemo(
    () =>
      activeFilters.map((filter) => ({
        id: filter.key,
        label: filter.label,
        onRemove: () => onClearFilter(filter.key),
      })),
    [activeFilters, onClearFilter],
  );
  const resultSummary = useMemo(
    () => ({ count: resultCount, noun: itemLabel }),
    [itemLabel, resultCount],
  );
  const legend = useMemo(() => <Legend items={tastingLegend} />, []);

  return (
    <FilterBar
      search={search}
      sort={sort}
      resultCount={resultSummary}
      activeFilters={chips}
      onClearAll={onReset}
      legend={legend}
    >
      <FilterPanel filters={filters} setFilters={setFilters} />
    </FilterBar>
  );
}
