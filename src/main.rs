mod joke;

use joke::*;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
extern crate tokio;
use tower_http::trace;
extern crate tracing;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

type JokeBase = HashMap<JokeId, Joke>;

async fn jokes(State(jokebase): State<Arc<JokeBase>>) -> Response {
    (StatusCode::OK, Json(&*jokebase)).into_response()
}

async fn handler_404() -> Response {
    (StatusCode::NOT_FOUND, "404 Not Found").into_response()
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "knock_knock=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    // https://carlosmv.hashnode.dev/adding-logging-and-tracing-to-an-axum-app-rust
    let trace_layer = trace::TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().level(tracing::Level::INFO))
        .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO));

    let jokes_vec = vec![
        Joke::new(
            1,
            "Boo",
            "You don't have to cry about it!",
            &["kids", "oldie"],
        ),
    ];

    let jokebase: JokeBase = jokes_vec
        .into_iter()
        .map(|j| (j.id.clone(), j))
        .collect();

    let app = Router::new()
        .route("/jokes", get(jokes))
        .fallback(handler_404)
        .layer(trace_layer)
        .with_state(Arc::new(jokebase));

    let ip = SocketAddr::new([127, 0, 0, 1].into(), 3000);
    let listener = tokio::net::TcpListener::bind(ip).await.unwrap();
    tracing::debug!("serving {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
