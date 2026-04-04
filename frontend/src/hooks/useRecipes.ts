import { useEffect, useState } from "react";
import { fetchRecipes } from "../api";
import type { Recipe } from "../types";

export function useRecipes() {
  const [recipes, setRecipes] = useState<Recipe[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  useEffect(() => {
    let stale = false;
    fetchRecipes()
      .then((data) => {
        if (!stale) setRecipes(data);
      })
      .catch((e: unknown) => {
        if (!stale) setError((e as Error).message);
      })
      .finally(() => {
        if (!stale) setLoading(false);
      });
    return () => {
      stale = true;
    };
  }, []);

  const reload = () => {
    setLoading(true);
    setError("");
    fetchRecipes()
      .then(setRecipes)
      .catch((e: unknown) => setError((e as Error).message))
      .finally(() => setLoading(false));
  };

  return { recipes, loading, error, reload };
}
