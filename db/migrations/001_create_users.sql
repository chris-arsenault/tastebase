CREATE TABLE users (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email       TEXT UNIQUE NOT NULL,
    display_name TEXT NOT NULL DEFAULT '',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE cognito_users (
    cognito_sub TEXT PRIMARY KEY,
    user_id     UUID NOT NULL REFERENCES users(id),
    email       TEXT NOT NULL,
    linked_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_cognito_users_user_id ON cognito_users(user_id);
