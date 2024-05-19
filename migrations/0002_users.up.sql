CREATE TABLE IF NOT EXISTS users
(
    username     text primary key not null unique,
    access_token text not null
);

