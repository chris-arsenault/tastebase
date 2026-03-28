import { useCallback, useEffect, useState } from "react";
import { fetchRecipe } from "../api";
import type { RecipeFull } from "../types";

const sourceLabels: Record<string, string> = {
  claude: "AI Generated",
  manual: "Manual",
  import: "Imported"
};

const formatTimer = (seconds: number) => {
  if (seconds < 60) return `${seconds}s`;
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  if (secs === 0) return `${mins}m`;
  return `${mins}m ${secs}s`;
};

type RecipeDetailProps = {
  recipeId: string;
  onClose: () => void;
};

/** Fetches a recipe by ID. Relies on remounting (key={recipeId}) for reset. */
function useRecipeFetch(recipeId: string) {
  const [recipe, setRecipe] = useState<RecipeFull | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  useEffect(() => {
    let stale = false;
    fetchRecipe(recipeId)
      .then((data) => { if (!stale) { setRecipe(data); setLoading(false); } })
      .catch((e: unknown) => { if (!stale) { setError((e as Error).message); setLoading(false); } });
    return () => { stale = true; };
  }, [recipeId]);

  return { recipe, loading, error };
}

function RecipeHero({ recipe, onClose }: Readonly<{ recipe: RecipeFull | null; onClose: () => void }>) {
  return (
    <div className="view-hero-section recipe-detail-hero">
      {recipe?.cover_image_url ? (
        <img className="view-hero-img" src={recipe.cover_image_url} alt={recipe.title} />
      ) : (
        <div className="view-hero-empty">
          <span>{"\uD83D\uDCD6"}</span>
        </div>
      )}
      <button type="button" className="view-close" onClick={onClose}>{"\u00D7"}</button>
    </div>
  );
}

function RecipeHeader({ recipe }: Readonly<{ recipe: RecipeFull }>) {
  return (
    <header className="recipe-detail-header">
      <h2>{recipe.title}</h2>
      <div className="recipe-detail-meta">
        <span className="recipe-source-badge">{sourceLabels[recipe.source] ?? recipe.source}</span>
        <span>{recipe.base_servings} serving{recipe.base_servings !== 1 ? "s" : ""}</span>
      </div>
      {recipe.description && <p className="recipe-detail-description">{recipe.description}</p>}
    </header>
  );
}

function RecipeIngredients({ ingredients }: Readonly<{ ingredients: RecipeFull["ingredients"] }>) {
  if (ingredients.length === 0) return null;
  return (
    <section className="recipe-ingredients-section">
      <h3>Ingredients</h3>
      <ul className="recipe-ingredients-list">
        {ingredients
          .sort((a, b) => a.sort_order - b.sort_order)
          .map((ing) => (
            <li key={ing.id}>
              <span className="recipe-ing-amount">{ing.amount} {ing.unit}</span>
              <span className="recipe-ing-name">{ing.name}</span>
            </li>
          ))}
      </ul>
    </section>
  );
}

function RecipeSteps({ steps }: Readonly<{ steps: RecipeFull["steps"] }>) {
  if (steps.length === 0) return null;
  return (
    <section className="recipe-steps-section">
      <h3>Steps</h3>
      <ol className="recipe-steps-list">
        {steps
          .sort((a, b) => a.sort_order - b.sort_order)
          .map((step) => (
            <li key={step.id} className="recipe-step">
              <div className="recipe-step-header">
                <span className="recipe-step-title">{step.title}</span>
                {step.timer_seconds !== null && (
                  <span className="recipe-timer-badge">{formatTimer(step.timer_seconds)}</span>
                )}
              </div>
              <p className="recipe-step-content">{step.content}</p>
            </li>
          ))}
      </ol>
    </section>
  );
}

function RecipeBody({ recipe }: Readonly<{ recipe: RecipeFull }>) {
  return (
    <>
      <RecipeHeader recipe={recipe} />
      <RecipeIngredients ingredients={recipe.ingredients} />
      <RecipeSteps steps={recipe.steps} />
      {recipe.notes && (
        <section className="recipe-notes-section">
          <h3>Notes</h3>
          <p>{recipe.notes}</p>
        </section>
      )}
    </>
  );
}

export function RecipeDetail({ recipeId, onClose }: Readonly<RecipeDetailProps>) {
  const { recipe, loading, error } = useRecipeFetch(recipeId);

  const handleOverlayClick = useCallback((e: React.MouseEvent<HTMLDivElement>) => {
    if (e.target === e.currentTarget) onClose();
  }, [onClose]);

  const handleOverlayKeyDown = useCallback((e: React.KeyboardEvent<HTMLDivElement>) => {
    if (e.key === "Escape") onClose();
  }, [onClose]);

  return (
    <div className="view-overlay" role="presentation" onClick={handleOverlayClick} onKeyDown={handleOverlayKeyDown}>
      <article className="view-modal recipe-detail-modal" role="dialog" aria-modal="true">
        <RecipeHero recipe={recipe} onClose={onClose} />
        <div className="view-content">
          {loading && <div className="loading">Loading recipe...</div>}
          {error && <div className="error-banner">{error}</div>}
          {recipe && <RecipeBody recipe={recipe} />}
        </div>
      </article>
    </div>
  );
}
