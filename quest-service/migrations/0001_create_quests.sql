CREATE TABLE quests (
    id           UUID PRIMARY KEY DEFAULT uuidv7(),
    name         TEXT NOT NULL,
    description  TEXT NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
