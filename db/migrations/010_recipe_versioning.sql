-- Add versioning support to recipes.
-- version_group_id groups all versions of the same recipe together.
-- version is the incrementing version number within a group.

ALTER TABLE recipes ADD COLUMN version INT NOT NULL DEFAULT 1;
ALTER TABLE recipes ADD COLUMN version_group_id UUID;

-- Backfill: each existing recipe is its own group at version 1
UPDATE recipes SET version_group_id = id;

ALTER TABLE recipes ALTER COLUMN version_group_id SET NOT NULL;

CREATE INDEX idx_recipes_version_group ON recipes(version_group_id, version DESC);
