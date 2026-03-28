import { HeatDisplay, ScoreDisplay } from "./display";
import type { NutritionFacts, TastingRecord } from "../types";

const formatDate = (value: string) => {
  if (!value) return "";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleDateString();
};

function HeroSection({ record, productType, onClose }: Readonly<{
  record: TastingRecord;
  productType: string;
  onClose: () => void;
}>) {
  return (
    <div className="view-hero-section">
      {record.imageUrl ? (
        <img className="view-hero-img" src={record.imageUrl} alt={record.name || "Tasting"} />
      ) : (
        <div className="view-hero-empty">
          <span>{productType === "drink" ? "\uD83E\uDD64" : "\uD83C\uDF36\uFE0F"}</span>
        </div>
      )}
      <button type="button" className="view-close" onClick={onClose}>{"\u00D7"}</button>
    </div>
  );
}

function HeaderInfo({ record, productType }: Readonly<{ record: TastingRecord; productType: string }>) {
  return (
    <header className="view-header-info">
      <div className="view-title-group">
        <h2>{record.name || "Untitled"}</h2>
        <span className="view-maker">{record.maker || "Unknown maker"}</span>
      </div>
      <div className="view-ratings-inline">
        <div className="view-rating-item">
          <ScoreDisplay value={record.score} />
        </div>
        {productType !== "drink" && record.heatUser !== null && (
          <div className="view-rating-item view-heat">
            <HeatDisplay value={record.heatUser} />
          </div>
        )}
      </div>
    </header>
  );
}

function QuickInfo({ record, productType }: Readonly<{ record: TastingRecord; productType: string }>) {
  return (
    <div className="view-quick-info">
      {record.style && <span className="view-tag">{record.style}</span>}
      {record.date && <span className="view-date">{formatDate(record.date)}</span>}
      {productType !== "drink" && record.heatVendor !== null && (
        <span className="view-vendor-heat">
          Vendor: <HeatDisplay value={record.heatVendor} />
        </span>
      )}
      {record.productUrl && (
        <a className="view-product-link" href={record.productUrl} target="_blank" rel="noreferrer">
          Product Page {"\u2192"}
        </a>
      )}
    </div>
  );
}

function NotesSection({ record }: Readonly<{ record: TastingRecord }>) {
  if (!record.tastingNotesUser && !record.tastingNotesVendor) return null;
  return (
    <div className="view-notes">
      {record.tastingNotesUser && (
        <div className="view-note">
          <span className="view-note-label">Tasting Notes</span>
          <p>{record.tastingNotesUser}</p>
        </div>
      )}
      {record.tastingNotesVendor && (
        <div className="view-note view-note-vendor">
          <span className="view-note-label">Vendor Description</span>
          <p>{record.tastingNotesVendor}</p>
        </div>
      )}
    </div>
  );
}

function NutritionDetails({ nutritionFacts }: Readonly<{ nutritionFacts: NutritionFacts }>) {
  return (
    <details className="view-details-block" open>
      <summary>Nutrition Facts</summary>
      <dl className="view-nutrition">
        {nutritionFacts.servingSize && <><dt>Serving</dt><dd>{nutritionFacts.servingSize}</dd></>}
        {nutritionFacts.calories !== undefined && <><dt>Calories</dt><dd>{nutritionFacts.calories}</dd></>}
        {nutritionFacts.sodium && <><dt>Sodium</dt><dd>{nutritionFacts.sodium}</dd></>}
        {nutritionFacts.totalCarbs && <><dt>Carbs</dt><dd>{nutritionFacts.totalCarbs}</dd></>}
        {nutritionFacts.sugars && <><dt>Sugars</dt><dd>{nutritionFacts.sugars}</dd></>}
        {nutritionFacts.protein && <><dt>Protein</dt><dd>{nutritionFacts.protein}</dd></>}
      </dl>
    </details>
  );
}

function ExtractedData({ record }: Readonly<{ record: TastingRecord }>) {
  const hasIngredients = record.ingredients && record.ingredients.length > 0;
  if (!hasIngredients && !record.nutritionFacts) return null;
  return (
    <div className="view-details-row">
      {hasIngredients && (
        <details className="view-details-block" open>
          <summary>Ingredients</summary>
          <div className="view-ingredients">
            {record.ingredients!.map((ing, i) => (
              <span key={i}>{ing}</span>
            ))}
          </div>
        </details>
      )}
      {record.nutritionFacts && <NutritionDetails nutritionFacts={record.nutritionFacts} />}
    </div>
  );
}

function SourceImages({ record }: Readonly<{ record: TastingRecord }>) {
  if (!record.ingredientsImageUrl && !record.nutritionImageUrl) return null;
  return (
    <details className="view-source-images">
      <summary>View Source Images</summary>
      <div className="view-media-grid">
        {record.ingredientsImageUrl && (
          <div className="view-media-slot">
            <img src={record.ingredientsImageUrl} alt="Ingredients label" />
            <span>Ingredients</span>
          </div>
        )}
        {record.nutritionImageUrl && (
          <div className="view-media-slot">
            <img src={record.nutritionImageUrl} alt="Nutrition facts" />
            <span>Nutrition</span>
          </div>
        )}
      </div>
    </details>
  );
}

type ViewModalProps = {
  record: TastingRecord;
  onClose: () => void;
};

export function ViewModal({ record, onClose }: Readonly<ViewModalProps>) {
  const productType = record.productType ?? "sauce";

  return (
    <div className="view-overlay">
      <article className="view-modal" role="dialog" aria-modal="true">
        <HeroSection record={record} productType={productType} onClose={onClose} />
        <div className="view-content">
          <HeaderInfo record={record} productType={productType} />
          <QuickInfo record={record} productType={productType} />
          <NotesSection record={record} />
          <ExtractedData record={record} />
          <SourceImages record={record} />
        </div>
      </article>
    </div>
  );
}
