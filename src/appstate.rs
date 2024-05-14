use crate::*;

pub struct AppState {
    pub jokebase: JokeBase,
    pub error: Option<String>,
}

pub type HandlerAppState = State<Arc<RwLock<AppState>>>;

impl AppState {
    pub fn new(jokebase: JokeBase) -> Self {
        Self {
            jokebase,
            error: None,
        }
    }
}
