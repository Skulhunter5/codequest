CREATE TABLE progression (
    user_id       UUID NOT NULL,
    quest_id      UUID NOT NULL,
    completed_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, quest_id)
);
