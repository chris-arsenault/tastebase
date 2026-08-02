import {
  useCallback,
  useState,
  type ChangeEvent,
  type SyntheticEvent,
  type SubmitEvent,
} from "react";
import type { BookRecommendation, BookStatus } from "../types";

const statusLabels: Record<BookStatus, string> = {
  recommended: "Want to read",
  reading: "Reading",
  read: "Read",
  did_not_finish: "Did not finish",
};

const ratingValues = [1, 2, 3, 4, 5];

type BookAction = (id: string, status: BookStatus) => void;
type ReviewAction = (id: string, rating: number, writeup: string) => void;
type VisibilityAction = (id: string, isPublic: boolean) => void;

function BookCopy({ book }: Readonly<{ book: BookRecommendation }>) {
  return (
    <>
      <div className="book-card-heading">
        <div>
          <h2>{book.title}</h2>
          <p className="book-author">by {book.author}</p>
        </div>
        <span className={`book-status book-status-${book.status}`}>
          {statusLabels[book.status]}
        </span>
      </div>
      <BookMetadata book={book} />
      <p className="book-summary">{book.summary}</p>
      <div className="book-reason">
        <h3>Why Claude recommended it</h3>
        <p>{book.whyRecommended}</p>
      </div>
    </>
  );
}

function BookMetadata({ book }: Readonly<{ book: BookRecommendation }>) {
  if (book.pageCount == null && !book.purchaseLink && book.tags.length === 0) {
    return null;
  }

  return (
    <div className="book-metadata">
      <div className="book-metadata-links">
        {book.pageCount != null && <span>{book.pageCount} pages</span>}
        {book.purchaseLink && (
          <a href={book.purchaseLink} target="_blank" rel="noreferrer">
            Purchase book
          </a>
        )}
      </div>
      {book.tags.length > 0 && (
        <ul className="book-tags" aria-label="Book tags">
          {book.tags.map((tag) => (
            <li key={`${tag.key}=${tag.value}`}>
              <span>{tag.key}</span>={tag.value}
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

function SavedReview({
  book,
  showVisibility,
}: Readonly<{ book: BookRecommendation; showVisibility: boolean }>) {
  if (book.rating == null || !book.writeup) return null;
  return (
    <div className="book-saved-review">
      <div className="book-review-heading">
        <h3>What I thought</h3>
        <RatingDisplay rating={book.rating} />
      </div>
      <p>{book.writeup}</p>
      {showVisibility && book.isPublic && (
        <span className="book-public-badge">Public</span>
      )}
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
  book: BookRecommendation;
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
        onReview(book.id, rating, writeup.trim());
      }
    },
    [book.id, onReview, rating, writeup],
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
        {book.rating == null ? "Rate this book" : "Edit my review"}
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
  onVisibility,
}: Readonly<{
  book: BookRecommendation;
  saving: boolean;
  onStatus: BookAction;
  onVisibility: VisibilityAction;
}>) {
  const handleStatus = useCallback(
    (event: ChangeEvent<HTMLSelectElement>) => {
      onStatus(book.id, event.currentTarget.value as BookStatus);
    },
    [book.id, onStatus],
  );
  const handleVisibility = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      onVisibility(book.id, event.currentTarget.checked);
    },
    [book.id, onVisibility],
  );
  const hasFeedback = book.rating != null && Boolean(book.writeup.trim());

  return (
    <div className="book-owner-controls">
      <label>
        Reading status
        <select value={book.status} onChange={handleStatus} disabled={saving}>
          {Object.entries(statusLabels).map(([value, label]) => (
            <option key={value} value={value}>
              {label}
            </option>
          ))}
        </select>
      </label>
      <label className="book-public-toggle">
        <input
          type="checkbox"
          checked={book.isPublic}
          onChange={handleVisibility}
          disabled={saving || !hasFeedback}
        />
        <span>Share this review publicly</span>
      </label>
      {!hasFeedback && (
        <p className="book-visibility-note">
          Add a rating and review before sharing.
        </p>
      )}
    </div>
  );
}

type BookCardProps = {
  book: BookRecommendation;
  editable: boolean;
  saving: boolean;
  onStatus: BookAction;
  onReview: ReviewAction;
  onVisibility: VisibilityAction;
};

export function BookCard({
  book,
  editable,
  saving,
  onStatus,
  onReview,
  onVisibility,
}: Readonly<BookCardProps>) {
  return (
    <article className="book-card">
      <BookCopy book={book} />
      <SavedReview book={book} showVisibility={editable} />
      {editable && (
        <>
          <OwnerControls
            book={book}
            saving={saving}
            onStatus={onStatus}
            onVisibility={onVisibility}
          />
          <ReviewEditor book={book} saving={saving} onReview={onReview} />
        </>
      )}
    </article>
  );
}
