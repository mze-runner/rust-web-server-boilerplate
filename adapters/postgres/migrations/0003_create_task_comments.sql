CREATE TABLE IF NOT EXISTS task_comments (
    id          UUID          PRIMARY KEY,
    task_id     UUID          NOT NULL REFERENCES tasks(id),
    author_id   UUID          NOT NULL REFERENCES users(id),
    body        VARCHAR(1000) NOT NULL,
    created_at  TIMESTAMPTZ   NOT NULL,
    modified_by UUID          NOT NULL REFERENCES users(id),
    modified_at TIMESTAMPTZ   NOT NULL
);
