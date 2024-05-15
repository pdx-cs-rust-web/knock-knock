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
    let base = authinfo
        .get("web")
        .ok_or(PathError::Component("web"))?;
    let get_component = |field| -> Result<String, Box<dyn Error>> {
        let mut component = base
            .get(field)
            .ok_or(PathError::Component(field))?;
        if let Some(list) = component.as_array() {
            component = list.first().ok_or(PathError::List(field))?;
        }
        let component = component
            .as_str()
            .ok_or(PathError::Leaf(field))?
            .into();
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

    let client = BasicClient::new(
        client_id,
        Some(client_secret),
        auth_uri,
        Some(token_uri),
    )
    .set_redirect_uri(redirect_uri)
    .set_revocation_uri(revocation_uri);

    Ok(client)
}
