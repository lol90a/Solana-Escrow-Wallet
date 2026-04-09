use std::{net::SocketAddr, sync::Arc};

use deadpool_postgres::{Config as PoolConfig, Runtime};
use escrow_api::{
    application::services::EscrowService,
    config::AppConfig,
    infrastructure::{migrations::run_migrations, postgres::repository::PostgresEscrowRepository},
    presentation::http::{router::build_router, state::AppState},
};
use tokio_postgres::NoTls;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    bootstrap_tracing();

    let _ = dotenvy::dotenv();
    let config = AppConfig::from_env().expect("failed to load configuration");

    let pool = create_pool(&config).expect("failed to create database pool");
    run_migrations(&pool)
        .await
        .expect("database migration failed");

    let repository = PostgresEscrowRepository::new(pool);
    let service = Arc::new(EscrowService::new(repository));
    let state = AppState::new(service);
    let app = build_router(state);

    let address = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("HTTP API listening on http://{address}");

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("failed to bind tcp listener");

    axum::serve(listener, app)
        .await
        .expect("http server exited unexpectedly");
}

fn bootstrap_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "escrow_api=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

fn create_pool(
    config: &AppConfig,
) -> Result<deadpool_postgres::Pool, deadpool_postgres::CreatePoolError> {
    let mut pool_config = PoolConfig::new();
    pool_config.url = Some(config.database_url.clone());
    pool_config.pool = Some(deadpool_postgres::PoolConfig::new(config.max_pool_size));
    pool_config.create_pool(Some(Runtime::Tokio1), NoTls)
}
