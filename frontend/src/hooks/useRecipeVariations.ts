import { useCallback, useState } from "react";
import type { RecipeFull, VariationSelection } from "../types";
import {
  resolveVariationSelection,
  selectVariationContent,
} from "../utils/recipeVariations";

export function useRecipeVariations(recipe: RecipeFull) {
  const [choices, setChoices] = useState<VariationSelection>({});
  const groups = recipe.variationGroups ?? [];
  const selection = resolveVariationSelection(groups, choices);
  const ingredients = selectVariationContent(recipe.ingredients, selection);
  const steps = selectVariationContent(recipe.steps, selection);
  const changeVariation = useCallback((group: string, option: string) => {
    setChoices((current) => ({ ...current, [group]: option }));
  }, []);
  return { groups, selection, ingredients, steps, changeVariation };
}
