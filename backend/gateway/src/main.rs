mod handler;
mod session;
mod hub;
mod events;

use axum::Router;
use axum::routing::get;
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;
use std::sync::Arc;

use crate::hub::Hub;

pub struct GatewayState {
    pub hub: Arc<Hub>,
    pub jwt_secret: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let jwt_secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET doit être défini");

    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let host = std::env::var("GATEWAY_HOST")
        .unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("GATEWAY_PORT")
        .unwrap_or_else(|_| "8081".to_string());

    // Connexion Redis pour la présence
    let redis_client = redis::Client::open(redis_url.as_str())?;
    let redis_conn = redis::aio::ConnectionManager::new(redis_client).await?;
    tracing::info!("✅ Gateway connecté à Redis");

    let hub = Arc::new(Hub::new(redis_conn));
    let state = Arc::new(GatewayState { hub, jwt_secret });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/ws", get(handler::ws_handler))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("{}:{}", host, port);
    tracing::info!("🚀 Vox Gateway démarré sur {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
