CREATE TABLE recipe_reviews (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recipe_id           UUID NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    voice_key           TEXT,
    voice_transcript    TEXT,
    notes               TEXT NOT NULL DEFAULT '',
    score               SMALLINT,
    status              processing_status NOT NULL DEFAULT 'pending',
    processing_error    TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE recipe_images (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recipe_id   UUID NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    image_url   TEXT NOT NULL,
    image_key   TEXT NOT NULL,
    caption     TEXT NOT NULL DEFAULT '',
    sort_order  INT NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_recipe_reviews_recipe_id ON recipe_reviews(recipe_id);
CREATE INDEX idx_recipe_images_recipe_id ON recipe_images(recipe_id);
