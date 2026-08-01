import { useCallback, useEffect, useMemo, useState } from "react";
import type { AppSection, Recipe } from "../types";
import { slugify } from "../utils/recipeText";

type RouteState =
  | { section: "tastings"; recipeSlug: null; reviewId: null }
  | { section: "books"; recipeSlug: null; reviewId: null }
  | { section: "recipes"; recipeSlug: null; reviewId: null }
  | { section: "recipes"; recipeSlug: string; reviewId: string | null };

function trimTrailingSlashes(path: string): string {
  let end = path.length;
  while (end > 0 && path[end - 1] === "/") end -= 1;
  return path.slice(0, end);
}

function parsePath(): RouteState {
  const path = window.location.pathname;
  if (path === "/books" || path.startsWith("/books/")) {
    return { section: "books", recipeSlug: null, reviewId: null };
  }
  if (path.startsWith("/recipes/")) {
    const recipePath = trimTrailingSlashes(path.slice("/recipes/".length));
    if (!recipePath) {
      return { section: "recipes", recipeSlug: null, reviewId: null };
    }
    const [slug, segment, reviewId] = recipePath.split("/");
    return {
      section: "recipes",
      recipeSlug: slug,
      reviewId: segment === "reviews" && reviewId ? reviewId : null,
    };
  }
  if (path === "/recipes") {
    return { section: "recipes", recipeSlug: null, reviewId: null };
  }
  return { section: "tastings", recipeSlug: null, reviewId: null };
}

function navigate(path: string) {
  window.history.pushState(null, "", path);
  window.dispatchEvent(new PopStateEvent("popstate"));
}

const sectionPaths: Record<AppSection, string> = {
  books: "/books",
  recipes: "/recipes",
  tastings: "/",
};

export function useAppRouter(recipes: Recipe[]) {
  const [route, setRoute] = useState<RouteState>(parsePath);

  useEffect(() => {
    const onNav = () => setRoute(parsePath());
    window.addEventListener("popstate", onNav);
    return () => window.removeEventListener("popstate", onNav);
  }, []);

  const setSection = useCallback((section: AppSection) => {
    navigate(sectionPaths[section]);
  }, []);
  const handleSelectRecipe = useCallback((recipe: Recipe) => {
    navigate(`/recipes/${slugify(recipe.title)}`);
  }, []);
  const handleBackToRecipes = useCallback(() => navigate("/recipes"), []);
  const selectedRecipe = useMemo(() => {
    if (!route.recipeSlug) return null;
    return (
      recipes.find((recipe) => slugify(recipe.title) === route.recipeSlug) ??
      null
    );
  }, [route.recipeSlug, recipes]);

  return {
    section: route.section,
    selectedRecipe,
    selectedReviewId: selectedRecipe ? route.reviewId : null,
    setSection,
    handleSelectRecipe,
    handleBackToRecipes,
  };
}
