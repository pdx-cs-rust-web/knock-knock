use crate::*;

pub struct AppState {
    pub jokebase: JokeBase,
    pub jwt_keys: JwtKeys,
    pub reg_key: String,
}

pub type SharedAppState = Arc<RwLock<AppState>>;

pub type HandlerAppState = State<SharedAppState>;

impl AppState {
    pub fn new(jokebase: JokeBase, jwt_keys: JwtKeys, reg_key: String) -> Self {
        Self {
            jokebase,
            jwt_keys,
            reg_key,
        }
    }
}
