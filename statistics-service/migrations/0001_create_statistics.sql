CREATE TABLE statistics (
    user_id       UUID NOT NULL,
    metric_key    TEXT NOT NULL,
    metric_value  BIGINT NOT NULL,
    PRIMARY KEY (user_id, metric_key)
);
