// From https://github.com/shuttle-hq/shuttle-examples/axum/jwt-authentication

use crate::*;

pub struct JwtKeys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl JwtKeys {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

pub async fn read_secret(env_var: &str) -> Result<String, Box<dyn Error>> {
    let secretf = std::env::var(env_var)?;
    let secret = tokio::fs::read_to_string(secretf).await?;
    Ok(secret.trim().to_string())
}

pub async fn make_jwt_keys() -> Result<JwtKeys, Box<dyn Error>> {
    let secret = read_secret("JWT_SECRETFILE").await?;
    Ok(JwtKeys::new(secret.as_bytes()))
}

#[derive(Debug, thiserror::Error, Serialize)]
pub enum AuthError {
    #[error("wrong credentials")]
    WrongCredentials,
    #[error("missing credentials")]
    MissingCredentials,
    #[error("token creation")]
    TokenCreation,
    #[error("invalid token")]
    InvalidToken,
}

impl<'s> ToSchema<'s> for AuthError {
    fn schema() -> (&'s str, RefOr<Schema>) {
        let example = serde_json::json!({
            "status":"401","error":"wrong credentials"
        });
        error_schema("AuthError", example)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    full_name: String,
    email: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthBody {
    access_token: String,
    token_type: String,
}

impl AuthBody {
    fn new(access_token: String) -> Self {
        Self {
            access_token,
            token_type: "Bearer".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AuthPayload {
    client_id: String,
    client_secret: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/login",
    responses(
        (status = 200, description = "login ok", body = AuthBody),
        (status = 400, description = "missing credentials", body = AuthError),
        (status = 401, description = "wrong credentials", body = AuthError),
        (status = 400, description = "invalid token", body = AuthError),
        (status = 500, description = "token creation error", body = AuthError),
    )
)]
pub async fn login(
    State(state): HandlerAppState,
    Json(payload): Json<AuthPayload>,
) -> Response {
    if payload.client_id.is_empty() || payload.client_secret.is_empty() {
        return AuthError::MissingCredentials.into_response();
    }

    #[derive(sqlx::FromRow)]
    struct PwUser {
        client_id: String,
        client_secret: String,
        full_name: String,
        email: String,
    }

    let user: Result<PwUser, sqlx::Error> = sqlx::query_as(r#"SELECT * FROM passwords WHERE client_id = $1"#)
        .bind(&payload.client_id)
        .fetch_one(&state.read().await.jokebase.0)
        .await;
    let user = match user {
        Ok(user) => user,
        Err(_) => return AuthError::WrongCredentials.into_response(),
    };

    if payload.client_id != user.client_id || payload.client_secret != user.client_secret {
        return AuthError::WrongCredentials.into_response();
    }

    let claims = Claims {
        full_name: user.full_name,
        email: user.email,
    };

    let token = match encode(&Header::default(), &claims, &state.read().await.jwt_keys.encoding) {
        Ok(token) => token,
        Err(_) => return AuthError::TokenCreation.into_response(),
    };

    Json(AuthBody::new(token)).into_response()
}

#[async_trait]
impl FromRequestParts<SharedAppState> for Claims {
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &SharedAppState) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;
        // Decode the user data
        let token_data = decode::<Claims>(bearer.token(), &state.read().await.jwt_keys.decoding, &Validation::default())
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
        };
        let body = Json(serde_json::json!({
            "status": status.as_u16(),
            "error": error_message,
        }));
        (status, body).into_response()
    }
}
