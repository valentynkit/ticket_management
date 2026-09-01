use std::sync::atomic::{AtomicU32, Ordering};

use axum::Router;
use reqwest::Client;
use sqlx::{AssertSqlSafe, Connection, PgConnection};
use ticket_management::AppConfig;
use tokio::net::TcpListener;

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// Every test gets its own database so the suite can assert on whole-table state
/// (pagination, counts) without seeing rows from tests running in parallel.
/// Process id plus a counter is enough: unique within a run, and stdlib only.
async fn provision_database() -> String {
    let config = AppConfig::load().expect("could not load configuration");
    let base = config.postgres_connection();
    let (prefix, _) = base
        .rsplit_once('/')
        .expect("connection string has no database segment");

    let name = format!(
        "test_{}_{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    );

    let mut maintenance = PgConnection::connect(&format!("{prefix}/postgres"))
        .await
        .expect("could not reach the maintenance database");

    // A database name cannot be a bind parameter, so it is interpolated. Safe here
    // because `name` is built from a pid and a counter, never from input.
    sqlx::raw_sql(AssertSqlSafe(format!(r#"CREATE DATABASE "{name}""#)))
        .execute(&mut maintenance)
        .await
        .expect("could not create the test database");

    format!("{prefix}/{name}")
}

/// Builds the app against a fresh database. `app` runs the embedded migrations, so the
/// schema is whatever `migrations/` says — no fixture files to drift.
pub async fn app() -> Router {
    ticket_management::app(provision_database().await)
        .await
        .expect("could not build the app")
}

/// Port 0 lets the OS pick a free port, so tests run in parallel and never collide
/// with a dev server. Binding happens here (synchronously) rather than inside the
/// spawned task — once `bind` returns, the socket is listening and the kernel
/// queues connections, so there is no race with `serve` being polled.
pub async fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let router = app().await;
    tokio::spawn(ticket_management::serve(listener, router));
    format!("http://{addr}")
}

pub async fn setup() -> (Client, String) {
    (reqwest::Client::new(), spawn_app().await)
}
