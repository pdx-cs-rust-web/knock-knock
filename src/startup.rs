use crate::*;

async fn handler_404() -> Response {
    (StatusCode::NOT_FOUND, "404 Not Found").into_response()
}

pub const SESSION_ERROR_KEY: &str = "session_error";

trait ExtUnwrapOrFail<T> {
    fn unwrap_or_fail(self, msg: &str) -> T;
}

impl<T, E: std::fmt::Display> ExtUnwrapOrFail<T> for Result<T, E> {
    fn unwrap_or_fail(self, msg: &str) -> T {
        match self {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("{}: {}", msg, e);
                std::process::exit(1);
            }
        }
    }
}

pub async fn startup(ip: String) {
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

    let jokebase = JokeBase::new().await.unwrap_or_fail("jokebase");

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnSessionEnd);
    let auth_client = auth_client().unwrap_or_fail("auth_client");
    let state = Arc::new(RwLock::new(AppState::new(jokebase, auth_client)));
    let backend = AuthBackend::new(&state);
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    let mime_type = core::str::FromStr::from_str("image/vnd.microsoft.icon").unwrap();
    let favicon = services::ServeFile::new_with_mime("assets/static/favicon.ico", &mime_type);

    let mime_type = core::str::FromStr::from_str("text/css").unwrap();
    let stylesheet = services::ServeFile::new_with_mime(STYLESHEET, &mime_type);

    let apis = Router::new()
        .route("/jokes", get(jokes))
        .route("/joke", get(joke))
        .route("/joke/:id", get(get_joke))
        .route("/joke/add", post(post_joke))
        .route("/joke/:id", delete(delete_joke))
        .route("/joke/:id", put(update_joke));

    let swagger_ui = SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi());
    let redoc_ui = Redoc::with_url("/redoc", ApiDoc::openapi());
    let rapidoc_ui = RapiDoc::new("/api-docs/openapi.json").path("/rapidoc");

    let app = Router::new()
        .route("/", get(handler_index))
        .route("/index.html", get(handler_index))
        .route("/tell", get(handler_tell))
        .route("/add", get(handler_add))
        .route_service("/knock-knock.css", stylesheet)
        .route_service("/favicon.ico", favicon)
        .merge(swagger_ui)
        .merge(redoc_ui)
        .merge(rapidoc_ui)
        .nest("/api/v1", apis)
        .fallback(handler_404)
        .layer(auth_layer)
        .layer(trace_layer)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(ip).await.unwrap();
    tracing::debug!("serving {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
