use crate::*;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    joke: &'a Joke,
    stylesheet: &'static str,
}

impl<'a> IndexTemplate<'a> {
    fn new(joke: &'a Joke) -> Self {
        Self {
            joke,
            stylesheet: "/knock-knock.css",
        }
    }
}

pub async fn handler_index(State(jokebase): State<Arc<RwLock<JokeBase>>>) -> Response {
    match jokebase.read().await.get_random() {
        Some(joke) => (StatusCode::OK, IndexTemplate::new(joke)).into_response(),
        None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
    }
}
