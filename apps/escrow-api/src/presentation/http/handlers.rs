use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};

use crate::presentation::http::{
    requests::{CreateEscrowRequest, ListEscrowsQuery},
    response::{error_response, ApiResponse},
    state::AppState,
};

pub async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

pub async fn list_escrows(
    State(state): State<AppState>,
    Query(query): Query<ListEscrowsQuery>,
) -> Result<
    Json<ApiResponse<Vec<crate::domain::escrow::Escrow>>>,
    (StatusCode, Json<ApiResponse<()>>),
> {
    state
        .escrow_service
        .list_escrows(query.buyer.as_deref())
        .await
        .map(|escrows| Json(ApiResponse::ok(escrows)))
        .map_err(error_response)
}

pub async fn create_escrow(
    State(state): State<AppState>,
    Json(request): Json<CreateEscrowRequest>,
) -> Result<
    (StatusCode, Json<ApiResponse<crate::domain::escrow::Escrow>>),
    (StatusCode, Json<ApiResponse<()>>),
> {
    state
        .escrow_service
        .create_escrow(request)
        .await
        .map(|escrow| (StatusCode::CREATED, Json(ApiResponse::ok(escrow))))
        .map_err(error_response)
}

pub async fn release_escrow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<crate::domain::escrow::Escrow>>, (StatusCode, Json<ApiResponse<()>>)> {
    state
        .escrow_service
        .release_escrow(&id)
        .await
        .map(|escrow| Json(ApiResponse::ok(escrow)))
        .map_err(error_response)
}

pub async fn cancel_escrow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<crate::domain::escrow::Escrow>>, (StatusCode, Json<ApiResponse<()>>)> {
    state
        .escrow_service
        .cancel_escrow(&id)
        .await
        .map(|escrow| Json(ApiResponse::ok(escrow)))
        .map_err(error_response)
}

pub async fn delete_escrow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    state
        .escrow_service
        .delete_escrow(&id)
        .await
        .map(|_| Json(ApiResponse::message("Deleted")))
        .map_err(error_response)
}
