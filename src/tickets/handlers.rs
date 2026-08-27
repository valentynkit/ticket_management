use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::{
    domain::ticket::{Ticket, TicketDraft, TicketId, TicketPatch},
    error::ApiError,
    state::AppState,
};

pub(super) async fn patch_ticket(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u64>,
    Json(payload): Json<TicketPatch>,
) -> Result<StatusCode, ApiError> {
    let mut ticket_store = state.store.lock().expect("stored lock poisoned");
    ticket_store.patch_ticket(id.into(), payload)?;
    Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn create_ticket(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<TicketDraft>,
) -> Result<(StatusCode, Json<TicketId>), ApiError> {
    let mut ticket_store = state.store.lock().expect("stored lock poisoned");
    let ticket_id = ticket_store.add_ticket(payload);
    Ok((StatusCode::CREATED, Json(ticket_id)))
}

pub(super) async fn get_ticket(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u64>,
) -> Result<Json<Ticket>, ApiError> {
    let ticket_store = state.store.lock().expect("stored lock poisoned");
    // 404 now travels the error channel; the success type says only what success is.
    let ticket = ticket_store.get_ticket(id.into())?.clone();
    Ok(Json(ticket))
}
