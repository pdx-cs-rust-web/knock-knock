use crate::*;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    joke: Option<&'a Joke>,
    tags: Option<String>,
    stylesheet: &'static str,
    error: Option<String>,
}

impl<'a> IndexTemplate<'a> {
    fn joke(joke: &'a Joke) -> Self {
        Self {
            joke: Some(joke),
            tags: joke.tags.as_ref().map(format_tags),
            stylesheet: "/knock-knock.css",
            error: None,
        }
    }

    fn error(error: String) -> Self {
        Self {
            joke: None,
            tags: None,
            stylesheet: "/knock-knock.css",
            error: Some(error),
        }
    }
}

#[derive(Deserialize)]
pub struct IndexParams {
    id: Option<String>,
}

pub async fn handler_index(
    State(appstate): HandlerAppState,
    Query(params): Query<IndexParams>,
) -> Response {
    let appstate = appstate.read().await;
    let jokebase = &appstate.jokebase;

    let joke = if let Some(id) = params.id {
        jokebase.get(&id).await
    } else {
        match jokebase.get_random().await {
            Ok(joke) => return Redirect::to(&format!("/?id={}", joke.id)).into_response(),
            e => e,
        }
    };

    match joke {
        Ok(joke) => (StatusCode::OK, IndexTemplate::joke(&joke)).into_response(),
        Err(JokeBaseErr::JokeDoesNotExist(id)) => (
            StatusCode::OK,
            IndexTemplate::error(format!("cannot find joke {}", id)),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            IndexTemplate::error(e.to_string()),
        )
            .into_response(),
    }
}

#[derive(Template)]
#[template(path = "tell.html")]
pub struct TellTemplate {
    stylesheet: &'static str,
    error: Option<String>,
}

impl TellTemplate {
    fn new(error: Option<String>) -> Self {
        Self {
            stylesheet: "/knock-knock.css",
            error,
        }
    }
}

pub async fn handler_tell(session: Session) -> Response {
    let error: Option<String> = session.get(SESSION_ERROR_KEY).await.unwrap_or(None).clone();
    let _ = session.remove::<Option<String>>(SESSION_ERROR_KEY).await;
    (StatusCode::OK, TellTemplate::new(error)).into_response()
}

#[derive(Deserialize)]
pub struct AddParams {
    id: String,
    who: String,
    answer: String,
    tags: Option<String>,
    source: Option<String>,
}

fn parse_tags(tags: Option<String>) -> Option<HashSet<String>> {
    let tags = tags?;
    if tags.is_empty() {
        return None;
    }
    let tags: HashSet<String> = tags.split(',').map(str::trim).map(str::to_string).collect();
    if tags.is_empty() {
        None
    } else {
        Some(tags)
    }
}

fn parse_source(source: Option<String>) -> Option<String> {
    let source = source?;
    if source.is_empty() {
        None
    } else {
        Some(source)
    }
}

pub async fn handler_add(
    State(appstate): HandlerAppState,
    Query(params): Query<AddParams>,
    session: Session,
) -> Response {
    // XXX Condition user input.
    let joke = Joke {
        id: params.id.clone(),
        whos_there: params.who,
        answer_who: params.answer,
        tags: parse_tags(params.tags),
        source: parse_source(params.source),
    };

    let mut appstate = appstate.write().await;

    match appstate.jokebase.add(joke).await {
        Ok(()) => Redirect::to(&format!("/?id={}", params.id)).into_response(),
        Err(JokeBaseErr::JokeBaseIoError(msg)) => {
            (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
        }
        Err(JokeBaseErr::JokeExists(id)) => {
            let error = Some(format!("joke {} already exists", id));
            let _ = session.insert(SESSION_ERROR_KEY, error).await;
            Redirect::to("/tell").into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
