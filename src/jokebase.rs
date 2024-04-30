use crate::*;

#[derive(Debug, thiserror::Error, ToSchema, Serialize)]
pub enum JokeBaseErr {
    #[error("joke already exists: {0}")]
    JokeExists(String),
    #[error("jokebase io failed: {0}")]
    JokeBaseIoError(String),
    #[error("no joke")]
    NoJoke,
    #[error("joke {0} doesn't exist")]
    JokeDoesNotExist(String),
    #[error("joke payload unprocessable")]
    JokeUnprocessable(String),
}

impl From<std::io::Error> for JokeBaseErr {
    fn from(e: std::io::Error) -> Self {
        JokeBaseErr::JokeBaseIoError(e.to_string())
    }
}

#[derive(Debug)]
pub struct JokeBaseError {
    pub status: StatusCode,
    pub error: JokeBaseErr,
}

impl<'s> ToSchema<'s> for JokeBaseError {
    fn schema() -> (&'s str, RefOr<Schema>) {
        let sch = ObjectBuilder::new()
            .property(
                "status",
                ObjectBuilder::new().schema_type(SchemaType::String),
            )
            .property(
                "error",
                ObjectBuilder::new().schema_type(SchemaType::String),
            )
            .example(Some(serde_json::json!({
                "status":"404","error":"no joke"
            })))
            .into();
        ("JokeBaseError", sch)
    }
}

impl Serialize for JokeBaseError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        use std::env::var;
        
        let pwf = var("PG_PASSWORDFILE")?;
        let password = std::fs::read_to_string(pwf)?;
        let url = format!(
            "postgres://{}:{}@{}:5432/{}",
            var("PG_USER")?,
            password.trim(),
            var("PG_HOST")?,
            var("PG_DBNAME")?,
        );
        let pool = PgPool::connect(&url).await?;
        sqlx::migrate!()
            .run(&pool)
            .await?;
        Ok(JokeBase(pool))
    }

    pub fn get_random(&self) -> Result<Joke, JokeBaseErr> {
        todo!()
    }

    pub fn get<'a>(&self, index: &str) -> Result<Joke, JokeBaseErr> {
        todo!()
    }

    pub fn get_jokes<'a>(&self) -> Vec<Joke> {
        todo!()
    }

    pub fn add(&mut self, joke: Joke) -> Result<(), JokeBaseErr> {
        todo!()
    }

    pub fn delete(&mut self, index: &str) -> Result<(), JokeBaseErr> {
        todo!()
    }

    pub fn update(&mut self, index: &str, joke: Joke) -> Result<(), JokeBaseErr> {
        todo!()
    }
}
