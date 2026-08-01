CREATE TYPE book_status AS ENUM ('recommended', 'reading', 'read', 'did_not_finish');

CREATE TABLE book_recommendations (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id             UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title               TEXT NOT NULL,
    author              TEXT NOT NULL,
    summary             TEXT NOT NULL,
    why_recommended     TEXT NOT NULL,
    status              book_status NOT NULL DEFAULT 'recommended',
    rating              SMALLINT CHECK (rating BETWEEN 1 AND 5),
    writeup             TEXT NOT NULL DEFAULT '',
    is_public           BOOLEAN NOT NULL DEFAULT false,
    recommended_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    read_at             TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT public_books_require_feedback CHECK (
        NOT is_public OR (
            rating IS NOT NULL
            AND length(btrim(writeup)) > 0
        )
    )
);

CREATE UNIQUE INDEX idx_book_recommendations_user_title_author
    ON book_recommendations (user_id, lower(title), lower(author));
CREATE INDEX idx_book_recommendations_user_status
    ON book_recommendations (user_id, status, recommended_at DESC);
CREATE INDEX idx_book_recommendations_public
    ON book_recommendations (read_at DESC)
    WHERE is_public = true;
