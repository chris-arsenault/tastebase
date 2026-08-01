-- The single-owner application no longer reads or writes this legacy identity
-- link. Drop it only after that application version has been deployed.
ALTER TABLE book_recommendations
    DROP COLUMN user_id;
