use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::{
    domain::ticket::{Ticket, TicketDraft, TicketId, TicketPatch},
    error::AppError,
    state::AppState,
};

pub(super) async fn patch_ticket(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u64>,
    Json(payload): Json<TicketPatch>,
) -> Result<(), AppError> {
    let mut ticket_store = state.store.lock().expect("stored lock poisoned");
    ticket_store.patch_ticket(id.into(), payload)?;
    Ok(())
}

pub(super) async fn create_ticket(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<TicketDraft>,
) -> Result<(StatusCode, Json<TicketId>), AppError> {
    let mut ticket_store = state.store.lock().expect("stored lock poisoned");
    let ticket_id = ticket_store.add_ticket(payload);
    Ok((StatusCode::CREATED, Json(ticket_id)))
}

pub(super) async fn get_ticket(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u64>,
) -> Result<(StatusCode, Json<Option<Ticket>>), AppError> {
    let ticket_store = state.store.lock().expect("stored lock poisoned");
    let ticket = ticket_store
        .get_ticket(&id.into())
        .map(|item| item.to_owned());
    let response = ticket.map_or((StatusCode::NOT_FOUND, Json(None)), |value| {
        (StatusCode::OK, Json(Some(value)))
    });
    Ok(response)
}
