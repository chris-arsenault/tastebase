import type { useBooks } from "../hooks/useBooks";
import {
  bookTagToken,
  useBookDiscovery,
  type BookFilter,
  type BookSort,
  type SortDirection,
} from "../hooks/useBookDiscovery";
import type { BookTag } from "../types";
import { BookCard } from "./BookCard";

type BooksHook = ReturnType<typeof useBooks>;
type BookDiscovery = ReturnType<typeof useBookDiscovery>;

const filterLabels: Record<BookFilter, string> = {
  all: "All books",
  recommended: "Want to read",
  reading: "Reading",
  read: "Read",
  did_not_finish: "Did not finish",
};

const sortLabels: Record<BookSort, string> = {
  recommendedAt: "Recommendation date",
  title: "Name",
  author: "Author",
  pageCount: "Page count",
};

function BooksIntro({ booksHook }: Readonly<{ booksHook: BooksHook }>) {
  return (
    <div className="books-intro">
      <div>
        <h1>{booksHook.isOwnerView ? "My books" : "Book reviews"}</h1>
        <p>
          {booksHook.isOwnerView
            ? "Recommendations, reading progress, and reviews in one place."
            : "My notes on books I've read."}
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
  tags,
  selected,
  onToggle,
  onClear,
}: Readonly<{
  tags: BookTag[];
  selected: Set<string>;
  onToggle: (tag: BookTag) => void;
  onClear: () => void;
}>) {
  if (tags.length === 0) return null;

  return (
    <fieldset className="books-tag-filter">
      <legend>Filter by tags</legend>
      <div className="books-tag-options">
        {tags.map((tag) => {
          const token = bookTagToken(tag);
          const isSelected = selected.has(token);
          return (
            <button
              key={token}
              type="button"
              className={isSelected ? "selected" : ""}
              aria-pressed={isSelected}
              onClick={() => onToggle(tag)}
            >
              <span>{tag.key}</span>={tag.value}
            </button>
          );
        })}
        {selected.size > 0 && (
          <button type="button" className="books-tag-clear" onClick={onClear}>
            Clear tags
          </button>
        )}
      </div>
      <p>Values within one key match either; different keys must all match.</p>
    </fieldset>
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
  isOwnerView,
  discovery,
}: Readonly<{
  isOwnerView: boolean;
  discovery: BookDiscovery;
}>) {
  return (
    <div className="books-controls">
      <div className="books-filter">
        <div className="books-filter-fields">
          {isOwnerView && (
            <label>
              Show
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
          )}
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
          {discovery.visibleBooks.length} book
          {discovery.visibleBooks.length === 1 ? "" : "s"}
        </span>
      </div>
      <TagFilters
        tags={discovery.availableTags}
        selected={discovery.selectedTags}
        onToggle={discovery.toggleTag}
        onClear={discovery.clearTags}
      />
    </div>
  );
}

function emptyBookMessage(
  isOwnerView: boolean,
  hasActiveFilters: boolean,
): string {
  if (hasActiveFilters) return "No books match these filters.";
  if (isOwnerView) {
    return "No books yet. Ask Claude for a recommendation to get started.";
  }
  return "No book reviews yet.";
}

export function BooksSection({
  booksHook,
}: Readonly<{ booksHook: BooksHook }>) {
  const discovery = useBookDiscovery(booksHook.books, booksHook.isOwnerView);
  const emptyMessage = emptyBookMessage(
    booksHook.isOwnerView,
    discovery.hasActiveFilters,
  );

  return (
    <main className="content books-section">
      <BooksIntro booksHook={booksHook} />
      {!booksHook.loading && booksHook.books.length > 0 && (
        <BooksControls
          isOwnerView={booksHook.isOwnerView}
          discovery={discovery}
        />
      )}
      {booksHook.error && <div className="error-banner">{booksHook.error}</div>}
      {booksHook.loading && <div className="loading">Loading books...</div>}
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
              key={book.id}
              book={book}
              editable={booksHook.isOwnerView}
              saving={booksHook.savingId === book.id}
              onStatus={booksHook.setStatus}
              onReview={booksHook.saveReview}
              onVisibility={booksHook.setVisibility}
            />
          ))}
        </div>
      )}
    </main>
  );
}
