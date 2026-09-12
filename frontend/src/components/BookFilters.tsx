import { useState } from "react";
import type { BookFilter, KindFilter } from "../hooks/useBookDiscovery";
import type { ShelfStatus } from "../types";
import { bookTagColorClass, formatBookTagKey } from "../utils/bookTags";
import {
  bookStatusLabels,
  publicationStatusLabels,
  type TagFacet,
  type TagSelection,
} from "../utils/shelf";

const kindOptions: { value: KindFilter; label: string }[] = [
  { value: "all", label: "All" },
  { value: "book", label: "Books" },
  { value: "publication", label: "Publications" },
];

const visibleValueLimit = 8;
const noValues: string[] = [];

function statusOptions(kind: KindFilter): [ShelfStatus, string][] {
  const books = Object.entries(bookStatusLabels) as [ShelfStatus, string][];
  const publications = Object.entries(publicationStatusLabels) as [
    ShelfStatus,
    string,
  ][];
  if (kind === "book") return books;
  if (kind === "publication") return publications;
  return [
    ...books,
    ...publications.filter(([value]) => value !== "recommended"),
  ];
}

export function KindToggle({
  value,
  onChange,
}: Readonly<{ value: KindFilter; onChange: (kind: KindFilter) => void }>) {
  return (
    <div className="filter-group">
      <span className="filter-group-label">Type</span>
      <div className="segmented segmented-light" role="group" aria-label="Type">
        {kindOptions.map((option) => (
          <button
            key={option.value}
            type="button"
            className={value === option.value ? "active" : ""}
            aria-pressed={value === option.value}
            onClick={() => onChange(option.value)}
          >
            {option.label}
          </button>
        ))}
      </div>
    </div>
  );
}

export function StatusChips({
  kind,
  value,
  reviewedOnly,
  onChange,
  onReviewedOnly,
}: Readonly<{
  kind: KindFilter;
  value: BookFilter;
  reviewedOnly: boolean;
  onChange: (status: BookFilter) => void;
  onReviewedOnly: (reviewedOnly: boolean) => void;
}>) {
  return (
    <div className="filter-group">
      <span className="filter-group-label">Status</span>
      <div className="chip-row" role="group" aria-label="Status">
        {statusOptions(kind).map(([status, label]) => (
          <button
            key={status}
            type="button"
            className={`chip ${value === status ? "chip-selected" : ""}`}
            aria-pressed={value === status}
            onClick={() => onChange(value === status ? "all" : status)}
          >
            {label}
          </button>
        ))}
        <button
          type="button"
          className={`chip chip-reviewed ${reviewedOnly ? "chip-selected" : ""}`}
          aria-pressed={reviewedOnly}
          onClick={() => onReviewedOnly(!reviewedOnly)}
        >
          {"✓"} Reviewed only
        </button>
      </div>
    </div>
  );
}

function TagRow({
  facet,
  selected,
  onToggle,
}: Readonly<{
  facet: TagFacet;
  selected: string[];
  onToggle: (key: string, value: string) => void;
}>) {
  const [showAll, setShowAll] = useState(false);
  const overflow = facet.values.length - visibleValueLimit;
  const values =
    showAll || overflow <= 0
      ? facet.values
      : facet.values.filter(
          (entry, index) =>
            index < visibleValueLimit || selected.includes(entry.value),
        );
  const colorClass = bookTagColorClass(facet.key);

  return (
    <div className={`tag-row ${colorClass}`}>
      <span className="tag-row-key">{formatBookTagKey(facet.key)}</span>
      <div className="chip-row" role="group" aria-label={facet.key}>
        {values.map((entry) => {
          const isSelected = selected.includes(entry.value);
          return (
            <button
              key={entry.value}
              type="button"
              className={`chip chip-tag ${isSelected ? "chip-selected" : ""} ${entry.count === 0 ? "chip-empty" : ""}`}
              aria-pressed={isSelected}
              onClick={() => onToggle(facet.key, entry.value)}
            >
              <span>{entry.value}</span>
              <span className="chip-count">{entry.count}</span>
            </button>
          );
        })}
        {overflow > 0 && (
          <button
            type="button"
            className="chip chip-more"
            onClick={() => setShowAll((current) => !current)}
            aria-expanded={showAll}
          >
            {showAll ? "Fewer" : `+${overflow} more`}
          </button>
        )}
      </div>
    </div>
  );
}

export function TagRows({
  facets,
  selected,
  onToggle,
}: Readonly<{
  facets: TagFacet[];
  selected: TagSelection;
  onToggle: (key: string, value: string) => void;
}>) {
  if (facets.length === 0) return null;
  return (
    <div className="filter-group">
      <span className="filter-group-label">Tags</span>
      <div className="tag-rows">
        {facets.map((facet) => (
          <TagRow
            key={facet.key}
            facet={facet}
            selected={selected[facet.key] ?? noValues}
            onToggle={onToggle}
          />
        ))}
      </div>
    </div>
  );
}
