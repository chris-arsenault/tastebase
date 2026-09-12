\set ON_ERROR_STOP on
BEGIN;
CREATE SCHEMA shelf_test;
SET LOCAL search_path TO shelf_test, public;
CREATE TABLE users (id UUID PRIMARY KEY);
\ir ../migrations/011_create_book_recommendations.sql
\ir ../migrations/012_single_owner_books.sql
\ir ../migrations/013_drop_book_user_id.sql
\ir ../migrations/014_book_metadata_and_tags.sql
INSERT INTO book_recommendations (title, author, summary, why_recommended)
VALUES ('Unread book', 'Author', 'Summary', 'Reason');
INSERT INTO book_recommendations (title, author, summary, why_recommended, rating, writeup, is_public)
VALUES ('Reviewed book', 'Author', 'Summary', 'Reason', 4, 'Worth reading', true);
\ir ../migrations/015_public_books_and_publications.sql

DO $$
BEGIN
    IF (SELECT count(*) FROM book_recommendations WHERE is_public) <> 2 THEN
        RAISE EXCEPTION 'Migration did not make all existing books public';
    END IF;
    IF NOT EXISTS (SELECT FROM book_recommendations WHERE title = 'Unread book' AND rating IS NULL AND writeup = '') THEN
        RAISE EXCEPTION 'Migration changed reading feedback';
    END IF;
END $$;

INSERT INTO book_recommendations (title, author, summary, why_recommended)
VALUES ('New book', 'Author', 'Summary', 'Reason');
INSERT INTO publication_recommendations (title, publisher, summary, why_recommended, details)
VALUES ('Journal', 'Publisher', 'Summary', 'Reason', '{"publicationType":"journal"}');
UPDATE publication_recommendations SET status = 'cancelled', rating = 3, writeup = 'Good but too costly';
INSERT INTO publication_recommendations (title, publisher, summary, why_recommended, details)
VALUES ('JOURNAL', 'PUBLISHER', 'Updated summary', 'New reason', '{"publicationType":"journal"}')
ON CONFLICT (lower(title), lower(COALESCE(publisher, '')))
DO UPDATE SET summary = EXCLUDED.summary, why_recommended = EXCLUDED.why_recommended, details = EXCLUDED.details;

INSERT INTO publication_tags (publication_id, tag_key, tag_value)
SELECT id, 'outlook', value FROM publication_recommendations CROSS JOIN (VALUES ('humanist'), ('rationalist')) tags(value);
INSERT INTO book_tags (book_id, tag_key, tag_value)
SELECT id, 'outlook', 'humanist' FROM book_recommendations WHERE title = 'New book';

DO $$
BEGIN
    IF NOT (SELECT is_public FROM book_recommendations WHERE title = 'New book') THEN
        RAISE EXCEPTION 'New books must default to public';
    END IF;
    IF (SELECT count(*) FROM publication_recommendations) <> 1 THEN
        RAISE EXCEPTION 'Publication identity did not deduplicate';
    END IF;
    IF NOT EXISTS (SELECT FROM publication_recommendations WHERE status = 'cancelled' AND rating = 3 AND writeup = 'Good but too costly') THEN
        RAISE EXCEPTION 'Recommendation refresh erased feedback';
    END IF;
    IF (SELECT count(*) FROM publication_tags WHERE tag_key = 'outlook') <> 2 THEN
        RAISE EXCEPTION 'Multiple outlook values must be preserved';
    END IF;
    BEGIN
        UPDATE publication_recommendations SET status = 'reading';
        RAISE EXCEPTION 'Book status incorrectly accepted for publication';
    EXCEPTION WHEN check_violation THEN NULL;
    END;
END $$;
ROLLBACK;
