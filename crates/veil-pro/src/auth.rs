use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
    routing::get,
    Router,
};
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use tower_sessions::Session;

use crate::AppState;

#[derive(Clone)]
pub struct OAuthConfig {
    pub google_client: Option<BasicClient>,
    pub apple_client: Option<BasicClient>,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/google/login", get(google_login))
        .route("/google/callback", get(google_callback))
}

pub fn init_oauth() -> OAuthConfig {
    let google_client = if let (Ok(client_id), Ok(client_secret)) = (
        std::env::var("GOOGLE_CLIENT_ID"),
        std::env::var("GOOGLE_CLIENT_SECRET"),
    ) {
        Some(
            BasicClient::new(
                ClientId::new(client_id),
                Some(ClientSecret::new(client_secret)),
                AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string()).unwrap(),
                Some(TokenUrl::new("https://oauth2.googleapis.com/token".to_string()).unwrap()),
            )
            .set_redirect_uri(
                RedirectUrl::new("http://127.0.0.1:3000/auth/google/callback".to_string()).unwrap(),
            ),
        )
    } else {
        None
    };

    // Apple/iCloud setup would follow a similar pattern, omitted for brevity but scaffolded
    let apple_client = None;

    OAuthConfig {
        google_client,
        apple_client,
    }
}

async fn google_login(State(state): State<Arc<AppState>>, session: Session) -> impl IntoResponse {
    let client = state
        .oauth
        .google_client
        .as_ref()
        .expect("Google SSO not configured");
    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();

    session
        .insert("oauth_csrf", csrf_token.secret().to_string())
        .await
        .unwrap();
    Redirect::to(auth_url.as_ref())
}

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    code: String,
    state: String,
}

#[derive(Debug, Deserialize)]
struct GoogleUser {
    email: String,
    name: String,
    picture: String,
}

async fn google_callback(
    State(state): State<Arc<AppState>>,
    session: Session,
    Query(query): Query<AuthRequest>,
) -> impl IntoResponse {
    let client = state
        .oauth
        .google_client
        .as_ref()
        .expect("Google SSO not configured");
    let csrf_val: Option<String> = session.get("oauth_csrf").await.unwrap();

    if csrf_val.unwrap_or_default() != query.state {
        return Redirect::to("/?error=csrf_mismatch");
    }

    // Exchange token
    let token = match client
        .exchange_code(oauth2::AuthorizationCode::new(query.code))
        .request_async(oauth2::reqwest::async_http_client)
        .await
    {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("OAuth token exchange failed: {:?}", e);
            return Redirect::to("/?error=token_exchange_failed");
        }
    };

    // Fetch user profile
    let reqwest_client = Client::new();
    let res = reqwest_client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(token.access_token().secret())
        .send()
        .await;

    let user_info = match res {
        Ok(res) if res.status().is_success() => res.json::<GoogleUser>().await.unwrap(),
        _ => return Redirect::to("/?error=profile_fetch_failed"),
    };

    // Authenticate session
    session.insert("user_email", user_info.email).await.unwrap();
    session.insert("user_name", user_info.name).await.unwrap();
    session
        .insert("user_picture", user_info.picture)
        .await
        .unwrap();

    // In an SSO B2B environment, the session cookie is the source of identity.
    Redirect::to("/")
}
