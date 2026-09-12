import { useCallback, type ChangeEvent } from "react";
import type { useBooks } from "../hooks/useBooks";
import {
  useBookDiscovery,
  type BookFilter,
  type BookSort,
  type SortDirection,
  type BookTagFacet,
} from "../hooks/useBookDiscovery";
import { bookTagColorClass, formatBookTagKey } from "../utils/bookTags";
import { BookCard } from "./BookCard";

type BooksHook = ReturnType<typeof useBooks>;
type BookDiscovery = ReturnType<typeof useBookDiscovery>;

const filterLabels: Record<BookFilter, string> = {
  all: "All statuses",
  recommended: "Recommended",
  reading: "Reading",
  read: "Read",
  did_not_finish: "Did not finish",
  subscribed: "Subscribed",
  cancelled: "Cancelled",
  not_interested: "Not interested",
};

const sortLabels: Record<BookSort, string> = {
  recommendedAt: "Recommendation date",
  title: "Name",
  author: "Author / publisher",
  pageCount: "Page count",
};

function BooksIntro({ booksHook }: Readonly<{ booksHook: BooksHook }>) {
  return (
    <div className="books-intro">
      <div>
        <h1>Bookshelf</h1>
        <p>
          Books, publications, and my reviews. Everything on the shelf is
          visible; use “Reviewed only” for items I’ve vetted.
        </p>
      </div>
      <button
        type="button"
        onClick={booksHook.reload}
        disabled={booksHook.loading}
      >
        {booksHook.loading ? "Loading..." : "Refresh"}
      </button>
    </div>
  );
}

function TagFilters({
  facets,
  selectedValues,
  onSelect,
  onClear,
}: Readonly<{
  facets: BookTagFacet[];
  selectedValues: Record<string, string>;
  onSelect: (key: string, value: string) => void;
  onClear: () => void;
}>) {
  if (facets.length === 0) return null;

  const hasSelections = Object.keys(selectedValues).length > 0;

  return (
    <section className="books-tag-filter" aria-labelledby="books-tags-heading">
      <div className="books-tag-filter-heading">
        <h2 id="books-tags-heading">Filter by tags</h2>
        {hasSelections && (
          <button type="button" onClick={onClear}>
            Clear tag filters
          </button>
        )}
      </div>
      <div className="books-tag-fields">
        {facets.map((facet) => (
          <TagFacetField
            key={facet.key}
            facet={facet}
            value={selectedValues[facet.key] ?? ""}
            onSelect={onSelect}
          />
        ))}
      </div>
    </section>
  );
}

function TagFacetField({
  facet,
  value,
  onSelect,
}: Readonly<{
  facet: BookTagFacet;
  value: string;
  onSelect: (key: string, value: string) => void;
}>) {
  const handleChange = useCallback(
    (event: ChangeEvent<HTMLSelectElement>) => {
      onSelect(facet.key, event.currentTarget.value);
    },
    [facet.key, onSelect],
  );

  return (
    <label className={`books-tag-field ${bookTagColorClass(facet.key)}`}>
      <span>{formatBookTagKey(facet.key)}</span>
      <select value={value} onChange={handleChange}>
        <option value="">Any</option>
        {facet.values.map((tagValue) => (
          <option key={tagValue} value={tagValue}>
            {tagValue}
          </option>
        ))}
      </select>
    </label>
  );
}

function SortDirectionButton({
  direction,
  onDirection,
}: Readonly<{
  direction: SortDirection;
  onDirection: () => void;
}>) {
  const nextDirection = direction === "asc" ? "descending" : "ascending";
  return (
    <button
      type="button"
      className="books-sort-direction"
      onClick={onDirection}
      aria-label={`Sort ${nextDirection}`}
    >
      {direction === "asc" ? "Ascending ↑" : "Descending ↓"}
    </button>
  );
}

function BooksControls({
  discovery,
}: Readonly<{
  discovery: BookDiscovery;
}>) {
  return (
    <div className="books-controls">
      <div className="books-filter">
        <div className="books-filter-fields">
          <label>
            Type
            <select
              value={discovery.kindFilter}
              onChange={discovery.handleKindFilter}
            >
              <option value="all">Books and publications</option>
              <option value="book">Books</option>
              <option value="publication">Publications</option>
            </select>
          </label>
          <label>
            Status
            <select
              value={discovery.statusFilter}
              onChange={discovery.handleStatusFilter}
            >
              {Object.entries(filterLabels).map(([value, label]) => (
                <option key={value} value={value}>
                  {label}
                </option>
              ))}
            </select>
          </label>
          <label className="books-reviewed-filter">
            <input
              type="checkbox"
              checked={discovery.reviewedOnly}
              onChange={discovery.handleReviewedOnly}
            />
            Reviewed only
          </label>
          <label>
            Sort by
            <select value={discovery.sort} onChange={discovery.handleSort}>
              {Object.entries(sortLabels).map(([value, label]) => (
                <option key={value} value={value}>
                  {label}
                </option>
              ))}
            </select>
          </label>
          <SortDirectionButton
            direction={discovery.direction}
            onDirection={discovery.toggleDirection}
          />
        </div>
        <span>
          {discovery.visibleBooks.length} item
          {discovery.visibleBooks.length === 1 ? "" : "s"}
        </span>
      </div>
      <TagFilters
        facets={discovery.availableTagFacets}
        selectedValues={discovery.selectedTagValues}
        onSelect={discovery.selectTagValue}
        onClear={discovery.clearTags}
      />
    </div>
  );
}

function emptyBookMessage(
  isOwnerView: boolean,
  hasActiveFilters: boolean,
): string {
  if (hasActiveFilters) return "No items match these filters.";
  if (isOwnerView) {
    return "No items yet. Ask your connected assistant for a book or publication recommendation.";
  }
  return "No bookshelf items yet.";
}

export function BooksSection({
  booksHook,
}: Readonly<{ booksHook: BooksHook }>) {
  const discovery = useBookDiscovery(booksHook.books);
  const emptyMessage = emptyBookMessage(
    booksHook.isOwnerView,
    discovery.hasActiveFilters,
  );

  return (
    <main className="content books-section">
      <BooksIntro booksHook={booksHook} />
      {!booksHook.loading && booksHook.books.length > 0 && (
        <BooksControls discovery={discovery} />
      )}
      {booksHook.error && <div className="error-banner">{booksHook.error}</div>}
      {booksHook.loading && <div className="loading">Loading bookshelf...</div>}
      {!booksHook.loading && discovery.visibleBooks.length === 0 && (
        <div className="empty-state">
          <span className="empty-icon">📚</span>
          <p>{emptyMessage}</p>
        </div>
      )}
      {!booksHook.loading && discovery.visibleBooks.length > 0 && (
        <div className="book-grid">
          {discovery.visibleBooks.map((book) => (
            <BookCard
              key={`${book.kind}:${book.id}`}
              book={book}
              editable={booksHook.isOwnerView}
              saving={booksHook.savingId !== ""}
              onStatus={booksHook.setStatus}
              onReview={booksHook.saveReview}
            />
          ))}
        </div>
      )}
    </main>
  );
}
