CREATE TABLE IF NOT EXISTS tasks (
    id          UUID         PRIMARY KEY,
    subject     VARCHAR(256) NOT NULL,
    description VARCHAR(500),
    status      VARCHAR(20)  NOT NULL,
    assignee_id UUID         NOT NULL REFERENCES users(id),
    created_by  UUID         NOT NULL REFERENCES users(id),
    created_at  TIMESTAMPTZ  NOT NULL,
    modified_by UUID         NOT NULL REFERENCES users(id),
    modified_at TIMESTAMPTZ  NOT NULL
);
