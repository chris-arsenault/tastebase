import { useState, type ReactNode } from "react";

export type SortOption<S extends string> = { value: S; label: string };
export type SortDirection = "asc" | "desc";

export type ActiveFilter = {
  id: string;
  label: string;
  onRemove: () => void;
  colorClass?: string;
};

type SearchProps = {
  value: string;
  placeholder: string;
  onChange: (value: string) => void;
};

type SortProps<S extends string> = {
  options: SortOption<S>[];
  value: S;
  direction: SortDirection;
  onChange: (value: S) => void;
  onToggleDirection: () => void;
};

type FilterBarProps<S extends string> = {
  search: SearchProps;
  sort: SortProps<S>;
  resultCount: { count: number; noun: string };
  activeFilters: ActiveFilter[];
  onClearAll: () => void;
  legend?: ReactNode;
  children?: ReactNode;
};

function SortControl<S extends string>({
  sort,
}: Readonly<{ sort: SortProps<S> }>) {
  const directionLabel = sort.direction === "asc" ? "Ascending" : "Descending";
  return (
    <div className="sort-control">
      <select
        className="sort-select"
        value={sort.value}
        onChange={(event) => sort.onChange(event.currentTarget.value as S)}
        aria-label="Sort by"
        title="Sort by"
      >
        {sort.options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
      <button
        type="button"
        className="sort-direction btn-icon"
        onClick={sort.onToggleDirection}
        aria-label={`${directionLabel}. Click to flip.`}
        title={directionLabel}
      >
        {sort.direction === "asc" ? "↑" : "↓"}
      </button>
    </div>
  );
}

function ActiveFilterChips({
  filters,
  onClearAll,
}: Readonly<{ filters: ActiveFilter[]; onClearAll: () => void }>) {
  if (filters.length === 0) return null;
  return (
    <ul className="active-filters" aria-label="Active filters">
      {filters.map((filter) => (
        <li key={filter.id}>
          <button
            type="button"
            className={`chip chip-active ${filter.colorClass ?? ""}`}
            onClick={filter.onRemove}
            aria-label={`Remove filter ${filter.label}`}
          >
            <span>{filter.label}</span>
            <span className="chip-remove" aria-hidden="true">
              {"×"}
            </span>
          </button>
        </li>
      ))}
      <li>
        <button type="button" className="clear-filters" onClick={onClearAll}>
          Clear all
        </button>
      </li>
    </ul>
  );
}

/**
 * One filter bar for every section: search, sort with visible direction, and
 * a collapsible panel with the section's own filters. Active filters are
 * always visible as removable chips beneath the bar.
 */
export function FilterBar<S extends string>({
  search,
  sort,
  resultCount,
  activeFilters,
  onClearAll,
  legend,
  children,
}: Readonly<FilterBarProps<S>>) {
  const [expanded, setExpanded] = useState(false);
  const activeCount = activeFilters.length;
  const nounText =
    resultCount.count === 1 ? resultCount.noun : `${resultCount.noun}s`;

  return (
    <div className="filter-bar">
      <div className="filter-bar-main">
        <div className="search-field">
          <span className="search-icon" aria-hidden="true">
            {"🔍"}
          </span>
          <input
            type="search"
            placeholder={search.placeholder}
            value={search.value}
            onChange={(event) => search.onChange(event.currentTarget.value)}
            aria-label={search.placeholder}
          />
        </div>
        <SortControl sort={sort} />
        {children && (
          <button
            type="button"
            className={`filter-toggle ${expanded ? "active" : ""} ${activeCount > 0 ? "has-filters" : ""}`}
            onClick={() => setExpanded((current) => !current)}
            aria-expanded={expanded}
          >
            <span className="filter-icon" aria-hidden="true">
              {"⚙"}
            </span>
            <span className="filter-toggle-text">Filters</span>
            {activeCount > 0 && (
              <span className="filter-badge">{activeCount}</span>
            )}
          </button>
        )}
      </div>
      {expanded && children && <div className="filter-panel">{children}</div>}
      <div className="filter-bar-status">
        <span className="content-count">
          {resultCount.count} {nounText}
        </span>
        <ActiveFilterChips filters={activeFilters} onClearAll={onClearAll} />
        {legend}
      </div>
    </div>
  );
}
