mod config;
mod domain;
mod error;
mod state;
mod tickets;

pub use config::AppConfig;
use state::AppState;
use std::sync::Arc;

use tower_http::trace::TraceLayer;

use anyhow::Context;
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

/// Serve on an already-bound listener. Production and the test harness both go
/// through here, so they exercise the same router, fallback and shutdown path —
/// they differ only in which listener they hand over.
///
/// Startup failures are `anyhow`, not [`ApiError`]: there is no client to answer,
/// so there is no status code to pick. Giving `ApiError` a `From<io::Error>` would
/// also let any stray io error in a handler become a silent 500.
pub async fn serve(listener: TcpListener) -> anyhow::Result<()> {
    info!(addr = %listener.local_addr().context("listener has no local address")?, "listening");
    axum::serve(listener, app())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server stopped unexpectedly")
}

pub async fn run(config: &AppConfig) -> anyhow::Result<()> {
    let address = config.server().address();
    let listener = TcpListener::bind(&address)
        .await
        .with_context(|| format!("could not bind {address}"))?;
    serve(listener).await
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
