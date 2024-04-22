use crate::*;

#[derive(Debug, thiserror::Error, ToSchema, Serialize)]
pub enum JokeBaseErr {
    #[error("joke already exists: {0}")]
    JokeExists(String),
    #[error("jokebase io failed: {0}")]
    JokeBaseIoError(String),
    #[error("no joke")]
    NoJoke,
}

impl From<std::io::Error> for JokeBaseErr {
    fn from(e: std::io::Error) -> Self {
        JokeBaseErr::JokeBaseIoError(e.to_string())
    }
}

#[derive(Debug, ToSchema)]
pub struct JokeBaseError {
    #[schema(example = "404")]
    pub status: StatusCode,
    #[schema(example = "no joke")]
    pub error: JokeBaseErr,
}

impl Serialize for JokeBaseError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
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
        let error = JokeBaseError {
            status,
            error,
        };
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

    pub fn get_random(&self) -> Option<&Joke> {
        fastrand::choice(self.jokemap.iter()).map(|x| x.1)
    }

    pub fn get<'a>(&'a self, index: &str) -> Option<&'a Joke> {
        self.jokemap.get(index)
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
}

impl IntoResponse for &JokeBase {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(&self.jokemap)).into_response()
    }
}
