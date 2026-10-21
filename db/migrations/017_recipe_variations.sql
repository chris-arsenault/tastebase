-- Independent variation groups; existing recipe content always applies.
ALTER TABLE recipes
    ADD COLUMN variation_groups JSONB NOT NULL DEFAULT '[]'::jsonb
    CHECK (jsonb_typeof(variation_groups) = 'array');

ALTER TABLE recipe_ingredients
    ADD COLUMN applies_when JSONB NOT NULL DEFAULT '{}'::jsonb
    CHECK (jsonb_typeof(applies_when) = 'object');

ALTER TABLE recipe_steps
    ADD COLUMN applies_when JSONB NOT NULL DEFAULT '{}'::jsonb
    CHECK (jsonb_typeof(applies_when) = 'object');
