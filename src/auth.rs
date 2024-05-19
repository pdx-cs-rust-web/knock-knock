// Most code here derived from
// https://github.com/maxcountryman/axum-login/examples/oauth2

use crate::*;

#[derive(Debug, thiserror::Error)]
enum PathError {
    #[error("path error: missing {0}")]
    Component(&'static str),
    #[error("path error: empty list {0}")]
    List(&'static str),
    #[error("path error: type {0}")]
    Leaf(&'static str),
}

pub fn auth_client() -> Result<BasicClient, Box<dyn Error>> {
    use std::env::var;

    let authf = var("GOOGLE_AUTHFILE")?;
    let authf = std::fs::File::open(authf)?;
    let authinfo: serde_json::Value = serde_json::from_reader(authf)?;
    // https://stackoverflow.com/a/56281271
    let base = authinfo.get("web").ok_or(PathError::Component("web"))?;
    let get_component = |field| -> Result<String, Box<dyn Error>> {
        let mut component = base.get(field).ok_or(PathError::Component(field))?;
        if let Some(list) = component.as_array() {
            component = list.first().ok_or(PathError::List(field))?;
        }
        let component = component.as_str().ok_or(PathError::Leaf(field))?.into();
        Ok(component)
    };
    let client_id = get_component("client_id")?;
    let client_secret = get_component("client_secret")?;
    let auth_uri = get_component("auth_uri")?;
    let token_uri = get_component("token_uri")?;
    let redirect_uri = get_component("redirect_uris")?;

    let client_id = ClientId::new(client_id);
    let client_secret = ClientSecret::new(client_secret);
    let token_uri = TokenUrl::new(token_uri)?;
    let auth_uri = AuthUrl::new(auth_uri)?;
    let redirect_uri = RedirectUrl::new(redirect_uri)?;
    let revocation_uri = RevocationUrl::new("https://oauth2.googleapis.com/revoke".into())?;

    let client = BasicClient::new(client_id, Some(client_secret), auth_uri, Some(token_uri))
        .set_redirect_uri(redirect_uri)
        .set_revocation_uri(revocation_uri);

    Ok(client)
}

#[derive(Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    account: String,
    access_token: String,
}

// Don't expose access token.
impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("account", &self.account)
            .field("access_token", &"[redacted]")
            .finish()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    code: String,
    old_state: CsrfToken,
    new_state: CsrfToken,
}

impl AuthUser for User {
    type Id = String;

    fn id(&self) -> Self::Id {
        self.account.clone()
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.access_token.as_bytes()
    }
}

#[derive(Clone)]
pub struct AuthBackend(SharedAppState);

impl AuthBackend {
    pub fn new(app_state: &SharedAppState) -> Self {
        Self(Arc::clone(app_state))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error(transparent)]
    Sqlx(sqlx::Error),

    #[error(transparent)]
    Reqwest(reqwest::Error),

    #[error(transparent)]
    OAuth2(BasicRequestTokenError<AsyncHttpClientError>),
}

#[derive(Debug, Deserialize)]
struct UserInfo {
    login: String,
}

impl AuthnBackend for AuthBackend {
    type User = User;
    type Credentials = Credentials;
    type Error = BackendError;

    fn authenticate<'a, 'b>(
        &'a self,
        creds: Self::Credentials,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Option<Self::User>, Self::Error>> + Send + 'b>,
    >
    where
        Self: 'b,
        'a: 'b,
    {
        Box::pin(async move {
            let auth_backend = self.0.read().await;

            // Ensure the CSRF state has not been tampered with.
            if creds.old_state.secret() != creds.new_state.secret() {
                return Ok(None);
            };

            // Process authorization code, expecting a token response back.
            let token_res = auth_backend
                .auth_client
                .exchange_code(AuthorizationCode::new(creds.code))
                .request_async(async_http_client)
                .await
                .map_err(Self::Error::OAuth2)?;

            // Use access token to request user info.
            let user_info = reqwest::Client::new()
                .get("https://api.github.com/user")
                .header(USER_AGENT, "axum-login")
                .header(
                    AUTHORIZATION,
                    format!("Bearer {}", token_res.access_token().secret()),
                )
                .send()
                .await
                .map_err(Self::Error::Reqwest)?
                .json::<UserInfo>()
                .await
                .map_err(Self::Error::Reqwest)?;

            // Persist user in our database so we can use `get_user`.
            let user = sqlx::query_as(
                r#"
            INSERT INTO users (username, access_token)
            VALUES ($1, $2)
            ON CONFLICT(username) DO UPDATE
            SET access_token = excluded.access_token
            RETURNING *
            "#,
            )
            .bind(user_info.login)
            .bind(token_res.access_token().secret())
            .fetch_one(&auth_backend.jokebase.0)
            .await
            .map_err(Self::Error::Sqlx)?;

            Ok(Some(user))
        })
    }

    fn get_user<'a, 'b, 'c>(
        &'a self,
        user_id: &'b UserId<Self>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Option<Self::User>, Self::Error>> + Send + 'c>,
    >
    where
        Self: 'c,
        'a: 'c,
        'b: 'c,
    {
        Box::pin(async move {
            sqlx::query_as("SELECT * FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_optional(&self.0.read().await.jokebase.0)
                .await
                .map_err(Self::Error::Sqlx)
        })
    }
}
