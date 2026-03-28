import type { Recipe } from "../types";

const sourceLabels: Record<string, string> = {
  claude: "AI Generated",
  manual: "Manual",
  import: "Imported"
};

const formatDate = (value: string) => {
  if (!value) return "";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleDateString();
};

type RecipeListProps = {
  recipes: Recipe[];
  loading: boolean;
  error: string;
  onSelect: (recipe: Recipe) => void;
};

export function RecipeList({ recipes, loading, error, onSelect }: Readonly<RecipeListProps>) {
  if (loading) {
    return <div className="loading">Loading recipes...</div>;
  }

  if (error) {
    return <div className="error-banner">{error}</div>;
  }

  if (recipes.length === 0) {
    return (
      <div className="empty-state">
        <span className="empty-icon">{"\uD83D\uDCD6"}</span>
        <p>No recipes yet.</p>
      </div>
    );
  }

  return (
    <div className="recipe-grid">
      {recipes.map((recipe) => (
        <article
          key={recipe.id}
          className="recipe-card"
          role="button"
          tabIndex={0}
          onClick={() => onSelect(recipe)}
          onKeyDown={(e) => { if (e.key === "Enter") onSelect(recipe); }}
        >
          <div className="recipe-card-image">
            {recipe.cover_image_url ? (
              <img src={recipe.cover_image_url} alt={recipe.title} loading="lazy" />
            ) : (
              <div className="recipe-card-image-empty">{"\uD83D\uDCD6"}</div>
            )}
            <span className="recipe-source-badge">
              {sourceLabels[recipe.source] ?? recipe.source}
            </span>
          </div>
          <div className="recipe-card-content">
            <h3>{recipe.title}</h3>
            {recipe.description && (
              <p className="recipe-card-description">{recipe.description}</p>
            )}
            <div className="recipe-card-meta">
              <span className="recipe-card-servings">{recipe.base_servings} serving{recipe.base_servings !== 1 ? "s" : ""}</span>
              <span className="recipe-card-date">{formatDate(recipe.created_at)}</span>
            </div>
          </div>
        </article>
      ))}
    </div>
  );
}
