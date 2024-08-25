-- Your SQL goes here
-- up.sql
CREATE TABLE posts (
    id SERIAL PRIMARY KEY,
    post_id VARCHAR NOT NULL,
    title VARCHAR NOT NULL,
    body TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE skills (
    id SERIAL PRIMARY KEY,
    skill_id VARCHAR NOT NULL,
    skill_name VARCHAR NOT NULL,
    body TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE worklog (
    id SERIAL PRIMARY KEY,
    worklog_id VARCHAR NOT NULL,
    worklog_title VARCHAR NOT NULL,
    body TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

