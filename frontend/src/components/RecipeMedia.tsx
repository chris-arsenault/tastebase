import { useCallback, useState } from "react";
import {
  deleteImage,
  deleteReview,
  rerunReview,
  submitVoiceReview,
  uploadRecipeImage,
} from "../api";
import { useRecorder } from "../hooks/useRecorder";
import type { RecipeImage, RecipeReview } from "../types";

const formatDate = (val: string) => {
  if (!val) return "";
  const d = new Date(val);
  return Number.isNaN(d.getTime()) ? val : d.toLocaleDateString();
};

function renderMarkdown(text: string): React.ReactNode[] {
  const unescaped = text.replace(/\\n/g, "\n");
  const paragraphs = unescaped.split(/\n\n/);
  return paragraphs.map((para, pi) => {
    const lines = para.split(/\n/);
    const children: React.ReactNode[] = [];
    lines.forEach((line, li) => {
      if (li > 0) children.push(<br key={`br-${pi}-${li}`} />);
      const parts = line.split(/(\*\*[^*]+\*\*)/g);
      parts.forEach((part, partIdx) => {
        const boldMatch = /^\*\*(.+)\*\*$/.exec(part);
        if (boldMatch) {
          children.push(
            <strong key={`b-${pi}-${li}-${partIdx}`}>{boldMatch[1]}</strong>,
          );
        } else {
          children.push(part);
        }
      });
    });
    return (
      <p key={`p-${pi}`} className="recipe-notes-paragraph">
        {children}
      </p>
    );
  });
}

export function RecipeImages({
  images,
  token,
  onDeleted,
}: Readonly<{
  images: RecipeImage[];
  token: string;
  onDeleted: () => void;
}>) {
  const extras = images.slice(1); // first image is the hero/cover
  if (extras.length === 0) return null;
  const handleDelete = (id: string) => {
    if (!token) return;
    deleteImage(id, token)
      .then(onDeleted)
      .catch(() => {});
  };
  return (
    <section className="recipe-images-section">
      <h3>Photos</h3>
      <div className="recipe-images-grid">
        {extras.map((img) => (
          <div key={img.id} className="recipe-photo-wrapper">
            <img
              src={img.imageUrl}
              alt={img.caption || "Dish"}
              className="recipe-photo"
              loading="lazy"
            />
            {token && (
              <button
                type="button"
                className="media-remove"
                onClick={() => handleDelete(img.id)}
              >
                {"\u00D7"}
              </button>
            )}
          </div>
        ))}
      </div>
    </section>
  );
}

export function RecipeReviews({
  reviews,
  token,
  onDeleted,
}: Readonly<{
  reviews: RecipeReview[];
  token: string;
  onDeleted: () => void;
}>) {
  if (reviews.length === 0) return null;
  const handleDelete = (id: string) => {
    if (!token) return;
    deleteReview(id, token)
      .then(onDeleted)
      .catch(() => {});
  };
  const handleRerun = (id: string) => {
    if (!token) return;
    rerunReview(id, token)
      .then(onDeleted)
      .catch(() => {});
  };
  return (
    <section className="recipe-reviews-section">
      <h3>Reviews</h3>
      {reviews.map((review) => (
        <div key={review.id} className="recipe-review">
          {review.status !== "complete" && review.status !== "error" && (
            <span className="card-status">
              {review.status.replace(/_/g, " ")}
            </span>
          )}
          {review.status === "error" && review.processingError && (
            <div className="card-error">{review.processingError}</div>
          )}
          {review.score !== null && (
            <div className="recipe-review-score">Score: {review.score}/10</div>
          )}
          {review.notes && renderMarkdown(review.notes)}
          <div className="recipe-review-footer">
            <span className="recipe-review-date">
              {formatDate(review.createdAt)}
            </span>
            {token && review.voiceKey && (
              <button type="button" onClick={() => handleRerun(review.id)}>
                Rerun
              </button>
            )}
            {token && (
              <button
                type="button"
                className="btn-danger"
                onClick={() => handleDelete(review.id)}
              >
                Delete
              </button>
            )}
          </div>
        </div>
      ))}
    </section>
  );
}

function ImagePicker({
  onSelect,
}: Readonly<{ onSelect: (file: File) => void }>) {
  const handleFile = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (file) onSelect(file);
    },
    [onSelect],
  );

  return (
    <label className="review-add-media">
      <span>📷</span> Add a pic
      <input type="file" accept="image/*" onChange={handleFile} hidden />
    </label>
  );
}

function VoiceSlot({
  recorder,
}: Readonly<{ recorder: ReturnType<typeof useRecorder> }>) {
  if (recorder.audioUrl) {
    return (
      <div className="voice-preview">
        {/* eslint-disable-next-line jsx-a11y/media-has-caption -- user-recorded review */}
        <audio controls src={recorder.audioUrl} />
        <button type="button" onClick={recorder.clear}>
          Remove
        </button>
      </div>
    );
  }
  if (recorder.isRecording) {
    return (
      <div className="voice-recording">
        <span className="voice-pulse" />
        <span>Recording...</span>
        <button type="button" onClick={recorder.stop}>
          Stop
        </button>
      </div>
    );
  }
  return (
    <button type="button" className="review-add-media" onClick={recorder.start}>
      <span>🎙️</span> Voice Review
    </button>
  );
}

function PhotoUpload({
  recipeId,
  token,
  onUploaded,
}: Readonly<{
  recipeId: string;
  token: string;
  onUploaded: () => void;
}>) {
  const [uploading, setUploading] = useState(false);
  const [error, setError] = useState("");

  const handleSelect = useCallback(
    (file: File) => {
      setUploading(true);
      setError("");
      uploadRecipeImage(recipeId, token, file)
        .then(onUploaded)
        .catch((e: unknown) => setError((e as Error).message))
        .finally(() => setUploading(false));
    },
    [recipeId, token, onUploaded],
  );

  if (uploading) return <span className="card-status">Uploading...</span>;
  return (
    <>
      {error && <div className="error-banner">{error}</div>}
      <ImagePicker onSelect={handleSelect} />
    </>
  );
}

function VoiceReviewCapture({
  recipeId,
  token,
  onSubmitted,
}: Readonly<{
  recipeId: string;
  token: string;
  onSubmitted: () => void;
}>) {
  const [error, setError] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const recorder = useRecorder(setError);

  const handleSubmit = useCallback(() => {
    if (!recorder.audioBlob) return;
    setSubmitting(true);
    submitVoiceReview(
      recipeId,
      token,
      recorder.audioBlob,
      recorder.audioMimeType,
    )
      .then(() => {
        recorder.clear();
        onSubmitted();
      })
      .catch((e: unknown) => setError((e as Error).message))
      .finally(() => setSubmitting(false));
  }, [recipeId, token, recorder, onSubmitted]);

  return (
    <>
      {error && <div className="error-banner">{error}</div>}
      <VoiceSlot recorder={recorder} />
      {recorder.audioBlob && (
        <button
          type="button"
          className="btn-submit review-submit"
          onClick={handleSubmit}
          disabled={submitting}
        >
          {submitting ? "Submitting..." : "Submit Review"}
        </button>
      )}
    </>
  );
}

export function ReviewCapture({
  recipeId,
  token,
  onSubmitted,
}: Readonly<{
  recipeId: string;
  token: string;
  onSubmitted: () => void;
}>) {
  return (
    <section className="recipe-review-capture">
      <div className="review-capture-controls">
        <div className="review-media-slot">
          <h3>Add Photo</h3>
          <PhotoUpload
            recipeId={recipeId}
            token={token}
            onUploaded={onSubmitted}
          />
        </div>
        <div className="review-voice-slot">
          <h3>Voice Review</h3>
          <VoiceReviewCapture
            recipeId={recipeId}
            token={token}
            onSubmitted={onSubmitted}
          />
        </div>
      </div>
    </section>
  );
}
