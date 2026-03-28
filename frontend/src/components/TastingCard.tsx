import { HeatDisplay, ScoreDisplay } from "./display";
import type { AuthState } from "../hooks/useAuth";
import type { TastingRecord } from "../types";

const formatDate = (value: string) => {
  if (!value) return "";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleDateString();
};

const statusLabels: Record<string, string> = {
  pending: "Queued",
  image_extracted: "Analyzing photo",
  image_enriched: "Finding product page",
  ingredients_extracted: "Reading ingredients",
  nutrition_extracted: "Reading nutrition facts",
  back_extracted: "Reading label",
  voice_transcribed: "Transcribing audio",
  voice_extracted: "Extracting scores",
  notes_formatted: "Formatting notes",
  complete: "Complete",
  error: "Error"
};

const formatStatus = (status?: string) => {
  if (!status) return "";
  return statusLabels[status] ?? status.replace(/_/g, " ");
};

const resolveType = (item: TastingRecord) => item.productType ?? "sauce";
const isDrink = (item: TastingRecord) => resolveType(item) === "drink";
const typeEmoji = (item: TastingRecord) => isDrink(item) ? "\uD83E\uDD64" : "\uD83C\uDF36\uFE0F";
const hasMediaKeys = (item: TastingRecord) =>
  Boolean(item.imageKey || item.ingredientsImageKey || item.nutritionImageKey);

function CardImage({ item, productTypeFilter, onView }: Readonly<{
  item: TastingRecord;
  productTypeFilter: string;
  onView: () => void;
}>) {
  return (
    <div className="card-image" role="button" tabIndex={0} onClick={onView} onKeyDown={(e) => { if (e.key === "Enter") onView(); }}>
      {item.imageUrl ? (
        <img src={item.imageUrl} alt={item.name} loading="lazy" />
      ) : (
        <div className="card-image-empty">{typeEmoji(item)}</div>
      )}
      {productTypeFilter === "all" && (
        <span className={`card-badge ${isDrink(item) ? "badge-drink" : "badge-sauce"}`}>
          {typeEmoji(item)}
        </span>
      )}
      {item.needsAttention && <span className="card-attention">!</span>}
    </div>
  );
}

function CardMeta({ item }: Readonly<{ item: TastingRecord }>) {
  return (
    <div className="card-meta">
      {item.style && <span className="card-tag">{item.style}</span>}
      {item.date && <span className="card-date">{formatDate(item.date)}</span>}
      {item.status && item.status !== "complete" && (
        <span className={`card-status ${item.status === "error" ? "status-error" : ""}`}>
          {formatStatus(item.status)}
        </span>
      )}
    </div>
  );
}

function CardFooter({ item, auth, rerunId, onView, onEdit, onRerun, onDelete }: Readonly<{
  item: TastingRecord;
  auth: AuthState;
  rerunId: string | null;
  onView: () => void;
  onEdit: () => void;
  onRerun: () => void;
  onDelete: () => void;
}>) {
  return (
    <footer className="card-footer">
      <button className="card-view-btn" onClick={onView}>View Details</button>
      {auth.status === "signedIn" && (
        <div className="card-actions">
          <button onClick={onEdit} title="Edit">Edit</button>
          {hasMediaKeys(item) && (
            <button onClick={onRerun} disabled={rerunId === item.id} title="Rerun AI">
              {rerunId === item.id ? "..." : "\u21BB"}
            </button>
          )}
          <button className="card-delete" onClick={onDelete} title="Delete">\u00D7</button>
        </div>
      )}
    </footer>
  );
}

type TastingCardProps = {
  item: TastingRecord;
  auth: AuthState;
  productTypeFilter: string;
  rerunId: string | null;
  onView: () => void;
  onEdit: () => void;
  onRerun: () => void;
  onDelete: () => void;
};

export function TastingCard({ item, auth, productTypeFilter, rerunId, onView, onEdit, onRerun, onDelete }: Readonly<TastingCardProps>) {
  return (
    <article className={`card ${item.needsAttention ? "needs-attention" : ""} ${isDrink(item) ? "card-drink" : "card-sauce"}`}>
      <CardImage item={item} productTypeFilter={productTypeFilter} onView={onView} />
      <div className="card-content">
        <header className="card-header">
          <h3>{item.name || "Untitled"}</h3>
          <p>{item.maker || "Unknown"}</p>
        </header>
        <div className="card-ratings">
          <ScoreDisplay value={item.score} />
          {!isDrink(item) && <HeatDisplay value={item.heatUser} />}
        </div>
        <CardMeta item={item} />
        {item.tastingNotesUser && <p className="card-notes">{item.tastingNotesUser}</p>}
        {item.status === "error" && item.processingError && (
          <div className="card-error">
            <span>Error:</span> {item.processingError}
          </div>
        )}
        <CardFooter item={item} auth={auth} rerunId={rerunId} onView={onView} onEdit={onEdit} onRerun={onRerun} onDelete={onDelete} />
      </div>
    </article>
  );
}
