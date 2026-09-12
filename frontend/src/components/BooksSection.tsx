import { useCallback, useMemo } from "react";
import type { useBooks } from "../hooks/useBooks";
import {
  useBookDiscovery,
  type BookFilter,
  type BookSort,
  type KindFilter,
} from "../hooks/useBookDiscovery";
import { bookTagColorClass } from "../utils/bookTags";
import { shelfStatusLabels } from "../utils/shelf";
import { BookCard } from "./BookCard";
import { KindToggle, StatusChips, TagRows } from "./BookFilters";
import { FilterBar, type ActiveFilter } from "./FilterBar";
import { Legend } from "./signals";

type BooksHook = ReturnType<typeof useBooks>;
type BookDiscovery = ReturnType<typeof useBookDiscovery>;

const sortOptions: { value: BookSort; label: string }[] = [
  { value: "recommendedAt", label: "Date" },
  { value: "title", label: "Name" },
  { value: "author", label: "Author" },
  { value: "pageCount", label: "Pages" },
];

const kindLabels: Record<KindFilter, string> = {
  all: "All",
  book: "Books",
  publication: "Publications",
};

const bookLegend = [
  { glyph: <span className="status-dot state-info" />, label: "want to read" },
  { glyph: <span className="status-dot state-active" />, label: "reading" },
  { glyph: <span className="status-dot state-ok" />, label: "read" },
  { glyph: <span className="status-dot state-muted" />, label: "set aside" },
  { glyph: "✓", label: "reviewed" },
  { glyph: "↗", label: "external link" },
];

function useActiveChips(discovery: BookDiscovery): ActiveFilter[] {
  const { state, patch, toggleTag } = discovery;
  return useMemo(() => {
    const chips: ActiveFilter[] = [];
    if (state.kind !== "all") {
      chips.push({
        id: "kind",
        label: kindLabels[state.kind],
        onRemove: () => patch({ kind: "all" }),
      });
    }
    if (state.status !== "all") {
      chips.push({
        id: "status",
        label: shelfStatusLabels[state.status],
        onRemove: () => patch({ status: "all" }),
      });
    }
    if (state.reviewedOnly) {
      chips.push({
        id: "reviewed",
        label: "Reviewed only",
        onRemove: () => patch({ reviewedOnly: false }),
      });
    }
    for (const [key, values] of Object.entries(state.tags)) {
      for (const value of values) {
        chips.push({
          id: `tag:${key}:${value}`,
          label: value,
          colorClass: `chip-tag ${bookTagColorClass(key)}`,
          onRemove: () => toggleTag(key, value),
        });
      }
    }
    return chips;
  }, [
    patch,
    state.kind,
    state.reviewedOnly,
    state.status,
    state.tags,
    toggleTag,
  ]);
}

function BooksFilterBar({ discovery }: Readonly<{ discovery: BookDiscovery }>) {
  const { state, patch, setSort, toggleDirection, toggleTag, clearFilters } =
    discovery;
  const chips = useActiveChips(discovery);
  const search = useMemo(
    () => ({
      value: state.search,
      placeholder: "Search titles, authors, summaries...",
      onChange: (value: string) => patch({ search: value }),
    }),
    [patch, state.search],
  );
  const sort = useMemo(
    () => ({
      options: sortOptions,
      value: state.sort,
      direction: state.direction,
      onChange: setSort,
      onToggleDirection: toggleDirection,
    }),
    [setSort, state.direction, state.sort, toggleDirection],
  );
  const resultCount = useMemo(
    () => ({ count: discovery.visibleBooks.length, noun: "item" }),
    [discovery.visibleBooks.length],
  );
  const onKind = useCallback(
    (kind: KindFilter) => patch({ kind, status: "all" }),
    [patch],
  );
  const onStatus = useCallback(
    (status: BookFilter) => patch({ status }),
    [patch],
  );
  const onReviewedOnly = useCallback(
    (reviewedOnly: boolean) => patch({ reviewedOnly }),
    [patch],
  );
  const legend = useMemo(() => <Legend items={bookLegend} />, []);

  return (
    <FilterBar
      search={search}
      sort={sort}
      resultCount={resultCount}
      activeFilters={chips}
      onClearAll={clearFilters}
      legend={legend}
    >
      <KindToggle value={state.kind} onChange={onKind} />
      <StatusChips
        kind={state.kind}
        value={state.status}
        reviewedOnly={state.reviewedOnly}
        onChange={onStatus}
        onReviewedOnly={onReviewedOnly}
      />
      <TagRows
        facets={discovery.tagFacets}
        selected={state.tags}
        onToggle={toggleTag}
      />
    </FilterBar>
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

function BookGrid({
  booksHook,
  discovery,
}: Readonly<{ booksHook: BooksHook; discovery: BookDiscovery }>) {
  const { toggleTag } = discovery;
  const onTagClick = useCallback(
    (key: string, value: string) => toggleTag(key, value),
    [toggleTag],
  );
  if (booksHook.loading) {
    return <div className="loading">Loading bookshelf...</div>;
  }
  if (discovery.visibleBooks.length === 0) {
    return (
      <div className="empty-state">
        <span className="empty-icon">📚</span>
        <p>
          {emptyBookMessage(booksHook.isOwnerView, discovery.hasActiveFilters)}
        </p>
      </div>
    );
  }
  return (
    <div className="book-grid">
      {discovery.visibleBooks.map((book) => (
        <BookCard
          key={`${book.kind}:${book.id}`}
          book={book}
          editable={booksHook.isOwnerView}
          saving={booksHook.savingId !== ""}
          selectedTags={discovery.state.tags}
          onStatus={booksHook.setStatus}
          onReview={booksHook.saveReview}
          onTagClick={onTagClick}
        />
      ))}
    </div>
  );
}

export function BooksSection({
  booksHook,
}: Readonly<{ booksHook: BooksHook }>) {
  const discovery = useBookDiscovery(booksHook.books);

  return (
    <>
      <BooksFilterBar discovery={discovery} />
      <main className="content books-section">
        <div className="books-intro">
          <h1>Bookshelf</h1>
          <p>
            Books and publications recommended to me, with my reviews once
            I&apos;ve read them. Tap a tag to filter by it.
          </p>
        </div>
        {booksHook.error && (
          <div className="error-banner">{booksHook.error}</div>
        )}
        <BookGrid booksHook={booksHook} discovery={discovery} />
      </main>
    </>
  );
}
