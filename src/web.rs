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

#[derive(Deserialize)]
pub struct IndexParams {
    id: Option<String>,
}

pub async fn handler_index(
    State(jokebase): State<Arc<RwLock<JokeBase>>>,
    Query(params): Query<IndexParams>,
) -> Response {
    let jokebase = jokebase.read().await;

    let joke = if let Some(id) = params.id {
        jokebase.get(&id)
    } else {
        jokebase.get_random()
    };

    match joke {
        Ok(joke) => (StatusCode::OK, IndexTemplate::new(joke)).into_response(),
        Err(e) => (StatusCode::NO_CONTENT, e.to_string()).into_response(),
    }
}

#[derive(Template)]
#[template(path = "tell.html")]
pub struct TellTemplate {
    stylesheet: &'static str,
}

impl TellTemplate {
    fn new() -> Self {
        Self { stylesheet: "/knock-knock.css" }
    }
}

pub async fn handler_tell() -> Response {
    (StatusCode::OK, TellTemplate::new()).into_response()
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
    tags.map(|tags| {
        tags.split(',').map(str::trim).map(str::to_string).collect()
    })
}

pub async fn handler_add(
    State(jokebase): State<Arc<RwLock<JokeBase>>>,
    Query(params): Query<AddParams>,
) -> Response {
    // XXX Condition user input.
    let joke = Joke {
        id: params.id,
        whos_there: params.who,
        answer_who: params.answer,
        tags: parse_tags(params.tags),
        source: params.source,
    };

    let mut jokebase = jokebase.write().await;

    match jokebase.add(joke) {
        Ok(()) => (StatusCode::OK, "".to_string()).into_response(),
        Err(JokeBaseErr::JokeBaseIoError(msg)) =>
            (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response(),
        Err(JokeBaseErr::JokeExists(id)) =>
            (StatusCode::CONFLICT, id).into_response(),
        Err(e) =>
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
