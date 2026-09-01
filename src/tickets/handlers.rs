use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    domain::ticket::{Ticket, TicketDraft, TicketId, TicketPatch},
    error::ApiError,
    state::AppState,
    tickets::repo,
};

const DEFAULT_LIMIT: u32 = 20;
const MAX_LIMIT: u32 = 100;

#[derive(Deserialize)]
pub(super) struct ListQuery {
    after: Option<Uuid>,
    limit: Option<u32>,
}

#[derive(Serialize)]
pub(super) struct TicketPage {
    items: Vec<Ticket>,
    next_cursor: Option<TicketId>,
}

pub(super) async fn patch_ticket(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<TicketPatch>,
) -> Result<StatusCode, ApiError> {
    repo::patch_ticket(state.pg_pool(), id.into(), payload).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn delete_ticket(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    repo::delete_ticket(state.pg_pool(), id.into()).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn list_tickets(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<TicketPage>, ApiError> {
    // Clamped, not rejected: a caller asking for a million rows gets the cap, and an
    // unbounded LIMIT would let one request tie up a pool connection indefinitely.
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let items = repo::list_tickets(
        state.pg_pool(),
        query.after.map(Into::into),
        i64::from(limit),
    )
    .await?;

    // A short page means the end of the table; only a full page can have more after it.
    let next_cursor = (items.len() == limit as usize)
        .then(|| items.last().map(|ticket| ticket.id))
        .flatten();

    Ok(Json(TicketPage { items, next_cursor }))
}

pub(super) async fn health(State(state): State<Arc<AppState>>) -> StatusCode {
    match repo::ping(state.pg_pool()).await {
        Ok(()) => StatusCode::OK,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}

pub(super) async fn create_ticket(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<TicketDraft>,
) -> Result<(StatusCode, Json<TicketId>), ApiError> {
    let ticket_id = repo::add_ticket(state.pg_pool(), payload).await?;
    Ok((StatusCode::CREATED, Json(ticket_id)))
}

pub(super) async fn get_ticket(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Ticket>, ApiError> {
    // 404 now travels the error channel; the success type says only what success is.
    let ticket = repo::get_ticket(state.pg_pool(), id.into()).await?;
    Ok(Json(ticket))
}
