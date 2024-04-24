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
