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

pub struct JokeBase(HashMap<JokeId, Joke>);

impl JokeBase {
    pub fn new<P: AsRef<std::path::Path>>(_dbpath: P) -> Self {
        let jokebase = JOKEBASE
            .iter()
            .enumerate()
            .map(|(i, (l1, l2, tags))| {
                (JokeId::new(i), Joke::new(i, l1, l2, tags))
            })
            .collect();
        Self(jokebase)
    }
}

impl IntoResponse for &JokeBase {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(&self.0)).into_response()
    }
}
