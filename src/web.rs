use crate::*;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    joke: Joke,
}

pub async fn handler_index(State(jokebase): State<Arc<RwLock<JokeBase>>>) -> Response {
    match jokebase.read().await.get_random() {
        Some(joke) => (StatusCode::OK, IndexTemplate{joke}).into_response(),
        None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
    }
}
