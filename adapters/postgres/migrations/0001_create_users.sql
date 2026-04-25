CREATE TABLE IF NOT EXISTS users (
    id         UUID        PRIMARY KEY,
    username   VARCHAR(30) NOT NULL UNIQUE,
    email      VARCHAR(64) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);
