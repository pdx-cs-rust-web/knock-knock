use crate::*;

#[derive(Debug, thiserror::Error, ToSchema, Serialize)]
// XXX Fixme!
#[allow(dead_code)]
pub enum JokeBaseErr {
    #[error("joke already exists: {0}")]
    JokeExists(String),
    #[error("jokebase io failed: {0}")]
    JokeBaseIoError(String),
    #[error("no joke")]
    NoJoke,
    #[error("joke {0} doesn't exist")]
    JokeDoesNotExist(String),
    #[error("joke payload unprocessable: {0}")]
    JokeUnprocessable(String),
    #[error("database error: {0}")]
    DatabaseError(String),
}

impl From<std::io::Error> for JokeBaseErr {
    fn from(e: std::io::Error) -> Self {
        JokeBaseErr::JokeBaseIoError(e.to_string())
    }
}

impl From<sqlx::Error> for JokeBaseErr {
    fn from(e: sqlx::Error) -> Self {
        JokeBaseErr::DatabaseError(e.to_string())
    }
}

#[derive(Debug)]
pub struct JokeBaseError {
    pub status: StatusCode,
    pub error: JokeBaseErr,
}

pub fn error_schema(name: &str, example: serde_json::Value) -> (&str, RefOr<Schema>) {
    let sch = ObjectBuilder::new()
        .property(
            "status",
            ObjectBuilder::new().schema_type(SchemaType::String),
        )
        .property(
            "error",
            ObjectBuilder::new().schema_type(SchemaType::String),
        )
        .example(Some(example))
        .into();
    (name, sch)
}

impl<'s> ToSchema<'s> for JokeBaseError {
    fn schema() -> (&'s str, RefOr<Schema>) {
        let example = serde_json::json!({
            "status":"404","error":"no joke"
        });
        error_schema("JokeBaseError", example)
    }
}

impl Serialize for JokeBaseError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let status: String = self.status.to_string();
        let mut state = serializer.serialize_struct("JokeBaseError", 2)?;
        state.serialize_field("status", &status)?;
        state.serialize_field("error", &self.error)?;
        state.end()
    }
}

impl JokeBaseError {
    pub fn response(status: StatusCode, error: JokeBaseErr) -> Response {
        let error = JokeBaseError { status, error };
        (status, Json(error)).into_response()
    }
}

#[derive(Debug)]
pub struct JokeBase(pub Pool<Postgres>);

impl JokeBase {
    async fn to_joke(&self, row: &PgRow) -> Result<Joke, sqlx::Error> {
        let id = row.get("id");
        let tags = sqlx::query(r#"SELECT tag FROM tags WHERE id = $1"#)
            .bind(&id)
            .fetch_all(&self.0)
            .await?;
        let tags: HashSet<String> = tags.iter().map(|row| row.get("tag")).collect();
        let tags = if tags.is_empty() { None } else { Some(tags) };
        Ok(Joke {
            id,
            whos_there: row.get("whos_there"),
            answer_who: row.get("answer_who"),
            source: row.get("source"),
            tags,
        })
    }

    async fn insert_tags(
        tx: &mut PgConnection,
        id: &str,
        tags: &Option<HashSet<String>>,
    ) -> Result<(), sqlx::Error> {
        if let Some(tags) = tags {
            for tag in tags {
                sqlx::query(r#"INSERT INTO tags (id, tag) VALUES ($1, $2);"#)
                    .bind(id)
                    .bind(tag)
                    .execute(&mut *tx)
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn new() -> Result<Self, Box<dyn Error>> {
        use std::env::var;

        let password = read_secret("PG_PASSWORDFILE").await?;
        let url = format!(
            "postgres://{}:{}@{}:5432/{}",
            var("PG_USER")?,
            password.trim(),
            var("PG_HOST")?,
            var("PG_DBNAME")?,
        );
        let pool = PgPool::connect(&url).await?;
        sqlx::migrate!().run(&pool).await?;
        Ok(JokeBase(pool))
    }

    pub async fn get_random(&self) -> Result<Joke, JokeBaseErr> {
        let row = sqlx::query(r#"SELECT * FROM jokes ORDER BY RANDOM () LIMIT 1;"#)
            .fetch_one(&self.0)
            .await?;

        let joke = self.to_joke(&row).await?;
        Ok(joke)
    }

    pub async fn get<'a>(&self, index: &str) -> Result<Joke, JokeBaseErr> {
        let row = sqlx::query(r#"SELECT * FROM jokes WHERE id = $1;"#)
            .bind(index)
            .fetch_one(&self.0)
            .await?;

        let joke = self.to_joke(&row).await?;
        Ok(joke)
    }

    pub async fn get_jokes<'a>(&self) -> Result<Vec<Joke>, JokeBaseErr> {
        let rows = sqlx::query(r#"SELECT * FROM jokes;"#)
            .fetch_all(&self.0)
            .await?;
        let mut jokes: Vec<Joke> = Vec::with_capacity(rows.len());
        for j in rows.iter() {
            jokes.push(self.to_joke(j).await?);
        }
        Ok(jokes)
    }

    pub async fn add(&mut self, joke: Joke) -> Result<(), JokeBaseErr> {
        let mut tx = Pool::begin(&self.0).await?;
        let result = sqlx::query(
            r#"INSERT INTO jokes
            (id, whos_there, answer_who, source)
            VALUES ($1, $2, $3, $4);"#,
        )
        .bind(&joke.id)
        .bind(&joke.whos_there)
        .bind(&joke.answer_who)
        .bind(&joke.source)
        .execute(&mut *tx)
        .await;
        result.map_err(|e| {
            if let sqlx::Error::Database(ref dbe) = e {
                if let Some("23505") = dbe.code().as_deref() {
                    return JokeBaseErr::JokeExists(joke.id.to_string());
                }
            }
            JokeBaseErr::DatabaseError(e.to_string())
        })?;
        Self::insert_tags(&mut tx, &joke.id, &joke.tags).await?;
        Ok(tx.commit().await?)
    }

    pub async fn delete(&mut self, index: &str) -> Result<(), JokeBaseErr> {
        let mut tx = Pool::begin(&self.0).await?;
        sqlx::query(r#"DELETE FROM tags WHERE id = $1;"#)
            .bind(index)
            .execute(&mut *tx)
            .await?;
        let result = sqlx::query(r#"DELETE FROM jokes WHERE id = $1 RETURNING jokes.id;"#)
            .bind(index)
            .fetch_all(&mut *tx)
            .await?;
        if result.len() == 0 {
            return Err(JokeBaseErr::JokeDoesNotExist(index.to_string()));
        }
        Ok(tx.commit().await?)
    }

    pub async fn update(&mut self, index: &str, joke: Joke) -> Result<(), JokeBaseErr> {
        let mut tx = Pool::begin(&self.0).await?;
        let q = sqlx::query(
            r#"UPDATE jokes
            SET (whos_there, answer_who, source) = ($2, $3, $4)
            WHERE jokes.id = $1
            RETURNING jokes.id;"#,
        );
        let result = q.bind(&joke.id)
            .bind(&joke.whos_there)
            .bind(&joke.answer_who)
            .bind(&joke.source)
            .fetch_all(&mut *tx)
            .await?;
        if result.len() == 0 {
            return Err(JokeBaseErr::JokeDoesNotExist(index.to_string()));
        }
        sqlx::query(r#"DELETE FROM tags WHERE id = $1;"#)
            .bind(index)
            .execute(&mut *tx)
            .await?;
        Self::insert_tags(&mut tx, &joke.id, &joke.tags).await?;
        Ok(tx.commit().await?)
    }
}
