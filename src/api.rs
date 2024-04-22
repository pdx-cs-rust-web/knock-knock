use crate::*;

// From utoipa/examples/{simple-axum, axum-todo}.

#[derive(OpenApi)]
#[openapi(
    paths(
        jokes,
        joke,
        get_joke,
        post_joke,
        delete_joke,
    ),
    components(
        schemas(Joke, JokeBaseError)
    ),
    tags(
        (name = "knock-knock", description = "Knock-Knock Joke API")
    )
)]
pub struct ApiDoc;

#[utoipa::path(
    get,
    path = "/api/v1/jokes",
    responses(
        (status = 200, description = "List jokes", body = [Joke])
    )
)]
pub async fn jokes(State(jokebase): State<Arc<RwLock<JokeBase>>>) -> Response {
    jokebase.read().await.into_response()
}

#[utoipa::path(
    get,
    path = "/api/v1/joke",
    responses(
        (status = 200, description = "Return random joke", body = Joke),
        (status = 404, description = "Jokebase is empty", body = JokeBaseError)
    )
)]
pub async fn joke(
    State(jokebase): State<Arc<RwLock<JokeBase>>>,
) -> Response {
    match jokebase.read().await.get_random() {
        Some(joke) => joke.into_response(),
        None => JokeBaseError::response(StatusCode::NOT_FOUND, JokeBaseErr::NoJoke),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/joke/{id}",
    responses(
        (status = 200, description = "Return specified joke", body = Joke),
        (status = 404, description = "No joke with this id", body = JokeBaseError),
    )
)]
pub async fn get_joke(
    State(jokebase): State<Arc<RwLock<JokeBase>>>,
    Path(joke_id): Path<String>,
) -> Response {
    match jokebase.read().await.get(&joke_id) {
        Some(joke) => joke.into_response(),
        None => JokeBaseError::response(StatusCode::NOT_FOUND, JokeBaseErr::NoJoke),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/joke/add",
    request_body(
        content = inline(Joke),
        description = "Joke to add"
    ),
    responses(
        (status = 200, description = "Added joke", body = ()),
        (status = 400, description = "Joke add failed", body = JokeBaseError)
    )
)]
pub async fn post_joke(
    State(jokebase): State<Arc<RwLock<JokeBase>>>,
    Json(joke): Json<Joke>,
) -> Response {
    match jokebase.write().await.add(joke) {
        Ok(()) => StatusCode::CREATED.into_response(),
        Err(e) => JokeBaseError::response(StatusCode::BAD_REQUEST, e),
    }
}

#[utoipa::path(
    delete,
    path = "/joke/:id",
    responses(
        (status = 200, description = "Deleted joke", body = ()),
        (status = 400, description = "Joke delete fail", body = ()),
    )
)]
pub async fn delete_joke(
    State(jokebase): State<Arc<RwLock<JokeBase>>>,
    Path(joke_id): Path<String>,
) -> Response {
    match jokebase.write().await.delete(joke_id) {
        Ok(()) => StatusCode::CREATED.into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}
