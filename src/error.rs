use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
pub struct AppError(anyhow::Error);
impl AppError {
    #[must_use]
    pub const fn inner(&self) -> &anyhow::Error {
        &self.0
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(value: E) -> Self {
        Self(value.into())
    }
}

/*
impl From<anyhow::Error> for AppError {
    fn from(value: anyhow::Error) -> Self {
        Self(value)
    }
}
*/

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // it's often easiest to implement `IntoResponse` by calling other implementations
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something wend wrong: {}", self.0),
        )
            .into_response()
    }
}
