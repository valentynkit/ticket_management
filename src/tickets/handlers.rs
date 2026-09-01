use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::{
    domain::ticket::{Ticket, TicketDraft, TicketId, TicketPatch},
    error::ApiError,
    state::AppState,
    tickets::repo::{add_ticket, get_ticket as fetch_ticket, patch_ticket as apply_patch},
};

pub(super) async fn patch_ticket(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<TicketPatch>,
) -> Result<StatusCode, ApiError> {
    apply_patch(state.pg_pool(), id.into(), payload).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn create_ticket(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<TicketDraft>,
) -> Result<(StatusCode, Json<TicketId>), ApiError> {
    let pg_pool = state.pg_pool();
    let ticket_id = add_ticket(pg_pool, payload).await?;
    Ok((StatusCode::CREATED, Json(ticket_id)))
}

pub(super) async fn get_ticket(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Ticket>, ApiError> {
    // 404 now travels the error channel; the success type says only what success is.
    let ticket = fetch_ticket(state.pg_pool(), id.into()).await?;
    Ok(Json(ticket))
}
