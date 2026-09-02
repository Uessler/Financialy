use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("não autorizado")]
    Unauthorized,
    #[error("recurso não encontrado")]
    NotFound,
    #[error("dados inválidos: {0}")]
    Validation(String),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            Self::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            Self::Validation(_) => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()),
            Self::Database(sqlx::Error::RowNotFound) => {
                (StatusCode::NOT_FOUND, "recurso não encontrado".into())
            }
            _ => {
                tracing::error!(error = ?self, "request failed");
                (StatusCode::INTERNAL_SERVER_ERROR, "erro interno".into())
            }
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
