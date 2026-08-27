mod domain;
mod error;
mod state;
mod tickets;

pub use error::AppError;
use state::AppState;
use std::sync::Arc;

use tower_http::trace::TraceLayer;

use axum::{http::StatusCode, response::IntoResponse, Router};
use tokio::{net::TcpListener, signal};
use tracing::{error, info};

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}

fn build_router(state: AppState) -> Router {
    let state = Arc::new(state);
    Router::new()
        .merge(tickets::router())
        .fallback(handler_404)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

pub fn app() -> Router {
    build_router(AppState::new())
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
