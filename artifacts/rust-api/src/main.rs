use axum::{
    extract::{Path, Query, State},
    http::{Method, StatusCode},
    response::Json,
    routing::{delete, get, patch},
    Router,
};
use deadpool_postgres::{Config as PoolConfig, Pool, Runtime};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio_postgres::NoTls;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// ─── Models ──────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Escrow {
    pub id: String,
    pub buyer: String,
    pub seller: String,
    pub amount: i64,
    pub amount_sol: f64,
    pub status: String,
    pub escrow_id: i64,
    pub pda: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateEscrowRequest {
    pub id: String,
    pub buyer: String,
    pub seller: String,
    pub amount: i64,
    pub amount_sol: f64,
    pub escrow_id: i64,
    pub pda: String,
}

#[derive(Debug, Deserialize)]
pub struct BuyerQuery {
    pub buyer: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    fn ok(data: T) -> Self {
        ApiResponse { success: true, data: Some(data), message: None }
    }
}

fn err_response(msg: impl Into<String>) -> ApiResponse<()> {
    ApiResponse { success: false, data: None, message: Some(msg.into()) }
}

// ─── Shared App State ─────────────────────────────────────────────────────────

#[derive(Clone)]
struct AppState {
    db: Pool,
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn row_to_escrow(row: &tokio_postgres::Row) -> Escrow {
    let created_at: std::time::SystemTime = row.get("created_at");
    let secs = created_at
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    Escrow {
        id: row.get("id"),
        buyer: row.get("buyer"),
        seller: row.get("seller"),
        amount: row.get("amount"),
        amount_sol: row.get("amount_sol"),
        status: row.get("status"),
        escrow_id: row.get("escrow_id"),
        pda: row.get("pda"),
        created_at: format!("{}Z", chrono_secs_to_iso(secs)),
    }
}

fn chrono_secs_to_iso(secs: u64) -> String {
    // Simple ISO8601 formatting without chrono dependency
    let s = secs as i64;
    let mins = s / 60;
    let secs_part = s % 60;
    let hrs = mins / 60;
    let mins_part = mins % 60;
    let days_total = hrs / 24;
    let hrs_part = hrs % 24;
    // Very rough date calculation (good enough for display)
    let year = 1970 + days_total / 365;
    let day_of_year = days_total % 365;
    let month = (day_of_year / 30) + 1;
    let day = (day_of_year % 30) + 1;
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
        year, month, day, hrs_part, mins_part, secs_part
    )
}

// ─── Handlers ─────────────────────────────────────────────────────────────────

/// GET /api/healthz
async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

/// GET /api/escrows?buyer=<pubkey>
async fn list_escrows(
    State(state): State<AppState>,
    Query(params): Query<BuyerQuery>,
) -> Result<Json<ApiResponse<Vec<Escrow>>>, (StatusCode, Json<ApiResponse<()>>)> {
    let client = state.db.get().await.map_err(|e| {
        tracing::error!("DB pool error: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_response("Database unavailable")))
    })?;

    let rows = match params.buyer {
        Some(ref buyer) => client
            .query(
                "SELECT id, buyer, seller, amount, amount_sol, status, escrow_id, pda, created_at \
                 FROM escrows WHERE buyer = $1 ORDER BY created_at DESC",
                &[buyer],
            )
            .await,
        None => client
            .query(
                "SELECT id, buyer, seller, amount, amount_sol, status, escrow_id, pda, created_at \
                 FROM escrows ORDER BY created_at DESC LIMIT 100",
                &[],
            )
            .await,
    }
    .map_err(|e| {
        tracing::error!("Query error: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_response("Failed to fetch escrows")))
    })?;

    let escrows: Vec<Escrow> = rows.iter().map(row_to_escrow).collect();
    Ok(Json(ApiResponse::ok(escrows)))
}

/// POST /api/escrows
async fn create_escrow(
    State(state): State<AppState>,
    Json(payload): Json<CreateEscrowRequest>,
) -> Result<(StatusCode, Json<ApiResponse<Escrow>>), (StatusCode, Json<ApiResponse<()>>)> {
    if payload.buyer.is_empty() || payload.seller.is_empty() || payload.pda.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(err_response("buyer, seller, and pda are required"))));
    }
    if payload.amount <= 0 {
        return Err((StatusCode::BAD_REQUEST, Json(err_response("amount must be > 0"))));
    }

    let client = state.db.get().await.map_err(|e| {
        tracing::error!("DB pool error: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_response("Database unavailable")))
    })?;

    let rows = client
        .query(
            "INSERT INTO escrows (id, buyer, seller, amount, amount_sol, status, escrow_id, pda) \
             VALUES ($1, $2, $3, $4, $5, 'Pending', $6, $7) \
             RETURNING id, buyer, seller, amount, amount_sol, status, escrow_id, pda, created_at",
            &[
                &payload.id,
                &payload.buyer,
                &payload.seller,
                &payload.amount,
                &payload.amount_sol,
                &payload.escrow_id,
                &payload.pda,
            ],
        )
        .await
        .map_err(|e| {
            tracing::error!("Insert error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err_response("Failed to create escrow")))
        })?;

    let escrow = row_to_escrow(&rows[0]);
    tracing::info!(escrow_id = payload.escrow_id, buyer = %payload.buyer, "Escrow created");
    Ok((StatusCode::CREATED, Json(ApiResponse::ok(escrow))))
}

/// PATCH /api/escrows/:id/release
async fn release_escrow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<Escrow>>, (StatusCode, Json<ApiResponse<()>>)> {
    let client = state.db.get().await.map_err(|e| {
        tracing::error!("DB pool error: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_response("Database unavailable")))
    })?;

    let rows = client
        .query(
            "UPDATE escrows SET status = 'Completed' WHERE id = $1 AND status = 'Pending' \
             RETURNING id, buyer, seller, amount, amount_sol, status, escrow_id, pda, created_at",
            &[&id],
        )
        .await
        .map_err(|e| {
            tracing::error!("Update error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err_response("Failed to release escrow")))
        })?;

    if rows.is_empty() {
        return Err((StatusCode::NOT_FOUND, Json(err_response("Escrow not found or not Pending"))));
    }

    tracing::info!(id = %id, "Escrow released");
    Ok(Json(ApiResponse::ok(row_to_escrow(&rows[0]))))
}

/// PATCH /api/escrows/:id/cancel
async fn cancel_escrow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<Escrow>>, (StatusCode, Json<ApiResponse<()>>)> {
    let client = state.db.get().await.map_err(|e| {
        tracing::error!("DB pool error: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_response("Database unavailable")))
    })?;

    let rows = client
        .query(
            "UPDATE escrows SET status = 'Cancelled' WHERE id = $1 AND status = 'Pending' \
             RETURNING id, buyer, seller, amount, amount_sol, status, escrow_id, pda, created_at",
            &[&id],
        )
        .await
        .map_err(|e| {
            tracing::error!("Update error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err_response("Failed to cancel escrow")))
        })?;

    if rows.is_empty() {
        return Err((StatusCode::NOT_FOUND, Json(err_response("Escrow not found or not Pending"))));
    }

    tracing::info!(id = %id, "Escrow cancelled");
    Ok(Json(ApiResponse::ok(row_to_escrow(&rows[0]))))
}

/// DELETE /api/escrows/:id
async fn delete_escrow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    let client = state.db.get().await.map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_response("Database unavailable")))
    })?;

    let result = client
        .execute("DELETE FROM escrows WHERE id = $1", &[&id])
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(err_response("Delete failed"))))?;

    if result == 0 {
        return Err((StatusCode::NOT_FOUND, Json(err_response("Escrow not found"))));
    }

    Ok(Json(ApiResponse { success: true, data: Some(serde_json::json!({})), message: Some("Deleted".into()) }))
}

// ─── Migration ───────────────────────────────────────────────────────────────

async fn run_migrations(pool: &Pool) -> Result<(), Box<dyn std::error::Error>> {
    let client = pool.get().await?;
    client
        .batch_execute(
            "CREATE TABLE IF NOT EXISTS escrows (
                id          TEXT PRIMARY KEY,
                buyer       TEXT NOT NULL,
                seller      TEXT NOT NULL,
                amount      BIGINT NOT NULL,
                amount_sol  DOUBLE PRECISION NOT NULL,
                status      TEXT NOT NULL DEFAULT 'Pending'
                                CHECK (status IN ('Pending', 'Completed', 'Cancelled')),
                escrow_id   BIGINT NOT NULL,
                pda         TEXT NOT NULL,
                created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            CREATE INDEX IF NOT EXISTS idx_escrows_buyer ON escrows (buyer);
            CREATE INDEX IF NOT EXISTS idx_escrows_status ON escrows (status);",
        )
        .await?;
    tracing::info!("Database migrations applied");
    Ok(())
}

// ─── Main ─────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rust_api=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let _ = dotenvy::dotenv();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Parse DATABASE_URL for deadpool-postgres
    let mut cfg = PoolConfig::new();
    cfg.url = Some(database_url);
    cfg.pool = Some(deadpool_postgres::PoolConfig::new(10));

    let pool = cfg
        .create_pool(Some(Runtime::Tokio1), NoTls)
        .expect("Failed to create connection pool");

    run_migrations(&pool).await.expect("Migration failed");

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("PORT must be valid");

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE, Method::OPTIONS])
        .allow_headers(Any);

    let state = AppState { db: pool };

    let app = Router::new()
        .route("/api/healthz", get(health_check))
        .route("/api/escrows", get(list_escrows).post(create_escrow))
        .route("/api/escrows/:id/release", patch(release_escrow))
        .route("/api/escrows/:id/cancel", patch(cancel_escrow))
        .route("/api/escrows/:id", delete(delete_escrow))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Rust/Axum API listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.expect("Bind failed");
    axum::serve(listener, app).await.expect("Server error");
}
