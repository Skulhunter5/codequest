CREATE TABLE statistics (
    username      TEXT NOT NULL,
    metric_key    TEXT NOT NULL,
    metric_value  BIGINT NOT NULL,
    PRIMARY KEY (username, metric_key)
);
