mod joke;

use joke::*;

use std::net::SocketAddr;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};

async fn jokes() -> Response {
    let jokebase = vec![Joke::new(
        1,
        "Boo",
        "You don't have to cry about it!",
        &["kids", "oldie"],
    )];
    (StatusCode::OK, Json(jokebase)).into_response()
}

async fn handler_404() -> Response {
    (StatusCode::NOT_FOUND, "404 Not Found").into_response()
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/jokes", get(jokes)).fallback(handler_404);
    let ip = SocketAddr::new([127, 0, 0, 1].into(), 3000);
    eprintln!("knock-knock: serving {}", ip);
    let listener = tokio::net::TcpListener::bind(ip).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
