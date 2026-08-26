mod data;
mod store;

use std::{
    clone,
    sync::{Arc, Mutex},
};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use serde::Serialize;
use tokio::{net::TcpListener, signal};
use tracing::{error, info};

pub use crate::data::{
    data_fields::{Description, Status, TicketId, Title},
    ticket_data::TicketDraft,
};
use crate::{
    data::ticket_data::{Ticket, TicketPatch},
    store::TicketStore,
};

// This is our last exercise. Let's go down a more unstructured path!
// Try writing an **asynchronous REST API** to expose the functionality
// of the ticket management system we built throughout the course.
// It should expose endpoints to:
//  - Create a ticket
//  - Retrieve ticket details
//  - Patch a ticket
//
// Use Rust's package registry, crates.io, to find the dependencies you need
// (if any) to build this system.
//
async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}
pub struct AppState {
    /// TODO: rewrite to RwLock
    store: Mutex<TicketStore>,
}

impl AppState {
    const fn new() -> Self {
        let store = Mutex::new(TicketStore::new());
        Self { store }
    }
}

pub fn app() -> Router {
    let state = Arc::new(AppState::new());
    Router::new()
        .route("/ticket/{id}", routing::get(get_ticket).patch(patch_ticket))
        .route("/ticket", routing::post(create_ticket))
        .with_state(state)
        .fallback(handler_404)
}

pub async fn run() -> Result<(), AppError> {
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app())
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {error!("shutdown ctrl_c")},
        () = terminate => {error!("shutdown terminate")}
    }
}

pub struct AppError(anyhow::Error);
impl AppError {
    #[must_use]
    pub const fn inner(&self) -> &anyhow::Error {
        &self.0
    }
}

pub async fn patch_ticket(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u64>,
    Json(payload): Json<TicketPatch>,
) -> Result<(), AppError> {
    let mut ticket_store = state.store.lock().expect("stored lock poisoned");
    ticket_store.patch_ticket(&id.into(), payload)?;
    Ok(())
}

pub async fn create_ticket(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<TicketDraft>,
) -> Result<(StatusCode, Json<TicketId>), AppError> {
    let mut ticket_store = state.store.lock().expect("stored lock poisoned");
    let ticket_id = ticket_store.add_ticket(payload);
    Ok((StatusCode::CREATED, Json(ticket_id)))
}

pub async fn get_ticket(
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

// IntoResponse trait
// From for AppError
fn try_thing() -> Result<(), anyhow::Error> {
    anyhow::bail!("it failed!")
}
