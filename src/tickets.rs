mod handlers;
pub(super) mod repo;
use std::sync::Arc;

use handlers::{create_ticket, delete_ticket, get_ticket, health, list_tickets, patch_ticket};

use crate::state::AppState;

use axum::{routing, Router};

pub(crate) fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/ticket/{id}",
            routing::get(get_ticket)
                .patch(patch_ticket)
                .delete(delete_ticket),
        )
        .route("/ticket", routing::get(list_tickets).post(create_ticket))
        .route("/health", routing::get(health))
}
