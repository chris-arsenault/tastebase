CREATE TYPE product_type AS ENUM ('sauce', 'drink');
CREATE TYPE processing_status AS ENUM (
    'pending', 'image_extracted', 'ingredients_extracted',
    'nutrition_extracted', 'voice_transcribed', 'voice_extracted',
    'notes_formatted', 'back_extracted', 'complete', 'error'
);

CREATE TABLE tastings (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id                 UUID REFERENCES users(id),
    product_type            product_type,
    name                    TEXT NOT NULL DEFAULT '',
    maker                   TEXT NOT NULL DEFAULT '',
    date                    DATE NOT NULL DEFAULT CURRENT_DATE,
    score                   SMALLINT,
    style                   TEXT NOT NULL DEFAULT '',
    heat_user               SMALLINT,
    heat_vendor             SMALLINT,
    refreshing              SMALLINT,
    sweet                   SMALLINT,
    tasting_notes_user      TEXT NOT NULL DEFAULT '',
    tasting_notes_vendor    TEXT NOT NULL DEFAULT '',
    product_url             TEXT NOT NULL DEFAULT '',
    image_url               TEXT,
    image_key               TEXT,
    ingredients_image_url   TEXT,
    ingredients_image_key   TEXT,
    nutrition_image_url     TEXT,
    nutrition_image_key     TEXT,
    nutrition_facts         JSONB,
    ingredients             TEXT[],
    voice_key               TEXT,
    voice_transcript        TEXT,
    status                  processing_status NOT NULL DEFAULT 'pending',
    processing_error        TEXT,
    needs_attention         BOOLEAN NOT NULL DEFAULT FALSE,
    attention_reason        TEXT,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_tastings_user_id ON tastings(user_id);
CREATE INDEX idx_tastings_date ON tastings(date DESC);
CREATE INDEX idx_tastings_product_type ON tastings(product_type);
