import { useCallback, useMemo, useState } from "react";
import type { Recipe } from "../types";

export type RecipeSort = "createdAt" | "title" | "score";
export type SortDirection = "asc" | "desc";

const defaultDirection = (sort: RecipeSort): SortDirection =>
  sort === "title" ? "asc" : "desc";

const collator = new Intl.Collator(undefined, {
  numeric: true,
  sensitivity: "base",
});

function compareRecipes(left: Recipe, right: Recipe, sort: RecipeSort) {
  if (sort === "title") return collator.compare(left.title, right.title);
  if (sort === "score") {
    return (left.latestScore ?? -1) - (right.latestScore ?? -1);
  }
  return collator.compare(left.createdAt, right.createdAt);
}

export function useRecipeDiscovery(recipes: Recipe[]) {
  const [search, setSearch] = useState("");
  const [sort, setSortState] = useState<RecipeSort>("createdAt");
  const [direction, setDirection] = useState<SortDirection>("desc");

  const visibleRecipes = useMemo(() => {
    const query = search.trim().toLowerCase();
    const sign = direction === "asc" ? 1 : -1;
    return recipes
      .filter(
        (recipe) =>
          !query ||
          `${recipe.title} ${recipe.description ?? ""}`
            .toLowerCase()
            .includes(query),
      )
      .sort((left, right) => sign * compareRecipes(left, right, sort));
  }, [direction, recipes, search, sort]);

  const setSort = useCallback((next: RecipeSort) => {
    setSortState(next);
    setDirection(defaultDirection(next));
  }, []);
  const toggleDirection = useCallback(
    () => setDirection((current) => (current === "asc" ? "desc" : "asc")),
    [],
  );

  return {
    search,
    setSearch,
    sort,
    setSort,
    direction,
    toggleDirection,
    visibleRecipes,
  };
}
