use std::{str::FromStr, sync::Mutex, time::Duration};

use anyhow::Context;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};
use tracing::info;
static PG_MAX_CONNECTIONS: u32 = 50;
static PG_MIN_CONNECTIONS: u32 = 5;
use crate::tickets::repo::TicketStore;

pub(crate) struct AppState {
    /// TODO: rewrite to RwLock
    pub(crate) store: Mutex<TicketStore>,
    pub(crate) pg_pool: PgPool,
}

impl AppState {
    pub(crate) async fn new(db_connection: String) -> anyhow::Result<Self> {
        let store = Mutex::new(TicketStore::new());
        let connect_options = PgConnectOptions::from_str(&db_connection)
            .context("`postgres_connection` is not a valid postgres URL")?;
        let pg_options = PgPoolOptions::new()
            .max_connections(PG_MAX_CONNECTIONS)
            .min_connections(PG_MIN_CONNECTIONS)
            .acquire_timeout(Duration::from_secs(3))
            .idle_timeout(Duration::from_secs(60 * 5));

        let pg_pool = pg_options
            .connect_with(connect_options)
            .await
            .context("could not open the postgres pool")?;
        info!(
            pool_size = pg_pool.size(),
            min_connections = PG_MIN_CONNECTIONS,
            max_connections = PG_MAX_CONNECTIONS,
            "PG Pool created"
        );
        Ok(Self { store, pg_pool })
    }
}
