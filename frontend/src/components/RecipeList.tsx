import type { Recipe } from "../types";
import { Tooltip } from "./Tooltip";

const sourceLabels: Record<string, string> = {
  claude: "Recipe by Claude",
  manual: "Manual",
  import: "Imported",
};

const formatDate = (value: string) => {
  if (!value) return "";
  const d = new Date(value);
  if (Number.isNaN(d.getTime())) return value;
  return `${d.getMonth() + 1}/${d.getDate()}/${d.getFullYear()}`;
};

function RecipeScore({ score }: Readonly<{ score: number | null }>) {
  if (score == null) return null;
  return (
    <Tooltip label="Latest review score out of 10">
      <span className="recipe-card-score tabular">
        {score}
        <span className="recipe-card-score-max">/10</span>
      </span>
    </Tooltip>
  );
}

type RecipeListProps = {
  recipes: Recipe[];
  loading: boolean;
  error: string;
  onSelect: (recipe: Recipe) => void;
  filtered: boolean;
};

export function RecipeList({
  recipes,
  loading,
  error,
  onSelect,
  filtered,
}: Readonly<RecipeListProps>) {
  if (loading) {
    return <div className="loading">Loading recipes...</div>;
  }

  if (error) {
    return <div className="error-banner">{error}</div>;
  }

  if (recipes.length === 0) {
    return (
      <div className="empty-state">
        <span className="empty-icon">{"📖"}</span>
        <p>{filtered ? "No recipes match your search." : "No recipes yet."}</p>
      </div>
    );
  }

  return (
    <div className="recipe-grid">
      {recipes.map((recipe) => (
        <button
          key={recipe.id}
          type="button"
          className="recipe-card"
          onClick={() => onSelect(recipe)}
        >
          <div className="recipe-card-image">
            {recipe.thumbnailUrl ? (
              <img
                src={recipe.thumbnailUrl}
                alt={recipe.title}
                loading="lazy"
              />
            ) : (
              <div className="recipe-card-image-empty">{"📖"}</div>
            )}
          </div>
          <div className="recipe-card-content">
            <span className="recipe-card-source">
              {sourceLabels[recipe.source] ?? recipe.source}
            </span>
            <h3>{recipe.title}</h3>
            {recipe.description && (
              <p className="recipe-card-description">{recipe.description}</p>
            )}
            <div className="recipe-card-meta">
              <RecipeScore score={recipe.latestScore ?? null} />
              <span className="recipe-card-date">
                {formatDate(recipe.createdAt)}
              </span>
            </div>
          </div>
        </button>
      ))}
    </div>
  );
}
