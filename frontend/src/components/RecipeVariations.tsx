import { useId } from "react";
import type { RecipeVariationGroup, VariationSelection } from "../types";

export function RecipeVariations({
  groups,
  selection,
  onChange,
}: Readonly<{
  groups: RecipeVariationGroup[];
  selection: VariationSelection;
  onChange: (group: string, option: string) => void;
}>) {
  const id = useId();
  if (groups.length === 0) return null;
  return (
    <fieldset className="recipe-variations">
      <legend>Variations</legend>
      <div className="recipe-variation-selectors">
        {groups.map((group) => (
          <label key={group.id} htmlFor={`${id}-${group.id}`}>
            <span>{group.label}</span>
            <select
              id={`${id}-${group.id}`}
              value={selection[group.id]}
              onChange={(event) => onChange(group.id, event.target.value)}
            >
              {group.options.map((option) => (
                <option key={option.id} value={option.id}>
                  {option.label}
                </option>
              ))}
            </select>
          </label>
        ))}
      </div>
    </fieldset>
  );
}
