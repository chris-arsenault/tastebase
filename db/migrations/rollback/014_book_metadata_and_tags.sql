DROP TABLE IF EXISTS book_tags;

ALTER TABLE book_recommendations
    DROP COLUMN IF EXISTS purchase_link,
    DROP COLUMN IF EXISTS page_count;
