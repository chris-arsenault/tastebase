import { useCallback, useState } from "react";
import { HeatSlider, ScoreSlider } from "./display";
import type { Filters } from "../types";

type SearchBarProps = {
  filters: Filters;
  setFilters: React.Dispatch<React.SetStateAction<Filters>>;
  activeFilterCount: number;
  searchPlaceholder: string;
  onReset: () => void;
};

export function SearchBar({ filters, setFilters, activeFilterCount, searchPlaceholder, onReset }: Readonly<SearchBarProps>) {
  const [filtersExpanded, setFiltersExpanded] = useState(false);

  const updateFilter = useCallback(
    <K extends keyof Filters>(key: K, value: Filters[K]) => setFilters((prev) => ({ ...prev, [key]: value })),
    [setFilters]
  );
  const onMinHeatChange = useCallback((val: string) => updateFilter("minHeat", val), [updateFilter]);
  const onMinScoreChange = useCallback((val: string) => updateFilter("minScore", val), [updateFilter]);

  return (
    <div className="search-bar">
      <div className="search-main">
        <button
          className={`filter-toggle ${filtersExpanded ? "active" : ""} ${activeFilterCount > 0 ? "has-filters" : ""}`}
          onClick={() => setFiltersExpanded((prev) => !prev)}
          title="Filters"
        >
          <span className="filter-icon">&#x2699;</span>
          {activeFilterCount > 0 && <span className="filter-badge">{activeFilterCount}</span>}
        </button>
        <select
          value={filters.sortBy}
          onChange={(e) => setFilters((prev) => ({ ...prev, sortBy: e.target.value as Filters["sortBy"] }))}
          className="sort-select"
          title="Sort by"
        >
          <option value="date">New</option>
          <option value="name">A-Z</option>
          <option value="score">Top</option>
          <option value="heat">Hot</option>
        </select>
        <div className="search-field">
          <span className="search-icon">&#x1F50D;</span>
          <input
            type="search"
            placeholder={searchPlaceholder}
            value={filters.search}
            onChange={(e) => setFilters((prev) => ({ ...prev, search: e.target.value }))}
          />
        </div>
      </div>

      {filtersExpanded && (
        <div className="filter-panel">
          <div className="filter-row">
            <HeatSlider value={filters.minHeat} onChange={onMinHeatChange} label="Min Heat" />
            <ScoreSlider value={filters.minScore} onChange={onMinScoreChange} label="Min Score" />
          </div>
          <div className="filter-row">
            <label className="filter-field">
              <span>Style</span>
              <input placeholder="e.g. Habanero" value={filters.style} onChange={(e) => setFilters((prev) => ({ ...prev, style: e.target.value }))} />
            </label>
            <label className="filter-field">
              <span>Ingredient</span>
              <input placeholder="e.g. Garlic" value={filters.ingredient} onChange={(e) => setFilters((prev) => ({ ...prev, ingredient: e.target.value }))} />
            </label>
            <label className="filter-field">
              <span>Date</span>
              <input type="date" value={filters.date} onChange={(e) => setFilters((prev) => ({ ...prev, date: e.target.value }))} />
            </label>
            {activeFilterCount > 0 && (
              <button className="clear-filters" onClick={onReset}>Clear all</button>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
