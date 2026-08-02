ALTER TABLE book_recommendations
    ADD COLUMN page_count INTEGER CHECK (page_count > 0),
    ADD COLUMN purchase_link TEXT;

CREATE TABLE book_tags (
    book_id     UUID NOT NULL REFERENCES book_recommendations(id) ON DELETE CASCADE,
    tag_key     TEXT NOT NULL,
    tag_value   TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (book_id, tag_key, tag_value),
    CONSTRAINT book_tags_normalized CHECK (
        tag_key = lower(btrim(tag_key))
        AND tag_value = lower(btrim(tag_value))
        AND length(tag_key) > 0
        AND length(tag_value) > 0
    )
);

CREATE INDEX idx_book_tags_corpus
    ON book_tags (tag_key, tag_value);
