import type { RecipeVariationGroup, VariationSelection } from "../types";

export function resolveVariationSelection(
  groups: RecipeVariationGroup[],
  selection: VariationSelection,
): VariationSelection {
  return Object.fromEntries(
    groups.map((group) => [
      group.id,
      group.options.some((option) => option.id === selection[group.id])
        ? selection[group.id]
        : group.default,
    ]),
  );
}

export function matchesVariation(
  condition: VariationSelection | undefined,
  selection: VariationSelection,
): boolean {
  return Object.entries(condition ?? {}).every(
    ([group, option]) => selection[group] === option,
  );
}

export function selectVariationContent<T extends { when?: VariationSelection }>(
  rows: T[],
  selection: VariationSelection,
): T[] {
  return rows.filter((row) => matchesVariation(row.when, selection));
}
