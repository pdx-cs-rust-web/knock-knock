-- Add up migration script here

CREATE TABLE IF NOT EXISTS passwords (
  client_id TEXT PRIMARY KEY,
  client_secret TEXT NOT NULL,
  full_name TEXT NOT NULL,
  email TEXT NOT NULL
);
