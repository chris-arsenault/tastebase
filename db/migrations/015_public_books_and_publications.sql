-- Keep the legacy column during rolling deployment, but make visibility universal.
ALTER TABLE book_recommendations DROP CONSTRAINT public_books_require_feedback;
ALTER TABLE book_recommendations ALTER COLUMN is_public SET DEFAULT true;
UPDATE book_recommendations SET is_public = true;

CREATE TABLE publication_recommendations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title TEXT NOT NULL CHECK (length(btrim(title)) > 0),
    publisher TEXT,
    summary TEXT NOT NULL CHECK (length(btrim(summary)) > 0),
    why_recommended TEXT NOT NULL CHECK (length(btrim(why_recommended)) > 0),
    -- Typed publication metadata: cadence, audience, editorial home, outlook,
    -- homepage and subscription offers. Offers are replaced as a complete set.
    details JSONB NOT NULL CHECK (jsonb_typeof(details) = 'object'),
    status TEXT NOT NULL DEFAULT 'recommended'
        CHECK (status IN ('recommended', 'subscribed', 'cancelled', 'not_interested')),
    rating SMALLINT CHECK (rating BETWEEN 1 AND 5),
    writeup TEXT NOT NULL DEFAULT '',
    recommended_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX idx_publication_identity
    ON publication_recommendations (lower(title), lower(COALESCE(publisher, '')));
CREATE INDEX idx_publication_status
    ON publication_recommendations (status, recommended_at DESC);

CREATE TABLE publication_tags (
    publication_id UUID NOT NULL REFERENCES publication_recommendations(id) ON DELETE CASCADE,
    tag_key TEXT NOT NULL,
    tag_value TEXT NOT NULL,
    PRIMARY KEY (publication_id, tag_key, tag_value),
    CHECK (tag_key = lower(btrim(tag_key)) AND length(tag_key) > 0),
    CHECK (tag_value = lower(btrim(tag_value)) AND length(tag_value) > 0)
);
CREATE INDEX idx_publication_tags_corpus ON publication_tags (tag_key, tag_value);
