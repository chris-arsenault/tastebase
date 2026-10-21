import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import type { RecipeVariationGroup, VariationSelection } from "../types";
import { RecipeVariations } from "../components/RecipeVariations";
import {
  matchesVariation,
  resolveVariationSelection,
  selectVariationContent,
} from "./recipeVariations";

const groups: RecipeVariationGroup[] = [
  {
    id: "style",
    label: "Style",
    default: "greek",
    options: [
      { id: "greek", label: "Greek" },
      { id: "skyr", label: "Skyr" },
    ],
  },
  {
    id: "flavor",
    label: "Flavor",
    default: "plain",
    options: [
      { id: "plain", label: "Plain" },
      { id: "peach", label: "Peach" },
      { id: "cherry", label: "Cherry" },
    ],
  },
];

describe("independent recipe variations", () => {
  it("uses defaults and changes one group without resetting another", () => {
    const initial = resolveVariationSelection(groups, {});
    expect(initial).toEqual({ style: "greek", flavor: "plain" });
    const peach = resolveVariationSelection(groups, {
      ...initial,
      flavor: "peach",
    });
    const skyr = resolveVariationSelection(groups, { ...peach, style: "skyr" });
    expect(skyr).toEqual({ style: "skyr", flavor: "peach" });
    expect(
      resolveVariationSelection(groups, { ...skyr, style: "removed" }),
    ).toEqual({ style: "greek", flavor: "peach" });
  });

  it.each(["greek", "skyr"])(
    "composes %s with every flavor in source order",
    (style) => {
      const ingredients: { id: string; when?: VariationSelection }[] = [
        { id: "milk" },
        { id: "greekCulture", when: { style: "greek" } },
        { id: "skyrCulture", when: { style: "skyr" } },
        { id: "peaches", when: { flavor: "peach" } },
        { id: "cherries", when: { flavor: "cherry" } },
      ];
      const steps: { id: string; when?: VariationSelection }[] = [
        { id: "heat" },
        { id: "greekIncubation", when: { style: "greek" } },
        { id: "skyrIncubation", when: { style: "skyr" } },
        { id: "strain" },
        { id: "peachPrep", when: { flavor: "peach" } },
        { id: "cherryPrep", when: { flavor: "cherry" } },
        { id: "chill" },
      ];
      for (const flavor of ["plain", "peach", "cherry"]) {
        const selection = { style, flavor };
        expect(
          selectVariationContent(ingredients, selection).map((row) => row.id),
        ).toEqual(["milk", `${style}Culture`, ...fruitIds(flavor)]);
        expect(
          selectVariationContent(steps, selection).map((row) => row.id),
        ).toEqual([
          "heat",
          `${style}Incubation`,
          "strain",
          ...flavorSteps(flavor),
          "chill",
        ]);
      }
    },
  );
});

describe("recipe variation conditions and controls", () => {
  it("supports conditions requiring several selections together", () => {
    const condition = { style: "skyr", flavor: "peach" };
    expect(
      matchesVariation(condition, { style: "skyr", flavor: "peach" }),
    ).toBe(true);
    expect(
      matchesVariation(condition, { style: "greek", flavor: "peach" }),
    ).toBe(false);
    expect(
      matchesVariation(condition, { style: "skyr", flavor: "plain" }),
    ).toBe(false);
  });

  it("keeps legacy content and hides selectors for recipes without variations", () => {
    const rows = [{ id: "milk" }, { id: "heat", when: {} }];
    expect(selectVariationContent(rows, {})).toEqual(rows);
    expect(renderSelectors([])).toBe("");
  });

  it("renders separately labeled dropdowns with the selected values", () => {
    const html = renderSelectors(groups);
    expect(html.match(/<select /g)).toHaveLength(2);
    expect(html).toContain("Style</span>");
    expect(html).toContain("Flavor</span>");
    expect(html).toContain('value="skyr" selected=""');
    expect(html).toContain('value="peach" selected=""');
    expect(html).toContain("<label for=");
  });
});

function fruitIds(flavor: string) {
  if (flavor === "plain") return [];
  return [flavor === "peach" ? "peaches" : "cherries"];
}

function flavorSteps(flavor: string) {
  return flavor === "plain" ? [] : [`${flavor}Prep`];
}

function renderSelectors(variationGroups: RecipeVariationGroup[]) {
  return renderToStaticMarkup(
    createElement(RecipeVariations, {
      groups: variationGroups,
      selection: { style: "skyr", flavor: "peach" },
      onChange: () => {},
    }),
  );
}
