import {
  useCallback,
  useState,
  type ChangeEvent,
  type SyntheticEvent,
  type SubmitEvent,
} from "react";
import type { ShelfItem, ShelfStatus } from "../types";
import { PublicationDetails } from "./PublicationDetails";
import { ClampText, MetaChip, StatusDot, type SignalState } from "./signals";
import { Tooltip } from "./Tooltip";
import {
  bookStatusLabels,
  publicationStatusLabels,
  shelfStatusLabels,
  creator,
  isReviewed,
  type TagSelection,
} from "../utils/shelf";
import { bookTagColorClass, formatBookTagKey } from "../utils/bookTags";

const ratingValues = [1, 2, 3, 4, 5];

type BookAction = (item: ShelfItem, status: ShelfStatus) => void;
type ReviewAction = (item: ShelfItem, rating: number, writeup: string) => void;
type TagAction = (key: string, value: string) => void;

const statusSignal: Record<ShelfStatus, SignalState> = {
  recommended: "info",
  reading: "active",
  read: "ok",
  subscribed: "ok",
  did_not_finish: "muted",
  cancelled: "muted",
  not_interested: "muted",
};

const statusDetail: Record<ShelfStatus, string> = {
  recommended: "On the shelf, not started yet",
  reading: "Currently reading",
  read: "Finished reading",
  subscribed: "Currently subscribed",
  did_not_finish: "Started but set aside",
  cancelled: "Subscription cancelled",
  not_interested: "Passed on this one",
};

function ReviewMark({ book }: Readonly<{ book: ShelfItem }>) {
  if (isReviewed(book)) {
    return (
      <Tooltip label={`Reviewed: ${book.rating} out of 5`}>
        <span className="review-mark review-mark-done">
          <span aria-hidden="true">{"✓"}</span>
          <span className="visually-hidden">Reviewed</span>
        </span>
      </Tooltip>
    );
  }
  return (
    <Tooltip label="Not yet reviewed">
      <span className="review-mark review-mark-pending">
        <span aria-hidden="true">{"○"}</span>
        <span className="visually-hidden">Not yet reviewed</span>
      </span>
    </Tooltip>
  );
}

function BookHeading({ book }: Readonly<{ book: ShelfItem }>) {
  return (
    <div className="book-card-heading">
      <div className="book-card-title">
        <h2>{book.title}</h2>
        {creator(book) && <p className="book-author">{creator(book)}</p>}
      </div>
      <div className="book-card-signals">
        <ReviewMark book={book} />
        <StatusDot
          state={statusSignal[book.status]}
          text={shelfStatusLabels[book.status]}
          detail={statusDetail[book.status]}
        />
      </div>
    </div>
  );
}

function BookTags({
  book,
  selectedTags,
  onTagClick,
}: Readonly<{
  book: ShelfItem;
  selectedTags: TagSelection;
  onTagClick?: TagAction;
}>) {
  if (book.tags.length === 0) return null;
  return (
    <ul className="book-tags" aria-label="Tags">
      {book.tags.map((tag) => {
        const selected = selectedTags[tag.key]?.includes(tag.value) ?? false;
        const label = `${formatBookTagKey(tag.key)}: ${tag.value}`;
        return (
          <li key={`${tag.key}=${tag.value}`}>
            <Tooltip label={selected ? `${label} (filtering)` : label}>
              <button
                type="button"
                className={`chip chip-tag ${bookTagColorClass(tag.key)} ${selected ? "chip-selected" : ""}`}
                aria-pressed={selected}
                aria-label={`${label}. Filter by this tag.`}
                onClick={() => onTagClick?.(tag.key, tag.value)}
              >
                {tag.value}
              </button>
            </Tooltip>
          </li>
        );
      })}
    </ul>
  );
}

function BookMetadata({ book }: Readonly<{ book: ShelfItem }>) {
  if (book.kind === "publication") {
    return <PublicationDetails publication={book} />;
  }
  if (book.pageCount == null && !book.purchaseLink) return null;
  return (
    <div className="meta-row">
      {book.pageCount != null && (
        <MetaChip label="Page count" icon={"📖"}>
          <span className="tabular">{book.pageCount}</span> pages
        </MetaChip>
      )}
      {book.purchaseLink && (
        <MetaChip label="Purchase link" href={book.purchaseLink}>
          Buy
        </MetaChip>
      )}
    </div>
  );
}

function RatingDisplay({ rating }: Readonly<{ rating: number }>) {
  return (
    <span className="book-rating" aria-label={`${rating} out of 5 stars`}>
      {ratingValues.map((value) => (
        <span
          key={value}
          className={value <= rating ? "filled" : ""}
          aria-hidden="true"
        >
          ★
        </span>
      ))}
    </span>
  );
}

function SavedReview({ book }: Readonly<{ book: ShelfItem }>) {
  if (book.rating == null || !isReviewed(book)) return null;
  return (
    <div className="book-saved-review">
      <div className="book-review-heading">
        <h3>What I thought</h3>
        <RatingDisplay rating={book.rating} />
      </div>
      <ClampText text={book.writeup} />
    </div>
  );
}

function BookRatingInput({
  bookId,
  rating,
  onChange,
}: Readonly<{
  bookId: string;
  rating: number;
  onChange: (event: ChangeEvent<HTMLInputElement>) => void;
}>) {
  return (
    <fieldset className="book-rating-input">
      <legend>Rating</legend>
      {ratingValues.map((value) => (
        <label key={value} className={value <= rating ? "selected" : ""}>
          <input
            type="radio"
            name={`rating-${bookId}`}
            value={value}
            checked={rating === value}
            onChange={onChange}
          />
          <span aria-hidden="true">★</span>
          <span className="visually-hidden">{value} stars</span>
        </label>
      ))}
    </fieldset>
  );
}

function ReviewEditor({
  book,
  saving,
  onReview,
}: Readonly<{
  book: ShelfItem;
  saving: boolean;
  onReview: ReviewAction;
}>) {
  const [rating, setRating] = useState(book.rating ?? 0);
  const [writeup, setWriteup] = useState(book.writeup);
  const [expanded, setExpanded] = useState(book.rating == null);
  const handleRating = useCallback((event: ChangeEvent<HTMLInputElement>) => {
    setRating(Number(event.currentTarget.value));
  }, []);
  const handleWriteup = useCallback(
    (event: ChangeEvent<HTMLTextAreaElement>) => {
      setWriteup(event.currentTarget.value);
    },
    [],
  );
  const handleSubmit = useCallback(
    (event: SubmitEvent<HTMLFormElement>) => {
      event.preventDefault();
      if (rating > 0 && writeup.trim()) {
        onReview(book, rating, writeup.trim());
      }
    },
    [book, onReview, rating, writeup],
  );
  const handleToggle = useCallback(
    (event: SyntheticEvent<HTMLDetailsElement>) => {
      setExpanded(event.currentTarget.open);
    },
    [],
  );

  return (
    <details
      className="book-review-editor"
      open={expanded}
      onToggle={handleToggle}
    >
      <summary>
        {book.rating == null ? "Write a review" : "Edit my review"}
      </summary>
      <form onSubmit={handleSubmit}>
        <BookRatingInput
          bookId={book.id}
          rating={rating}
          onChange={handleRating}
        />
        <label className="book-writeup-field">
          What did you think?
          <textarea
            value={writeup}
            onChange={handleWriteup}
            rows={4}
            maxLength={6000}
            placeholder="What stood out? What worked—or didn’t?"
            required
          />
        </label>
        <div className="form-actions">
          <button
            type="submit"
            className="btn-primary"
            disabled={saving || rating === 0 || !writeup.trim()}
          >
            {saving ? "Saving..." : "Save review"}
          </button>
        </div>
      </form>
    </details>
  );
}

function OwnerControls({
  book,
  saving,
  onStatus,
}: Readonly<{
  book: ShelfItem;
  saving: boolean;
  onStatus: BookAction;
}>) {
  const handleStatus = useCallback(
    (event: ChangeEvent<HTMLSelectElement>) => {
      onStatus(book, event.currentTarget.value as ShelfStatus);
    },
    [book, onStatus],
  );
  const statusLabels =
    book.kind === "book" ? bookStatusLabels : publicationStatusLabels;

  return (
    <div className="book-owner-controls">
      <label>
        {book.kind === "book" ? "Reading status" : "Subscription status"}
        <select value={book.status} onChange={handleStatus} disabled={saving}>
          {Object.entries(statusLabels).map(([value, label]) => (
            <option key={value} value={value}>
              {label}
            </option>
          ))}
        </select>
      </label>
    </div>
  );
}

const noTags: TagSelection = {};

type BookCardProps = {
  book: ShelfItem;
  editable: boolean;
  saving: boolean;
  selectedTags?: TagSelection;
  onStatus: BookAction;
  onReview: ReviewAction;
  onTagClick?: TagAction;
};

export function BookCard({
  book,
  editable,
  saving,
  selectedTags = noTags,
  onStatus,
  onReview,
  onTagClick,
}: Readonly<BookCardProps>) {
  return (
    <article className="book-card">
      <BookHeading book={book} />
      <BookMetadata book={book} />
      <BookTags
        book={book}
        selectedTags={selectedTags}
        onTagClick={onTagClick}
      />
      <ClampText text={book.summary} className="book-summary" />
      <div className="book-reason">
        <h3>Why it’s on my shelf</h3>
        <ClampText text={book.whyRecommended} />
      </div>
      <SavedReview book={book} />
      {editable && (
        <>
          <OwnerControls book={book} saving={saving} onStatus={onStatus} />
          <ReviewEditor book={book} saving={saving} onReview={onReview} />
        </>
      )}
    </article>
  );
}
