use crate::*;

pub struct AppState {
    pub jokebase: JokeBase,
    pub jwt_keys: JwtKeys,
}

pub type SharedAppState = Arc<RwLock<AppState>>;

pub type HandlerAppState = State<SharedAppState>;

impl AppState {
    pub fn new(jokebase: JokeBase, jwt_keys: JwtKeys) -> Self {
        Self {
            jokebase,
            jwt_keys,
        }
    }
}
