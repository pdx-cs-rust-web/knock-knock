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

type JokeMap = HashMap<String, Joke>;

#[derive(Debug)]
pub struct JokeBase {
    file: File,
    jokemap: JokeMap,
}

impl JokeBase {
    pub fn new<P: AsRef<std::path::Path>>(db_path: P) -> Result<Self, std::io::Error> {
        let mut file = File::create_new(&db_path)
            .and_then(|mut f| {
                let jokemap: JokeMap = HashMap::new();
                let json = serde_json::to_string(&jokemap).unwrap();
                f.write_all(json.as_bytes())?;
                f.sync_all()?;
                f.rewind()?;
                Ok(f)
            })
            .or_else(|e| {
                if e.kind() == ErrorKind::AlreadyExists {
                    File::options().read(true).write(true).open(&db_path)
                } else {
                    Err(e)
                }
            })?;
        let json = std::io::read_to_string(&mut file)?;
        let jokemap = serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))?;
        Ok(Self { file, jokemap })
    }

    pub fn get_random(&self) -> Result<&Joke, JokeBaseErr> {
        let (_, joke) = fastrand::choice(self.jokemap.iter()).ok_or(JokeBaseErr::NoJoke)?;
        Ok(joke)
    }

    pub fn get<'a>(&'a self, index: &str) -> Result<&'a Joke, JokeBaseErr> {
        self.jokemap
            .get(index)
            .ok_or(JokeBaseErr::JokeDoesNotExist(index.to_string()))
    }

    fn write_jokes(&mut self) -> Result<(), std::io::Error> {
        let json = serde_json::to_string(&self.jokemap).unwrap();
        self.file.rewind()?;
        self.file.set_len(0)?;
        self.file.write_all(json.as_bytes())?;
        self.file.sync_all()
    }

    pub fn add(&mut self, joke: Joke) -> Result<(), JokeBaseErr> {
        let id = joke.id.clone();
        if self.jokemap.get(&id).is_some() {
            return Err(JokeBaseErr::JokeExists(id));
        }
        self.jokemap.insert(id, joke);
        self.write_jokes()?;
        Ok(())
    }

    pub fn delete(&mut self, index: &str) -> Result<(), JokeBaseErr> {
        if !self.jokemap.contains_key(index) {
            return Err(JokeBaseErr::JokeDoesNotExist(index.to_string()));
        }
        self.jokemap.remove(index);
        self.write_jokes()?;
        Ok(())
    }

    pub fn update(&mut self, index: &str, joke: Joke) -> Result<StatusCode, JokeBaseErr> {
        if !self.jokemap.contains_key(index) {
            return Err(JokeBaseErr::NoJoke);
        }
        if joke.id.is_empty() {
            return Err(JokeBaseErr::JokeUnprocessable(index.to_string()));
        }
        self.jokemap
            .entry(index.to_string())
            .and_modify(|x| *x = joke);
        self.write_jokes()?;
        Ok(StatusCode::OK)
    }
}

impl IntoResponse for &JokeBase {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(&self.jokemap)).into_response()
    }
}
