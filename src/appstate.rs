use crate::*;

pub struct AppState {
    pub jokebase: JokeBase,
    pub auth_client: BasicClient,
}

pub type SharedAppState = Arc<RwLock<AppState>>;
pub type HandlerAppState = State<SharedAppState>;

impl AppState {
    pub fn new(jokebase: JokeBase, auth_client: BasicClient) -> Self {
        Self {
            jokebase,
            auth_client,
        }
    }
}
