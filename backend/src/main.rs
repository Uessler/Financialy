mod auth;
mod categories;
mod error;
mod models;
mod reports;
mod transactions;
use axum::{
    Router,
    http::{HeaderValue, Method},
    routing::{get, post, put},
};
use sqlx::PgPool;
use std::{env, net::SocketAddr};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

#[derive(Clone)]
pub struct AppState {
    db: PgPool,
    jwt_secret: String,
    google_client_id: String,
    cookie_secure: bool,
    http: reqwest::Client,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "financialy_api=debug,tower_http=info".into()),
        )
        .init();
    let db = PgPool::connect(&env::var("DATABASE_URL")?).await?;
    sqlx::migrate!().run(&db).await?;
    let frontend = env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".into());
    let state = AppState {
        db,
        jwt_secret: env::var("JWT_SECRET")?,
        google_client_id: env::var("GOOGLE_CLIENT_ID")?,
        cookie_secure: env::var("COOKIE_SECURE").unwrap_or_default() == "true",
        http: reqwest::Client::new(),
    };
    let api = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/auth/google", post(auth::google_login))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/me", get(auth::me))
        .route(
            "/categories",
            get(categories::list).post(categories::create),
        )
        .route(
            "/categories/{id}",
            put(categories::update).delete(categories::delete),
        )
        .route(
            "/transactions",
            get(transactions::list).post(transactions::create),
        )
        .route(
            "/transactions/{id}",
            put(transactions::update).delete(transactions::delete),
        )
        .route("/dashboard", get(reports::dashboard));
    let app = Router::new()
        .nest("/api", api)
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(frontend.parse::<HeaderValue>()?)
                .allow_credentials(true)
                .allow_headers([axum::http::header::CONTENT_TYPE])
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE]),
        )
        .with_state(state);
    let port = env::var("API_PORT")
        .unwrap_or_else(|_| "3000".into())
        .parse()?;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!(%addr,"API ready");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
