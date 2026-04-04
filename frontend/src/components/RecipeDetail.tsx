import { useCallback, useRef, useState } from "react";
import { deleteRecipe } from "../api";
import { RecipeImages, RecipeReviews, ReviewCapture } from "./RecipeMedia";
import type { RecipeFull } from "../types";
import {
  sourceLabels,
  formatTimer,
  formatAmount,
  renderMarkdown,
  useRecipeFetch,
  buildIngredientMap,
  resolveTokens,
  LinkedIngredientRow,
} from "./RecipeDetailHelpers";

type RecipeDetailProps = {
  recipeId: string;
  token: string;
  onClose: () => void;
  onDeleted: () => void;
};

function RecipeHero({ recipe }: Readonly<{ recipe: RecipeFull | null }>) {
  const heroUrl = recipe?.images?.[0]?.imageUrl;
  return (
    <div className="recipe-page-hero">
      {heroUrl ? (
        <img
          className="recipe-page-hero-img"
          src={heroUrl}
          alt={recipe?.title ?? "Recipe"}
        />
      ) : (
        <div className="recipe-page-hero-empty">
          <span>{"\uD83D\uDCD6"}</span>
        </div>
      )}
    </div>
  );
}

function RecipeHeader({
  recipe,
  servings,
  onServingsChange,
}: Readonly<{
  recipe: RecipeFull;
  servings: number;
  onServingsChange: (n: number) => void;
}>) {
  return (
    <header className="recipe-detail-header">
      <h2>{recipe.title}</h2>
      <div className="recipe-detail-meta">
        <span className="recipe-source-badge">
          {sourceLabels[recipe.source] ?? recipe.source}
        </span>
        <span className="recipe-servings-control">
          <button
            type="button"
            className="servings-btn"
            onClick={() => onServingsChange(Math.max(1, servings - 1))}
            disabled={servings <= 1}
          >
            {"\u2212"}
          </button>
          <span>
            {servings} serving{servings !== 1 ? "s" : ""}
          </span>
          <button
            type="button"
            className="servings-btn"
            onClick={() => onServingsChange(servings + 1)}
          >
            +
          </button>
        </span>
      </div>
      {recipe.description && (
        <p className="recipe-detail-description">{recipe.description}</p>
      )}
    </header>
  );
}

function RecipeIngredients({
  ingredients,
  scale,
  linkedRecipeCache,
}: Readonly<{
  ingredients: RecipeFull["ingredients"];
  scale: number;
  linkedRecipeCache: React.MutableRefObject<Map<string, RecipeFull>>;
}>) {
  if (ingredients.length === 0) return null;
  return (
    <section className="recipe-ingredients-section">
      <h3>Ingredients</h3>
      <ul className="recipe-ingredients-list">
        {ingredients
          .slice()
          .sort((a, b) => a.sortOrder - b.sortOrder)
          .map((ing) =>
            ing.linkedRecipeId ? (
              <LinkedIngredientRow
                key={ing.id}
                ing={ing}
                scale={scale}
                cache={linkedRecipeCache}
              />
            ) : (
              <li key={ing.id}>
                <span className="recipe-ing-amount">
                  {formatAmount(ing.amount * scale)} {ing.unit}
                </span>
                <span className="recipe-ing-name">{ing.name}</span>
              </li>
            ),
          )}
      </ul>
    </section>
  );
}

function RecipeSteps({
  steps,
  ingredients,
  linkedRecipeCache,
}: Readonly<{
  steps: RecipeFull["steps"];
  ingredients: RecipeFull["ingredients"];
  linkedRecipeCache: React.MutableRefObject<Map<string, RecipeFull>>;
}>) {
  if (steps.length === 0) return null;
  const ingredientMap = buildIngredientMap(ingredients);
  return (
    <section className="recipe-steps-section">
      <h3>Steps</h3>
      <ol className="recipe-steps-list">
        {steps
          .slice()
          .sort((a, b) => a.sortOrder - b.sortOrder)
          .map((step) => (
            <li key={step.id} className="recipe-step">
              <div className="recipe-step-header">
                <span className="recipe-step-title">{step.title}</span>
                {step.timerSeconds !== null && (
                  <span className="recipe-timer-badge">
                    {formatTimer(step.timerSeconds)}
                  </span>
                )}
              </div>
              <p className="recipe-step-content">
                {resolveTokens(step.content, ingredientMap, linkedRecipeCache)}
              </p>
            </li>
          ))}
      </ol>
    </section>
  );
}

function RecipeBody({
  recipe,
  token,
  onReviewSubmitted,
}: Readonly<{
  recipe: RecipeFull;
  token: string;
  onReviewSubmitted: () => void;
}>) {
  const [servings, setServings] = useState(recipe.baseServings);
  const scale = servings / recipe.baseServings;
  const linkedRecipeCache = useRef(new Map<string, RecipeFull>());
  return (
    <>
      <RecipeHeader
        recipe={recipe}
        servings={servings}
        onServingsChange={setServings}
      />
      <RecipeIngredients
        ingredients={recipe.ingredients}
        scale={scale}
        linkedRecipeCache={linkedRecipeCache}
      />
      <RecipeSteps
        steps={recipe.steps}
        ingredients={recipe.ingredients}
        linkedRecipeCache={linkedRecipeCache}
      />
      {recipe.notes && (
        <section className="recipe-notes-section">
          <h3>Notes</h3>
          {renderMarkdown(recipe.notes)}
        </section>
      )}
      <RecipeImages
        images={recipe.images}
        token={token}
        onDeleted={onReviewSubmitted}
      />
      <RecipeReviews
        reviews={recipe.reviews}
        token={token}
        onDeleted={onReviewSubmitted}
      />
      {token && (
        <ReviewCapture
          recipeId={recipe.id}
          token={token}
          onSubmitted={onReviewSubmitted}
        />
      )}
    </>
  );
}

function DeleteControls({
  deleteState,
  token,
  onDelete,
  onCancel,
  deleteError,
}: Readonly<{
  deleteState: string;
  token: string;
  onDelete: () => void;
  onCancel: () => void;
  deleteError: string;
}>) {
  return (
    <>
      {token && deleteState === "idle" && (
        <button
          type="button"
          className="btn-danger recipe-delete-btn"
          onClick={onDelete}
        >
          Delete
        </button>
      )}
      {deleteState === "confirm" && (
        <span className="recipe-delete-confirm">
          <span>Delete this recipe?</span>
          <button type="button" className="btn-danger" onClick={onDelete}>
            Yes, delete
          </button>
          <button type="button" className="btn-cancel" onClick={onCancel}>
            Cancel
          </button>
        </span>
      )}
      {deleteState === "deleting" && (
        <span className="recipe-delete-confirm">Deleting...</span>
      )}
      {deleteState === "error" && (
        <span className="error-banner">{deleteError}</span>
      )}
    </>
  );
}

export function RecipeDetail({
  recipeId,
  token,
  onClose,
  onDeleted,
}: Readonly<RecipeDetailProps>) {
  const { recipe, loading, error, reload } = useRecipeFetch(recipeId);
  const [deleteState, setDeleteState] = useState<
    "idle" | "confirm" | "deleting" | "error"
  >("idle");
  const [deleteError, setDeleteError] = useState("");

  const handleDelete = useCallback(() => {
    if (!token) return;
    if (deleteState !== "confirm") {
      setDeleteState("confirm");
      return;
    }
    setDeleteState("deleting");
    deleteRecipe(recipeId, token)
      .then(onDeleted)
      .catch((e: unknown) => {
        setDeleteError((e as Error).message);
        setDeleteState("error");
      });
  }, [recipeId, token, onDeleted, deleteState]);

  const cancelDelete = useCallback(() => setDeleteState("idle"), []);

  return (
    <div className="recipe-page">
      <div className="recipe-page-back">
        <button type="button" className="recipe-back-btn" onClick={onClose}>
          {"\u2190"} Back to recipes
        </button>
        <DeleteControls
          deleteState={deleteState}
          token={token}
          onDelete={handleDelete}
          onCancel={cancelDelete}
          deleteError={deleteError}
        />
      </div>
      <RecipeHero recipe={recipe} />
      <div className="recipe-page-content">
        {loading && <div className="loading">Loading recipe...</div>}
        {error && <div className="error-banner">{error}</div>}
        {recipe && (
          <RecipeBody
            recipe={recipe}
            token={token}
            onReviewSubmitted={reload}
          />
        )}
      </div>
    </div>
  );
}
