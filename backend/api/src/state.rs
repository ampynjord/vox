use sqlx::PgPool;
use redis::aio::ConnectionManager;
use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: ConnectionManager,
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
}

impl AppState {
    pub async fn new(config: &Config) -> anyhow::Result<Self> {
        // Connexion PostgreSQL
        let db = PgPool::connect(&config.database_url).await?;
        tracing::info!("✅ Connecté à PostgreSQL");

        // Connexion Redis
        let redis_client = redis::Client::open(config.redis_url.as_str())?;
        let redis = ConnectionManager::new(redis_client).await?;
        tracing::info!("✅ Connecté à Redis");

        Ok(Self {
            db,
            redis,
            jwt_secret: config.jwt_secret.clone(),
            jwt_expiration_hours: config.jwt_expiration_hours,
        })
    }
}
