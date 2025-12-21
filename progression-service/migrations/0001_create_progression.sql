CREATE TABLE progression (
    quest_id      UUID NOT NULL,
    username      TEXT NOT NULL,
    completed_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (username, quest_id)
);
