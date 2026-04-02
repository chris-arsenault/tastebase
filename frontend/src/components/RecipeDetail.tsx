import { useCallback, useEffect, useRef, useState } from "react";
import { deleteRecipe, fetchRecipe } from "../api";
import { RecipeImages, RecipeReviews, ReviewCapture } from "./RecipeMedia";
import type { RecipeFull, RecipeIngredient } from "../types";

function slugify(title: string): string {
  return title.toLowerCase().replace(/[^a-z0-9\s-]/g, "").replace(/\s+/g, "-").replace(/-+/g, "-").replace(/^-|-$/g, "");
}

function navigateToRecipe(slug: string) {
  window.history.pushState(null, "", `/recipes/${slug}`);
  window.dispatchEvent(new PopStateEvent("popstate"));
}

const sourceLabels: Record<string, string> = { claude: "Recipe by Claude", manual: "Manual", import: "Imported" };

const formatTimer = (s: number) => {
  const m = Math.floor(s / 60), r = s % 60;
  if (s < 60) return `${s}s`;
  return r === 0 ? `${m}m` : `${m}m ${r}s`;
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
  token: string;
  onClose: () => void;
  onDeleted: () => void;
};

function useRecipeFetch(recipeId: string) {
  const [recipe, setRecipe] = useState<RecipeFull | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [fetchKey, setFetchKey] = useState(0);

  useEffect(() => {
    let stale = false;
    fetchRecipe(recipeId)
      .then((data) => { if (!stale) { setRecipe(data); setLoading(false); } })
      .catch((e: unknown) => { if (!stale) { setError((e as Error).message); setLoading(false); } });
    return () => { stale = true; };
  }, [recipeId, fetchKey]);

  const reload = useCallback(() => setFetchKey((k) => k + 1), []);
  return { recipe, loading, error, reload };
}

function RecipeHero({ recipe }: Readonly<{ recipe: RecipeFull | null }>) {
  const heroUrl = recipe?.images?.[0]?.imageUrl;
  return (
    <div className="recipe-page-hero">
      {heroUrl ? (
        <img className="recipe-page-hero-img" src={heroUrl} alt={recipe?.title ?? "Recipe"} />
      ) : (
        <div className="recipe-page-hero-empty"><span>{"\uD83D\uDCD6"}</span></div>
      )}
    </div>
  );
}

function RecipeHeader({ recipe, servings, onServingsChange }: Readonly<{
  recipe: RecipeFull; servings: number; onServingsChange: (n: number) => void;
}>) {
  return (
    <header className="recipe-detail-header">
      <h2>{recipe.title}</h2>
      <div className="recipe-detail-meta">
        <span className="recipe-source-badge">{sourceLabels[recipe.source] ?? recipe.source}</span>
        <span className="recipe-servings-control">
          <button type="button" className="servings-btn" onClick={() => onServingsChange(Math.max(1, servings - 1))} disabled={servings <= 1}>{"\u2212"}</button>
          <span>{servings} serving{servings !== 1 ? "s" : ""}</span>
          <button type="button" className="servings-btn" onClick={() => onServingsChange(servings + 1)}>+</button>
        </span>
      </div>
      {recipe.description && <p className="recipe-detail-description">{recipe.description}</p>}
    </header>
  );
}

const formatAmount = (n: number): string => {
  if (Number.isInteger(n)) return n.toString();
  const frac = n % 1;
  const whole = Math.floor(n);
  const fractions: [number, string][] = [[0.25, "\u00BC"], [0.333, "\u2153"], [0.5, "\u00BD"], [0.666, "\u2154"], [0.75, "\u00BE"]];
  for (const [val, sym] of fractions) {
    if (Math.abs(frac - val) < 0.02) return whole > 0 ? `${whole}${sym}` : sym;
  }
  return n.toFixed(1).replace(/\.0$/, "");
};

function LinkedIngredientRow({ ing, scale, cache }: Readonly<{
  ing: RecipeIngredient; scale: number; cache: React.MutableRefObject<Map<string, RecipeFull>>;
}>) {
  const [showTooltip, setShowTooltip] = useState(false);
  const [preview, setPreview] = useState<RecipeFull | null>(() => cache.current.get(ing.linkedRecipeId!) ?? null);
  const [fetchStarted, setFetchStarted] = useState(false);
  const timerRef = useRef<number>(0);
  const linkedId = ing.linkedRecipeId!;

  const startHover = useCallback(() => {
    if (!fetchStarted && !cache.current.has(linkedId)) {
      setFetchStarted(true);
      fetchRecipe(linkedId).then((data) => {
        cache.current.set(linkedId, data);
        setPreview(data);
      }).catch(() => {});
    }
    timerRef.current = window.setTimeout(() => setShowTooltip(true), 200);
  }, [linkedId, cache, fetchStarted]);

  const endHover = useCallback(() => {
    clearTimeout(timerRef.current);
    setShowTooltip(false);
  }, []);

  useEffect(() => () => clearTimeout(timerRef.current), []);

  const handleClick = useCallback(() => {
    const cached = cache.current.get(linkedId);
    if (cached) {
      navigateToRecipe(slugify(cached.title));
      return;
    }
    fetchRecipe(linkedId).then((data) => {
      cache.current.set(linkedId, data);
      navigateToRecipe(slugify(data.title));
    }).catch(() => {});
  }, [linkedId, cache]);

  const thumbUrl = preview?.images?.[0]?.imageUrl;

  return (
    <li className="recipe-ing-linked-row" onMouseEnter={startHover} onMouseLeave={endHover}>
      <span className="recipe-ing-amount">{formatAmount(ing.amount * scale)} {ing.unit}</span>
      <button type="button" className="recipe-ing-linked" onClick={handleClick}>
        {ing.name}
        <span className="recipe-ing-linked-icon" aria-hidden="true">{"\u2197"}</span>
      </button>
      {showTooltip && (
        <div className="recipe-linked-tooltip">
          {!preview ? (
            <span className="recipe-linked-tooltip-loading">{"\u2026"}</span>
          ) : (
            <>
              {thumbUrl && <img className="recipe-linked-tooltip-thumb" src={thumbUrl} alt="" />}
              <div className="recipe-linked-tooltip-info">
                <span className="recipe-linked-tooltip-title">{preview.title}</span>
                {preview.description && <span className="recipe-linked-tooltip-desc">{preview.description}</span>}
                <span className="recipe-linked-tooltip-meta">
                  {preview.ingredients.length} ingredient{preview.ingredients.length !== 1 ? "s" : ""} {"\u00B7"} {preview.baseServings} serving{preview.baseServings !== 1 ? "s" : ""}
                </span>
              </div>
            </>
          )}
        </div>
      )}
    </li>
  );
}

function RecipeIngredients({ ingredients, scale, linkedRecipeCache }: Readonly<{
  ingredients: RecipeFull["ingredients"]; scale: number;
  linkedRecipeCache: React.MutableRefObject<Map<string, RecipeFull>>;
}>) {
  if (ingredients.length === 0) return null;
  return (
    <section className="recipe-ingredients-section">
      <h3>Ingredients</h3>
      <ul className="recipe-ingredients-list">
        {ingredients
          .slice().sort((a, b) => a.sortOrder - b.sortOrder)
          .map((ing) =>
            ing.linkedRecipeId ? (
              <LinkedIngredientRow key={ing.id} ing={ing} scale={scale} cache={linkedRecipeCache} />
            ) : (
              <li key={ing.id}>
                <span className="recipe-ing-amount">{formatAmount(ing.amount * scale)} {ing.unit}</span>
                <span className="recipe-ing-name">{ing.name}</span>
              </li>
            )
          )}
      </ul>
    </section>
  );
}

type IngredientInfo = { name: string; linkedRecipeId?: string | null };

function resolveTokens(
  content: string,
  ingredientMap: Map<string, IngredientInfo>,
  linkedRecipeCache: React.MutableRefObject<Map<string, RecipeFull>>,
): React.ReactNode[] {
  const result: React.ReactNode[] = [];
  let lastIndex = 0;
  const re = /\{(\w+)\}/g;
  let match = re.exec(content);
  while (match !== null) {
    if (match.index > lastIndex) result.push(content.slice(lastIndex, match.index));
    const info = ingredientMap.get(match[1]);
    if (info) {
      if (info.linkedRecipeId) {
        const linkedId = info.linkedRecipeId;
        result.push(
          <button
            key={match.index}
            type="button"
            className="recipe-ingredient-token recipe-ingredient-token-linked"
            onClick={() => {
              const cached = linkedRecipeCache.current.get(linkedId);
              if (cached) { navigateToRecipe(slugify(cached.title)); return; }
              fetchRecipe(linkedId).then((data) => {
                linkedRecipeCache.current.set(linkedId, data);
                navigateToRecipe(slugify(data.title));
              }).catch(() => {});
            }}
          >
            {info.name}
          </button>
        );
      } else {
        result.push(<strong key={match.index} className="recipe-ingredient-token">{info.name}</strong>);
      }
    } else {
      result.push(match[0]);
    }
    lastIndex = re.lastIndex;
    match = re.exec(content);
  }
  if (lastIndex < content.length) result.push(content.slice(lastIndex));
  return result;
}

function buildIngredientMap(ingredients: RecipeIngredient[]): Map<string, IngredientInfo> {
  const map = new Map<string, IngredientInfo>();
  for (const ing of ingredients) map.set(ing.widgetId, { name: ing.shortName || ing.name, linkedRecipeId: ing.linkedRecipeId });
  return map;
}

function RecipeSteps({ steps, ingredients, linkedRecipeCache }: Readonly<{
  steps: RecipeFull["steps"]; ingredients: RecipeFull["ingredients"];
  linkedRecipeCache: React.MutableRefObject<Map<string, RecipeFull>>;
}>) {
  if (steps.length === 0) return null;
  const ingredientMap = buildIngredientMap(ingredients);
  return (
    <section className="recipe-steps-section">
      <h3>Steps</h3>
      <ol className="recipe-steps-list">
        {steps.slice().sort((a, b) => a.sortOrder - b.sortOrder).map((step) => (
            <li key={step.id} className="recipe-step">
              <div className="recipe-step-header">
                <span className="recipe-step-title">{step.title}</span>
                {step.timerSeconds !== null && <span className="recipe-timer-badge">{formatTimer(step.timerSeconds)}</span>}
              </div>
              <p className="recipe-step-content">{resolveTokens(step.content, ingredientMap, linkedRecipeCache)}</p>
            </li>
          ))}
      </ol>
    </section>
  );
}

function RecipeBody({ recipe, token, onReviewSubmitted }: Readonly<{
  recipe: RecipeFull; token: string; onReviewSubmitted: () => void;
}>) {
  const [servings, setServings] = useState(recipe.baseServings);
  const scale = servings / recipe.baseServings;
  const linkedRecipeCache = useRef(new Map<string, RecipeFull>());
  return (
    <>
      <RecipeHeader recipe={recipe} servings={servings} onServingsChange={setServings} />
      <RecipeIngredients ingredients={recipe.ingredients} scale={scale} linkedRecipeCache={linkedRecipeCache} />
      <RecipeSteps steps={recipe.steps} ingredients={recipe.ingredients} linkedRecipeCache={linkedRecipeCache} />
      {recipe.notes && (
        <section className="recipe-notes-section">
          <h3>Notes</h3>
          {renderMarkdown(recipe.notes)}
        </section>
      )}
      <RecipeImages images={recipe.images} token={token} onDeleted={onReviewSubmitted} />
      <RecipeReviews reviews={recipe.reviews} token={token} onDeleted={onReviewSubmitted} />
      {token && <ReviewCapture recipeId={recipe.id} token={token} onSubmitted={onReviewSubmitted} />}
    </>
  );
}

export function RecipeDetail({ recipeId, token, onClose, onDeleted }: Readonly<RecipeDetailProps>) {
  const { recipe, loading, error, reload } = useRecipeFetch(recipeId);
  const [deleteState, setDeleteState] = useState<"idle" | "confirm" | "deleting" | "error">("idle");
  const [deleteError, setDeleteError] = useState("");

  const handleDelete = useCallback(() => {
    if (!token) return;
    if (deleteState !== "confirm") { setDeleteState("confirm"); return; }
    setDeleteState("deleting");
    deleteRecipe(recipeId, token)
      .then(onDeleted)
      .catch((e: unknown) => { setDeleteError((e as Error).message); setDeleteState("error"); });
  }, [recipeId, token, onDeleted, deleteState]);

  const cancelDelete = useCallback(() => setDeleteState("idle"), []);

  return (
    <div className="recipe-page">
      <div className="recipe-page-back">
        <button type="button" className="recipe-back-btn" onClick={onClose}>{"\u2190"} Back to recipes</button>
        {token && deleteState === "idle" && (
          <button type="button" className="btn-danger recipe-delete-btn" onClick={handleDelete}>Delete</button>
        )}
        {deleteState === "confirm" && (
          <span className="recipe-delete-confirm">
            <span>Delete this recipe?</span>
            <button type="button" className="btn-danger" onClick={handleDelete}>Yes, delete</button>
            <button type="button" className="btn-cancel" onClick={cancelDelete}>Cancel</button>
          </span>
        )}
        {deleteState === "deleting" && <span className="recipe-delete-confirm">Deleting...</span>}
        {deleteState === "error" && <span className="error-banner">{deleteError}</span>}
      </div>
      <RecipeHero recipe={recipe} />
      <div className="recipe-page-content">
        {loading && <div className="loading">Loading recipe...</div>}
        {error && <div className="error-banner">{error}</div>}
        {recipe && <RecipeBody recipe={recipe} token={token} onReviewSubmitted={reload} />}
      </div>
    </div>
  );
}
