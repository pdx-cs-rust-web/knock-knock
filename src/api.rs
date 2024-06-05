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
        update_joke,
        register,
    ),
    components(
        schemas(Joke, JokeBaseError, AuthError)
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
pub async fn jokes(State(appstate): HandlerAppState) -> Response {
    let jokes = appstate.read().await.jokebase.get_jokes().await;
    (StatusCode::OK, Json(jokes)).into_response()
}

#[utoipa::path(
    get,
    path = "/api/v1/joke",
    responses(
        (status = 200, description = "Return random joke", body = Joke),
        (status = 204, description = "Jokebase is empty", body = JokeBaseError)
    )
)]
pub async fn joke(State(appstate): HandlerAppState) -> Response {
    match appstate.read().await.jokebase.get_random().await {
        Ok(joke) => joke.into_response(),
        Err(e) => JokeBaseError::response(StatusCode::NO_CONTENT, e),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/joke/{id}",
    responses(
        (status = 200, description = "Return specified joke", body = Joke),
        (status = 204, description = "No joke with this id", body = JokeBaseError),
    )
)]
pub async fn get_joke(State(appstate): HandlerAppState, Path(joke_id): Path<String>) -> Response {
    match appstate.read().await.jokebase.get(&joke_id).await {
        Ok(joke) => joke.into_response(),
        Err(e) => JokeBaseError::response(StatusCode::NO_CONTENT, e),
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
        (status = 201, description = "Added joke", body = ()),
        (status = 400, description = "Bad request", body = JokeBaseError)
    )
)]
pub async fn post_joke(
    _claims: Claims,
    State(appstate): HandlerAppState,
    Json(joke): Json<Joke>,
) -> Response {
    match appstate.write().await.jokebase.add(joke).await {
        Ok(()) => StatusCode::CREATED.into_response(),
        Err(e) => JokeBaseError::response(StatusCode::BAD_REQUEST, e),
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/joke/{id}",
    responses(
        (status = 200, description = "Deleted joke", body = ()),
        (status = 400, description = "Bad request", body = JokeBaseError),
    )
)]
pub async fn delete_joke(
    _claims: Claims,
    State(appstate): HandlerAppState,
    Path(joke_id): Path<String>,
) -> Response {
    match appstate.write().await.jokebase.delete(&joke_id).await {
        Ok(()) => StatusCode::OK.into_response(),
        Err(e) => JokeBaseError::response(StatusCode::BAD_REQUEST, e),
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/joke/{id}",
    request_body(
        content = inline(Joke),
        description = "Joke to update"
    ),
    responses(
        (status = 200, description = "Updated joke", body = ()),
        (status = 400, description = "Bad request", body = JokeBaseError),
        (status = 404, description = "Joke not found", body = JokeBaseError),
        (status = 422, description = "Unprocessable entity", body = JokeBaseError),
    )
)]
pub async fn update_joke(
    _claims: Claims,
    State(appstate): HandlerAppState,
    Path(joke_id): Path<String>,
    Json(joke): Json<Joke>,
) -> Response {
    match appstate.write().await.jokebase.update(&joke_id, joke).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(JokeBaseErr::JokeUnprocessable(e)) => JokeBaseError::response(
            StatusCode::UNPROCESSABLE_ENTITY,
            JokeBaseErr::JokeUnprocessable(e),
        ),
        Err(JokeBaseErr::NoJoke) => {
            JokeBaseError::response(StatusCode::NOT_FOUND, JokeBaseErr::NoJoke)
        }
        Err(e) => JokeBaseError::response(StatusCode::BAD_REQUEST, e),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/register",
    request_body(
        content = inline(Registration),
        description = "Get an API key"
    ),
    responses(
        (status = 200, description = "JSON Web Token", body = AuthBody),
        (status = 401, description = "Registration failed", body = AuthError),
    )
)]
pub async fn register(
    State(appstate): HandlerAppState,
    Json(registration): Json<Registration>,
) -> Response {
    let appstate = appstate.read().await;
    match make_jwt_token(&appstate, &registration) {
        Err(e) => e.into_response(),
        Ok(token) => (StatusCode::OK, token).into_response(),
    }
}
