# knock-knock: Knock-Knock Joke Server
Bart Massey 2024

This thing serves knock-knock jokes over a REST API backed by
PostgreSQL, with a small HTML frontend for browsing and adding
jokes.

## Running

The easiest path is `docker compose up --build`, which launches
the server on port 3000 alongside a Postgres database. Before
the first run, populate the three secret files read by
`compose.yaml`:

* `db/password.txt` — Postgres password
* `db/jwt-secret.txt` — signing key for JWT tokens
* `db/reg-password.txt` — shared password required to register
  and obtain a JWT

To run outside Docker, point the following environment
variables at a Postgres instance and secret files, then
`cargo run`:

| Variable          | Purpose                               |
|-------------------|---------------------------------------|
| `PG_HOST`         | Postgres host                         |
| `PG_USER`         | Postgres user                         |
| `PG_DBNAME`       | Database name                         |
| `PG_PASSWORDFILE` | Path to file containing DB password   |
| `JWT_SECRETFILE`  | Path to file with the JWT signing key |
| `REG_PASSWORD`    | Path to file with the shared reg pw   |

The server migrates the schema on startup (see `migrations/`).
The listen address defaults to `0.0.0.0:3000` and can be
overridden with `--serve <addr>`.

## API reference

While the server is running, interactive API documentation is
available at:

* `/swagger-ui` — Swagger UI
* `/redoc` — ReDoc
* `/rapidoc` — RapiDoc

All three are generated from the `#[utoipa::path(...)]`
annotations in `src/api.rs` and share a single OpenAPI spec at
`/api-docs/openapi.json`.

## Authentication

Write endpoints (`POST /api/v1/joke/add`, `PUT` and `DELETE`
on `/api/v1/joke/{id}`) require a bearer JWT. Obtain one by
registering with the shared `REG_PASSWORD`:

```sh
curl http://localhost:3000/api/v1/register \
    -H 'Content-Type: application/json' \
    -d '{"full_name":"Jane Doe",
         "email":"jane@example.org",
         "password":"<contents of reg-password.txt>"}'
```

The response is `{"access_token": "...", "token_type": "Bearer"}`.
Pass the token on subsequent requests as
`Authorization: Bearer <token>`. Tokens expire after one day.

## HTML frontend

* `GET /` — show a random joke (or `?id=<joke-id>` for a
  specific one)
* `GET /tell` — form for adding a new joke
* `GET /add` — submit handler for the form above

See also `test-api.py` for a Python driver exercising the
REST endpoints end-to-end.

## Acknowledgements

Knock-knock jokes are from memory, my students, and from:

* Parade Magazine (1 joke, fair use) <https://parade.com/944054/parade/knock-knock-jokes/>
* La Jolla Mom Magazine (141 jokes, implicit permission):
  <https://lajollamom.com/kid-friendly-knock-knock-jokes>

Bastian Gruber's book *Rust Web Development* inspired this.
Many web tutorials and blog posts were perused to learn
Axum.

# License

With the content exceptions noted above, this work is
licensed under the "MIT License". Please see the file
`LICENSE.txt` in this distribution for license terms.
