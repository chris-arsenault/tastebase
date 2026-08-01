-- Tastebase has one authenticated owner. Keep the old identity column nullable
-- during the rolling application deploy, but do not use it as a tenancy boundary.
DROP INDEX IF EXISTS idx_book_recommendations_user_status;
DROP INDEX IF EXISTS idx_book_recommendations_user_title_author;

ALTER TABLE book_recommendations
    ALTER COLUMN user_id DROP NOT NULL;

CREATE UNIQUE INDEX idx_book_recommendations_title_author
    ON book_recommendations (lower(title), lower(author));
CREATE INDEX idx_book_recommendations_status
    ON book_recommendations (status, recommended_at DESC);
