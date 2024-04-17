use crate::*;

#[derive(Debug, thiserror::Error)]
pub enum JokeBaseError {
    #[error("joke already exists: {0}")]
    JokeExists(JokeId),
    #[error("jokebase write: {0}")]
    JokeFileWrite(#[from] std::io::Error),
}

type JokeMap = HashMap<JokeId, Joke>;

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

    pub fn get(&self, index: &JokeId) -> Option<&Joke> {
        self.jokemap.get(index)
    }

    fn write_jokes(&mut self) -> Result<(), std::io::Error> {
        let json = serde_json::to_string(&self.jokemap).unwrap();
        self.file.rewind()?;
        self.file.set_len(0)?;
        self.file.write_all(json.as_bytes())?;
        self.file.sync_all()
    }

    pub fn add(&mut self, joke: Joke) -> Result<(), JokeBaseError> {
        let id = joke.id().clone();
        if self.jokemap.get(&id).is_some() {
            return Err(JokeBaseError::JokeExists(id));
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
