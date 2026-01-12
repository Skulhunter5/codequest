CREATE TABLE users (
    id             UUID PRIMARY KEY DEFAULT uuidv4(),
    username       TEXT NOT NULL UNIQUE,
    password_hash  TEXT NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);
