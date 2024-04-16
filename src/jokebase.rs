use crate::*;

pub static JOKEBASE: &[(&str, &str, &[&str])] = &[
    (
        "Boo",
        "You don't have to cry about it!",
        &["kids", "oldie"],
    ),
    (
        "Cargo",
        "Car go beep.",
        &["kids"],
    ),
    (
        "Dwayne",
        "Dwayne the bathtub, I'm dwowning!",
        &["kids", "oldie"],
    ),
];


type JokeMap = HashMap<JokeId, Joke>;

#[derive(Debug)]
pub struct JokeBase {
    _file: File,
    jokemap: JokeMap,
}

fn default_jokemap() -> JokeMap {
    JOKEBASE
        .iter()
        .enumerate()
        .map(|(i, (l1, l2, tags))| {
            (JokeId::new(i), Joke::new(i, l1, l2, tags))
        })
        .collect()
}

impl JokeBase {
    pub fn new<P: AsRef<std::path::Path>>(db_path: P) -> Result<Self, std::io::Error> {
        let mut file = File::create_new(&db_path)
            .and_then(|mut f| {
                let jokemap = default_jokemap();
                let json = serde_json::to_string(&jokemap).unwrap();
                f.write_all(json.as_bytes())?;
                f.sync_all()?;
                Ok(f)
            })
            .or_else(|e| {
                if e.kind() == ErrorKind::AlreadyExists {
                    File::options().read(true).write(true).open(&db_path)
                } else {
                    Err(e)
                }
            })?;
        file.rewind()?;
        let json = std::io::read_to_string(&mut file)?;
        let jokemap = serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))?;
        Ok(Self { _file: file, jokemap })
    }
}

impl IntoResponse for &JokeBase {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(&self.jokemap)).into_response()
    }
}
