CREATE TYPE recipe_source AS ENUM ('claude', 'manual', 'import');
CREATE TYPE unit_type AS ENUM (
    'g', 'kg', 'ml', 'l', 'tsp', 'tbsp', 'cup',
    'fl_oz', 'oz', 'lb', 'pinch', 'piece', ''
);

CREATE TABLE recipes (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id),
    title           TEXT NOT NULL,
    description     TEXT,
    base_servings   INT NOT NULL DEFAULT 1,
    notes           TEXT,
    source          recipe_source NOT NULL DEFAULT 'manual',
    source_meta     JSONB,
    cover_image_url TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE recipe_ingredients (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recipe_id   UUID NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    widget_id   TEXT NOT NULL,
    name        TEXT NOT NULL,
    amount      DECIMAL NOT NULL,
    unit        unit_type NOT NULL DEFAULT '',
    sort_order  INT NOT NULL DEFAULT 0
);

CREATE TABLE recipe_steps (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recipe_id       UUID NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    widget_id       TEXT NOT NULL,
    title           TEXT NOT NULL,
    content         TEXT NOT NULL,
    timer_seconds   INT,
    sort_order      INT NOT NULL DEFAULT 0
);

CREATE TABLE collections (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id),
    name        TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE collection_recipes (
    collection_id   UUID NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
    recipe_id       UUID NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    added_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (collection_id, recipe_id)
);

CREATE INDEX idx_recipes_user_id ON recipes(user_id);
CREATE INDEX idx_recipe_ingredients_recipe_id ON recipe_ingredients(recipe_id);
CREATE INDEX idx_recipe_steps_recipe_id ON recipe_steps(recipe_id);
CREATE INDEX idx_collections_user_id ON collections(user_id);
