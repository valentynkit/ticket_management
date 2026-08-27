mod handlers;
pub(super) mod repo;
use std::sync::Arc;

use handlers::{create_ticket, get_ticket, patch_ticket};

use crate::state::AppState;

use axum::{routing, Router};

pub(crate) fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/ticket/{id}", routing::get(get_ticket).patch(patch_ticket))
        .route("/ticket", routing::post(create_ticket))
}
