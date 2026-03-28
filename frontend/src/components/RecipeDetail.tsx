import { useEffect, useState } from "react";
import { fetchRecipe } from "../api";
import type { RecipeFull, RecipeIngredient } from "../types";

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

/** Simple markdown: **bold**, \n\n → paragraphs, \n → <br /> */
function renderMarkdown(text: string): React.ReactNode[] {
  const paragraphs = text.split(/\n\n/);
  return paragraphs.map((para, pi) => {
    const lines = para.split(/\n/);
    const children: React.ReactNode[] = [];
    lines.forEach((line, li) => {
      if (li > 0) children.push(<br key={`br-${pi}-${li}`} />);
      // Split on **bold** markers
      const parts = line.split(/(\*\*[^*]+\*\*)/g);
      parts.forEach((part, partIdx) => {
        const boldMatch = /^\*\*(.+)\*\*$/.exec(part);
        if (boldMatch) {
          children.push(<strong key={`b-${pi}-${li}-${partIdx}`}>{boldMatch[1]}</strong>);
        } else {
          children.push(part);
        }
      });
    });
    return <p key={`p-${pi}`} className="recipe-notes-paragraph">{children}</p>;
  });
}

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

function RecipeHero({ recipe }: Readonly<{ recipe: RecipeFull | null }>) {
  return (
    <div className="recipe-page-hero">
      {recipe?.coverImageUrl ? (
        <img className="recipe-page-hero-img" src={recipe.coverImageUrl} alt={recipe.title} />
      ) : (
        <div className="recipe-page-hero-empty">
          <span>{"\uD83D\uDCD6"}</span>
        </div>
      )}
    </div>
  );
}

function RecipeHeader({ recipe }: Readonly<{ recipe: RecipeFull }>) {
  return (
    <header className="recipe-detail-header">
      <h2>{recipe.title}</h2>
      <div className="recipe-detail-meta">
        <span className="recipe-source-badge">{sourceLabels[recipe.source] ?? recipe.source}</span>
        <span>{recipe.baseServings} serving{recipe.baseServings !== 1 ? "s" : ""}</span>
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
          .toSorted((a, b) => a.sortOrder - b.sortOrder)
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

/** Resolve {widgetId} tokens in step content to ingredient names */
function resolveTokens(
  content: string,
  ingredientMap: Map<string, string>,
): React.ReactNode[] {
  const result: React.ReactNode[] = [];
  let lastIndex = 0;
  const re = /\{(\w+)\}/g;
  let match = re.exec(content);
  while (match !== null) {
    if (match.index > lastIndex) {
      result.push(content.slice(lastIndex, match.index));
    }
    const widgetId = match[1];
    const name = ingredientMap.get(widgetId);
    if (name) {
      result.push(
        <strong key={match.index} className="recipe-ingredient-token">{name}</strong>,
      );
    } else {
      result.push(match[0]);
    }
    lastIndex = re.lastIndex;
    match = re.exec(content);
  }
  if (lastIndex < content.length) {
    result.push(content.slice(lastIndex));
  }
  return result;
}

function buildIngredientMap(ingredients: RecipeIngredient[]): Map<string, string> {
  const map = new Map<string, string>();
  for (const ing of ingredients) {
    map.set(ing.widgetId, ing.name);
  }
  return map;
}

function RecipeSteps({ steps, ingredients }: Readonly<{
  steps: RecipeFull["steps"];
  ingredients: RecipeFull["ingredients"];
}>) {
  if (steps.length === 0) return null;
  const ingredientMap = buildIngredientMap(ingredients);
  return (
    <section className="recipe-steps-section">
      <h3>Steps</h3>
      <ol className="recipe-steps-list">
        {steps
          .toSorted((a, b) => a.sortOrder - b.sortOrder)
          .map((step) => (
            <li key={step.id} className="recipe-step">
              <div className="recipe-step-header">
                <span className="recipe-step-title">{step.title}</span>
                {step.timerSeconds !== null && (
                  <span className="recipe-timer-badge">{formatTimer(step.timerSeconds)}</span>
                )}
              </div>
              <p className="recipe-step-content">{resolveTokens(step.content, ingredientMap)}</p>
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
      <RecipeSteps steps={recipe.steps} ingredients={recipe.ingredients} />
      {recipe.notes && (
        <section className="recipe-notes-section">
          <h3>Notes</h3>
          {renderMarkdown(recipe.notes)}
        </section>
      )}
    </>
  );
}

export function RecipeDetail({ recipeId, onClose }: Readonly<RecipeDetailProps>) {
  const { recipe, loading, error } = useRecipeFetch(recipeId);

  return (
    <div className="recipe-page">
      <div className="recipe-page-back">
        <button type="button" className="recipe-back-btn" onClick={onClose}>
          {"\u2190"} Back to recipes
        </button>
      </div>
      <RecipeHero recipe={recipe} />
      <div className="recipe-page-content">
        {loading && <div className="loading">Loading recipe...</div>}
        {error && <div className="error-banner">{error}</div>}
        {recipe && <RecipeBody recipe={recipe} />}
      </div>
    </div>
  );
}
