use crate::*;

pub struct AppState {
    pub jokebase: JokeBase,
    pub auth_client: BasicClient,
}

pub type HandlerAppState = State<Arc<RwLock<AppState>>>;

impl AppState {
    pub fn new(jokebase: JokeBase, auth_client: BasicClient) -> Self {
        Self {
            jokebase,
            auth_client,
        }
    }
}
