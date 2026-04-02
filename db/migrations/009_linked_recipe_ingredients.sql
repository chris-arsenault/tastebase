-- Allow an ingredient to reference another recipe as a sub-component.
-- Example: "1 cup olive tapenade" links to the full tapenade recipe.
-- ON DELETE SET NULL: if the linked recipe is deleted, the ingredient
-- gracefully becomes a plain ingredient.

ALTER TABLE recipe_ingredients
  ADD COLUMN linked_recipe_id UUID REFERENCES recipes(id) ON DELETE SET NULL;

CREATE INDEX idx_recipe_ingredients_linked
  ON recipe_ingredients(linked_recipe_id)
  WHERE linked_recipe_id IS NOT NULL;
