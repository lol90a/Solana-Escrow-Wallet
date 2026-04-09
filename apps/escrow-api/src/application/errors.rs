use axum::http::StatusCode;

#[derive(Debug)]
pub enum AppError {
    Validation(String),
    NotFound(String),
    Infrastructure(String),
}

impl AppError {
    pub fn message(&self) -> &str {
        match self {
            Self::Validation(message) => message,
            Self::NotFound(message) => message,
            Self::Infrastructure(message) => message,
        }
    }

    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Infrastructure(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
