import {
  useCallback,
  useState,
  type ChangeEvent,
  type SyntheticEvent,
  type SubmitEvent,
} from "react";
import type { ShelfItem, ShelfStatus } from "../types";
import { PublicationDetails } from "./PublicationDetails";
import {
  bookStatusLabels,
  publicationStatusLabels,
  shelfStatusLabels,
  creator,
  isReviewed,
} from "../utils/shelf";
import { bookTagColorClass, formatBookTagKey } from "../utils/bookTags";

const ratingValues = [1, 2, 3, 4, 5];

type BookAction = (item: ShelfItem, status: ShelfStatus) => void;
type ReviewAction = (item: ShelfItem, rating: number, writeup: string) => void;

function BookCopy({ book }: Readonly<{ book: ShelfItem }>) {
  return (
    <>
      <div className="book-card-heading">
        <div>
          <h2>{book.title}</h2>
          {creator(book) && <p className="book-author">{creator(book)}</p>}
        </div>
        <span className={`book-status book-status-${book.status}`}>
          {shelfStatusLabels[book.status]}
        </span>
      </div>
      <span className="book-review-state">
        {isReviewed(book) ? "Reviewed" : "Not yet reviewed"}
      </span>
      <BookMetadata book={book} />
      <p className="book-summary">{book.summary}</p>
      <div className="book-reason">
        <h3>Why it’s on my shelf</h3>
        <p>{book.whyRecommended}</p>
      </div>
    </>
  );
}

function BookMetadata({ book }: Readonly<{ book: ShelfItem }>) {
  return (
    <div className="book-metadata">
      {book.kind === "publication" && <PublicationDetails publication={book} />}
      {book.kind === "book" && (
        <div className="book-metadata-links">
          {book.pageCount != null && <span>{book.pageCount} pages</span>}
          {book.purchaseLink && (
            <a href={book.purchaseLink} target="_blank" rel="noreferrer">
              Purchase book
            </a>
          )}
        </div>
      )}
      {book.tags.length > 0 && (
        <ul className="book-tags" aria-label="Tags">
          {book.tags.map((tag) => (
            <li
              key={`${tag.key}=${tag.value}`}
              className={bookTagColorClass(tag.key)}
              aria-label={`${formatBookTagKey(tag.key)}: ${tag.value}`}
            >
              {tag.value}
            </li>
          ))}
        </ul>
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
      <p>{book.writeup}</p>
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
        <button
          type="submit"
          disabled={saving || rating === 0 || !writeup.trim()}
        >
          {saving ? "Saving..." : "Save review"}
        </button>
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

type BookCardProps = {
  book: ShelfItem;
  editable: boolean;
  saving: boolean;
  onStatus: BookAction;
  onReview: ReviewAction;
};

export function BookCard({
  book,
  editable,
  saving,
  onStatus,
  onReview,
}: Readonly<BookCardProps>) {
  return (
    <article className="book-card">
      <BookCopy book={book} />
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
