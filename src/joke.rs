use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct JokeId(String);

impl JokeId {
    pub fn new(id: &str) -> Self {
        Self(id.to_owned())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Joke {
    id: JokeId,
    whos_there: String,
    answer_who: String,
    tags: HashSet<String>,
}

impl Joke {
    pub fn new(id: &str, whos_there: &str, answer_who: &str, tags: &[&str]) -> Self {
        let id = JokeId(id.to_owned());
        let whos_there = whos_there.into();
        let answer_who = answer_who.into();
        let tags: HashSet<String> = tags.iter().copied().map(String::from).collect();
        Self {
            id,
            whos_there,
            answer_who,
            tags,
        }
    }
}

impl From<&Joke> for String {
    fn from(joke: &Joke) -> Self {
        let mut text: String = "Knock knock!\n".into();
        text += "Who's there?\n";
        text += &format!("{}.\n", joke.whos_there);
        text += &format!("\"{}\" who?\n", joke.whos_there);
        text += &format!("{}\n", joke.answer_who);
        text += "\n";
        let taglist: Vec<&str> = joke.tags.iter().map(String::as_ref).collect();
        let taglist = taglist.join(", ");
        text += &format!("[id: {}; tags: {}]\n", joke.id.0, taglist);
        text
    }
}

impl IntoResponse for &Joke {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(&self)).into_response()
    }
}
