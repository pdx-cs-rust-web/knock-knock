// From https://github.com/shuttle-hq/shuttle-examples/axum/jwt-authentication

use crate::*;

use jsonwebtoken::{EncodingKey, DecodingKey};

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
    #[error("invalid token")]
    InvalidToken,
    #[error("internal error: token creation")]
    TokenCreation,
    #[error("registration error")]
    Registration,
}

impl<'s> ToSchema<'s> for AuthError {
    fn schema() -> (&'s str, RefOr<Schema>) {
        let example = serde_json::json!({
            "status":"401","error":"wrong credentials"
        });
        error_schema("AuthError", example)
    }
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

impl IntoResponse for AuthBody {
    fn into_response(self) -> Response {
        Json(serde_json::json!(self)).into_response()
    }
}

/*
#[derive(Debug, Deserialize, ToSchema)]
pub struct AuthPayload {
    client_id: String,
    client_secret: String,
}
*/

#[async_trait]
impl FromRequestParts<SharedAppState> for Claims {
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &SharedAppState) -> Result<Self, Self::Rejection> {
        use jsonwebtoken::{Algorithm, Validation, decode};

        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;
        // Decode the user data
        let appstate = state.read().await;
        let decoding_key = &appstate.jwt_keys.decoding;
        let validation = Validation::new(Algorithm::HS512);
        let result = decode::<Claims>(
            bearer.token(),
            decoding_key,
            &validation,
        );
        let token_data = result.map_err(|_| AuthError::Registration)?;
        Ok(token_data.claims)
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::Registration => (StatusCode::UNAUTHORIZED, "Invalid registration"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
        };
        let body = Json(serde_json::json!({
            "status": status.as_u16(),
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct Registration {
    #[schema(example = "John Smith")]
    full_name: String,
    #[schema(example = "johnsmith@example.org")]
    email: String,
    #[schema(example = "password123")]
    password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Claims {
    #[schema(example = "knock-knock.po8.org")]
    iss: String,
    #[schema(example = "John Smith <johnsmith@example.org>")]
    sub: String,
    #[schema(example = "1717630066")]
    exp: u64,
}

pub fn make_jwt_token(appstate: &AppState, registration: &Registration) -> Result<AuthBody, AuthError> {
    use jsonwebtoken::{Algorithm, Header, encode};
    
    if registration.password != appstate.reg_key {
        return Err(AuthError::Registration);
    }

    let iss = "knock-knock.po8.org".to_string();
    let sub = format!("{} <{}>", registration.full_name, registration.email);
    let exp = (Utc::now() + TimeDelta::days(1)).timestamp();
    let exp = u64::try_from(exp).unwrap();
    let claims = Claims { iss, sub, exp };
    let header = Header::new(Algorithm::HS512);
    let token = encode(&header, &claims, &appstate.jwt_keys.encoding)
        .map_err(|_| AuthError::TokenCreation)?;
    Ok(AuthBody::new(token))
}
