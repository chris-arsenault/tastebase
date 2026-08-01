import { useMemo, useState, type ChangeEvent } from "react";
import type { useBooks } from "../hooks/useBooks";
import type { BookStatus } from "../types";
import { BookCard } from "./BookCard";

type BooksHook = ReturnType<typeof useBooks>;
type BookFilter = "all" | BookStatus;

const filterLabels: Record<BookFilter, string> = {
  all: "All books",
  recommended: "Want to read",
  reading: "Reading",
  read: "Read",
  did_not_finish: "Did not finish",
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

export function BooksSection({
  booksHook,
}: Readonly<{ booksHook: BooksHook }>) {
  const [filter, setFilter] = useState<BookFilter>("all");
  const visibleBooks = useMemo(
    () =>
      !booksHook.isOwnerView || filter === "all"
        ? booksHook.books
        : booksHook.books.filter((book) => book.status === filter),
    [booksHook.books, booksHook.isOwnerView, filter],
  );
  const handleFilter = (event: ChangeEvent<HTMLSelectElement>) => {
    setFilter(event.currentTarget.value as BookFilter);
  };

  return (
    <main className="content books-section">
      <BooksIntro booksHook={booksHook} />
      {booksHook.isOwnerView && (
        <div className="books-filter">
          <label>
            Show
            <select value={filter} onChange={handleFilter}>
              {Object.entries(filterLabels).map(([value, label]) => (
                <option key={value} value={value}>
                  {label}
                </option>
              ))}
            </select>
          </label>
          <span>
            {visibleBooks.length} book{visibleBooks.length === 1 ? "" : "s"}
          </span>
        </div>
      )}
      {booksHook.error && <div className="error-banner">{booksHook.error}</div>}
      {booksHook.loading && <div className="loading">Loading books...</div>}
      {!booksHook.loading && visibleBooks.length === 0 && (
        <div className="empty-state">
          <span className="empty-icon">📚</span>
          <p>
            {booksHook.isOwnerView
              ? "No books yet. Ask Claude for a recommendation to get started."
              : "No book reviews yet."}
          </p>
        </div>
      )}
      {!booksHook.loading && visibleBooks.length > 0 && (
        <div className="book-grid">
          {visibleBooks.map((book) => (
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
