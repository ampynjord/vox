    mod config;
mod routes;
mod middleware;
mod state;

use axum::Router;
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use crate::config::Config;
use crate::state::AppState;
use crate::routes::{auth_routes, space_routes, channel_routes, message_routes, file_routes, role_routes, voice_routes};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Charger les variables d'environnement
    dotenvy::dotenv().ok();

    // Initialiser le logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Charger la configuration
    let config = Config::from_env();
    
    // Créer l'état partagé de l'application
    let state = AppState::new(&config).await?;

    // Exécuter les migrations
    tracing::info!("Exécution des migrations...");
    sqlx::migrate!("../migrations")
        .run(&state.db)
        .await?;
    tracing::info!("Migrations terminées.");

    // Configurer CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Construire le routeur
    let app = Router::new()
        .nest("/api/v1/auth", auth_routes())
        .nest("/api/v1/spaces", space_routes())
        .nest("/api/v1/channels", channel_routes())
        .nest("/api/v1/messages", message_routes())
        .nest("/api/v1/files", file_routes())
        .nest("/api/v1/voice", voice_routes())
        .nest("/api/v1", role_routes())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Démarrer le serveur
    let addr = format!("{}:{}", config.api_host, config.api_port);
    tracing::info!("🚀 Vox API démarré sur {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
