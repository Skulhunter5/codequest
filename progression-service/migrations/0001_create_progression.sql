CREATE TABLE progression (
    quest_id      TEXT NOT NULL,
    username      TEXT NOT NULL,
    completed_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (username, quest_id)
);
