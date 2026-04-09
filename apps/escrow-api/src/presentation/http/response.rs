use serde::Serialize;

use crate::application::errors::AppError;

#[derive(Serialize)]
pub struct ApiResponse<T>
where
    T: Serialize,
{
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

impl<T> ApiResponse<T>
where
    T: Serialize,
{
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
        }
    }
}

impl ApiResponse<serde_json::Value> {
    pub fn message(message: impl Into<String>) -> Self {
        Self {
            success: true,
            data: Some(serde_json::json!({})),
            message: Some(message.into()),
        }
    }
}

pub fn error_response(error: AppError) -> (axum::http::StatusCode, axum::Json<ApiResponse<()>>) {
    let status = error.status_code();
    let payload = ApiResponse {
        success: false,
        data: None,
        message: Some(error.message().to_string()),
    };

    (status, axum::Json(payload))
}
