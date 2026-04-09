use axum::{
    http::Method,
    routing::{delete, get, patch},
    Router,
};
use tower_http::cors::{Any, CorsLayer};

use crate::presentation::http::{
    handlers::{
        cancel_escrow, create_escrow, delete_escrow, health_check, list_escrows, release_escrow,
    },
    state::AppState,
};

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(Any);

    Router::new()
        .route("/api/healthz", get(health_check))
        .route("/api/escrows", get(list_escrows).post(create_escrow))
        .route("/api/escrows/:id/release", patch(release_escrow))
        .route("/api/escrows/:id/cancel", patch(cancel_escrow))
        .route("/api/escrows/:id", delete(delete_escrow))
        .layer(cors)
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use tower::util::ServiceExt;

    use crate::presentation::http::handlers::health_check;

    #[tokio::test]
    async fn health_route_returns_ok() {
        let app = Router::new().route("/api/healthz", get(health_check));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/healthz")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");

        assert_eq!(response.status(), StatusCode::OK);
    }
}
