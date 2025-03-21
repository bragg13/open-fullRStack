-- Add migration script here
CREATE TABLE blogs (
    id SERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    url TEXT NOT NULL,
    likes INT NULL DEFAULT 0,
    created_at TIMESTAMP DEFAULT NOW ()
);
