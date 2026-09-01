use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use thiserror::Error;
use tracing::error;

use crate::domain::ticket::TicketId;
use crate::tickets::repo::StoreError;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("ticket {0} not found")]
    NotFound(TicketId),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            Self::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            Self::Internal(e) => {
                error!(error=?e, "internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".into(),
                )
            }
        };
        (status, Json(json!({"error": message}))).into_response()
    }
}

impl From<StoreError> for ApiError {
    fn from(value: StoreError) -> Self {
        match value {
            StoreError::NotFound(id) => Self::NotFound(id),
            StoreError::DbDriverInternal(err) => Self::Internal(err.into()),
        }
    }
}
