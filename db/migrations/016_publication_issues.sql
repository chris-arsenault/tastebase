-- Issue feedback is separate from the original publication recommendation.
-- Existing publication-level feedback stays on the publication: its issue is unknown.
CREATE TABLE publication_issues (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    publication_id UUID NOT NULL REFERENCES publication_recommendations(id) ON DELETE CASCADE,
    label TEXT NOT NULL CHECK (length(btrim(label)) BETWEEN 1 AND 200),
    metadata JSONB NOT NULL DEFAULT '{}' CHECK (jsonb_typeof(metadata) = 'object'),
    rating SMALLINT CHECK (rating BETWEEN 1 AND 5),
    writeup TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (rating IS NULL OR length(btrim(writeup)) > 0)
);

CREATE UNIQUE INDEX idx_publication_issue_identity
    ON publication_issues (publication_id, lower(label));
