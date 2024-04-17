use crate::*;

pub async fn jokes(State(jokebase): State<Arc<RwLock<JokeBase>>>) -> Response {
    jokebase.read().await.into_response()
}

pub async fn joke(
    State(jokebase): State<Arc<RwLock<JokeBase>>>,
) -> Response {
    match jokebase.read().await.get_random() {
        Some(joke) => joke.into_response(),
        None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
    }
}

pub async fn get_joke(
    State(jokebase): State<Arc<RwLock<JokeBase>>>,
    Path(joke_id): Path<JokeId>,
) -> Response {
    match jokebase.read().await.get(&joke_id) {
        Some(joke) => joke.into_response(),
        None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
    }
}

pub async fn post_joke(
    State(jokebase): State<Arc<RwLock<JokeBase>>>,
    Json(joke): Json<Joke>,
) -> Response {
    match jokebase.write().await.add(joke) {
        Ok(()) => StatusCode::CREATED.into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}
