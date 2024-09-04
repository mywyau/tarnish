-- Your SQL goes here
-- up.sql
CREATE TABLE posts (
    id SERIAL PRIMARY KEY,
    post_id VARCHAR NOT NULL,
    title VARCHAR NOT NULL,
    body TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE skills (
    id SERIAL PRIMARY KEY,
    skill_id VARCHAR NOT NULL,
    skill_name VARCHAR NOT NULL,
    body TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE worklog (
    id SERIAL PRIMARY KEY,
    worklog_id VARCHAR NOT NULL,
    work_title VARCHAR NOT NULL,
    body TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE roles (
    id SERIAL PRIMARY KEY,
    userType VARCHAR(50) NOT NULL
);

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    role_id INTEGER REFERENCES roles(id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT now(),
);

-- Insert roles (e.g., admin, editor, viewer)
INSERT INTO roles (name) VALUES ('admin'), ('editor'), ('viewer');


