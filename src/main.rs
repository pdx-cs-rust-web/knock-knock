mod api;
mod joke;
mod jokebase;

use api::*;
use joke::*;
use jokebase::*;

use std::fs::File;
use std::io::{ErrorKind, Seek, Write};
use std::net::SocketAddr;
use std::sync::Arc;
use std::collections::{HashMap, HashSet};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
extern crate fastrand;
use serde::{Serialize, Deserialize};
extern crate serde_json;
extern crate thiserror;
use tokio::{self, sync::RwLock};
use tower_http::trace;
extern crate tracing;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::{OpenApi, ToSchema};
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

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
    
    let jokebase = JokeBase::new("assets/jokebase.json")
        .unwrap_or_else(|e| {
            tracing::error!("jokebase new: {}", e);
            std::process::exit(1);
        });
    let jokebase = Arc::new(RwLock::new(jokebase));
    
    let apis = Router::new()
        .route("/jokes", get(jokes))
        .route("/joke", get(joke))
        .route("/joke/:id", get(get_joke))
        .route("/joke/add", post(post_joke));

    let swagger_ui = SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", ApiDoc::openapi());
    let redoc_ui = Redoc::with_url("/redoc", ApiDoc::openapi());
    let rapidoc_ui = RapiDoc::new("/api-docs/openapi.json").path("/rapidoc");
        
    let app = Router::new()
        .merge(swagger_ui)
        .merge(redoc_ui)
        .merge(rapidoc_ui)
        .nest("/api/v1", apis)
        .fallback(handler_404)
        .layer(trace_layer)
        .with_state(jokebase);

    let ip = SocketAddr::new([127, 0, 0, 1].into(), 3000);
    let listener = tokio::net::TcpListener::bind(ip).await.unwrap();
    tracing::debug!("serving {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
