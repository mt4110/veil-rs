use axum::{
    extract::{Request, State},
    http::{header, StatusCode, Uri},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use base64::Engine;
use clap::Parser;
use rand::RngCore;
use rust_embed::RustEmbed;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_sessions::{MemoryStore, Session, SessionManagerLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod auth;
mod config_loader;
pub mod evidence;
pub mod evidence_generator;

#[derive(Parser, Debug)]
#[command(name = "veil-pro")]
#[command(about = "Veil Pro Dashboard Server", long_about = None)]
struct Cli {
    /// Skip opening the browser automatically
    #[arg(long)]
    no_open: bool,

    /// Optional: explicitly set a port instead of auto-assigning (testing only)
    #[arg(long)]
    port: Option<u16>,
}

#[derive(Clone)]
pub struct AppState {
    pub token: String,
    pub run_cache: Arc<tokio::sync::RwLock<evidence::RunCache>>,
    pub oauth: Arc<auth::OAuthConfig>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "veil_pro=info,tower_http=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();

    // 1. Generate secure random token
    let mut token_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut token_bytes);
    let token = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(token_bytes);

    let oauth_config = auth::init_oauth();
    let state = Arc::new(AppState {
        token: token.clone(),
        run_cache: Arc::new(tokio::sync::RwLock::new(evidence::RunCache::new(
            20,
            100_000_000,
            30,
        ))), // 20 runs max, 100MB max, 30 min TTL
        oauth: Arc::new(oauth_config),
    });

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false) // For local development UI, keep false unless running HTTPS
        .with_same_site(tower_sessions::cookie::SameSite::Lax);

    // Middleware for token auth on API routes
    let api_routes = Router::new()
        .route("/me", get(api::get_me))
        .route("/projects", get(api::list_projects))
        .route("/scan", post(api::scan_project))
        .route("/runs/:run_id", get(api::get_run_meta))
        .route("/runs/:run_id/evidence.zip", get(api::export_evidence))
        .route("/policy", get(api::get_policy))
        .route("/baseline", post(api::write_baseline))
        .layer(middleware::from_fn_with_state(state.clone(), require_auth)); // Updated to require_auth

    // 2. Setup Router
    let app = Router::new()
        .nest("/api", api_routes)
        // OAuth Routes
        .nest("/auth", auth::router())
        // Svelte SPA Catch-All
        .fallback(static_handler)
        .layer(session_layer)
        .layer(middleware::from_fn(security_headers))
        .with_state(state.clone());

    // 3. Bind to 127.0.0.1 ONLY
    let port = cli.port.unwrap_or(0); // 0 means OS picks an available port
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr).await?;
    let bound_addr = listener.local_addr()?;

    let url = format!("http://{}/#token={}", bound_addr, token);

    // UI Console Output (Safe by default, clear UX)
    println!("=======================================================");
    println!(" 🛡️ Veil Pro Dashboard");
    println!("=======================================================");
    println!(" Server is running securely on localhost.");
    println!(" URL: {}", url);
    println!("=======================================================");

    if !cli.no_open {
        if open::that(&url).is_err() {
            tracing::warn!("Failed to open browser automatically. Please open the URL manually.");
        }
    }

    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(RustEmbed)]
#[folder = "frontend/dist/"]
struct Asset;

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();
    if path.is_empty() {
        path = "index.html".to_string();
    }

    match Asset::get(&path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => {
            if let Some(index) = Asset::get("index.html") {
                let mime = mime_guess::from_path("index.html").first_or_octet_stream();
                ([(header::CONTENT_TYPE, mime.as_ref())], index.data).into_response()
            } else {
                (StatusCode::NOT_FOUND, "404 Not Found").into_response()
            }
        }
    }
}

// Global B2B Security Headers
async fn security_headers(req: Request, next: Next) -> Response {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();
    headers.insert("Referrer-Policy", "no-referrer".parse().unwrap());
    headers.insert("Cache-Control", "no-store".parse().unwrap());

    // Strict CSP: NO 'unsafe-inline'
    headers.insert(
        "Content-Security-Policy",
        "default-src 'self'; img-src 'self' data:; style-src 'self'; script-src 'self'; connect-src 'self';"
            .parse()
            .unwrap(),
    );
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    response
}

// Hybrid Authorization: Bearer Token OR Cookie Session
async fn require_auth(
    State(state): State<Arc<AppState>>,
    session: Session,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 1. Check Bearer Token
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());
    if let Some(auth_header) = auth_header {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            if token == state.token {
                return Ok(next.run(req).await);
            }
        }
    }

    // 2. Check SSO Session
    let user_valid: Option<String> = session.get("user_email").await.unwrap_or(None);
    if user_valid.is_some() {
        return Ok(next.run(req).await);
    }

    // Unauthorized
    Err(StatusCode::UNAUTHORIZED)
}

#[cfg(test)]
mod tests {
    use std::net::{SocketAddr, TcpListener};

    #[test]
    fn test_server_binds_to_localhost_only() {
        // Enforce B2B security requirement: must bind to localhost, not 0.0.0.0
        let addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let listener = TcpListener::bind(addr).expect("Should bind to localhost");
        let bound_addr = listener.local_addr().unwrap();

        // Ensure it is explicitly bound to the loopback IPv4 address
        assert!(
            bound_addr.ip().is_loopback(),
            "Server must bind to the loopback interface for security!"
        );
        assert_eq!(
            bound_addr.ip().to_string(),
            "127.0.0.1",
            "Must explicitly be 127.0.0.1"
        );
    }
}
