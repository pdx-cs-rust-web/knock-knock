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

pub async fn make_jwt_keys() -> Result<JwtKeys, Box<dyn Error>> {
    use std::env::var;

    let secretf = var("JWT_SECRETFILE")?;
    let secret = tokio::fs::read_to_string(secretf).await?;
    Ok(JwtKeys::new(secret.trim().as_bytes()));
}

async fn public() -> &'static str {
    // A public endpoint that anyone can access
    "Welcome to the public area :)"
}

async fn private(claims: Claims) -> Result<String, AuthError> {
    // Send the protected data to the user
    Ok(format!(
        "Welcome to the protected area :)\nYour data:\n{claims}",
    ))
}

async fn login(Json(payload): Json<AuthPayload>) -> Result<Json<AuthBody>, AuthError> {
    // Check if the user sent the credentials
    if payload.client_id.is_empty() || payload.client_secret.is_empty() {
        return Err(AuthError::MissingCredentials);
    }
    // Here you can check the user credentials from a database
    if payload.client_id != "foo" || payload.client_secret != "bar" {
        return Err(AuthError::WrongCredentials);
    }

    // add 5 minutes to current unix epoch time as expiry date/time
    let exp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 300;

    let claims = Claims {
        sub: "b@b.com".to_owned(),
        company: "ACME".to_owned(),
        // Mandatory expiry time as UTC timestamp - takes unix epoch
        exp: usize::try_from(exp).unwrap(),
    };
    // Create the authorization token
    let token = encode(&Header::default(), &claims, &KEYS.encoding)
        .map_err(|_| AuthError::TokenCreation)?;

    // Send the authorized token
    Ok(Json(AuthBody::new(token)))
}

// allow us to print the claim details for the private route
impl Display for Claims {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Email: {}\nCompany: {}", self.sub, self.company)
    }
}

// implement a method to create a response type containing the JWT
impl AuthBody {
    fn new(access_token: String) -> Self {
        Self {
            access_token,
            token_type: "Bearer".to_string(),
        }
    }
}

// implement FromRequestParts for Claims (the JWT struct)
// FromRequestParts allows us to use Claims without consuming the request
#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;
        // Decode the user data
        let token_data = decode::<Claims>(bearer.token(), &KEYS.decoding, &Validation::default())
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }
}

// implement IntoResponse for AuthError so we can use it as an Axum response type
impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

// the JWT claim
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    company: String,
    exp: usize,
}

// the response that we pass back to HTTP client once successfully authorised
#[derive(Debug, Serialize)]
struct AuthBody {
    access_token: String,
    token_type: String,
}

// the request type - "client_id" is analogous to a username, client_secret can also be interpreted as a password
#[derive(Debug, Deserialize)]
struct AuthPayload {
    client_id: String,
    client_secret: String,
}

// error types for auth errors
#[derive(Debug)]
enum AuthError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
}
