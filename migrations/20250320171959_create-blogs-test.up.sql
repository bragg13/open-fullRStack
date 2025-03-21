-- Add up migration script here
CREATE TABLE blogs_test (
    id SERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    url TEXT NOT NULL,
    likes INT NULL DEFAULT 0,
    created_at TIMESTAMP DEFAULT NOW ()
);
