use std::{str::FromStr, time::Duration};

use anyhow::Context;
use moka::future::Cache;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};
use tracing::info;
use uuid::Uuid;

use crate::{
    cache::{self, AppCache},
    domain::ticket::Ticket,
};
static PG_MAX_CONNECTIONS: u32 = 50;
static PG_MIN_CONNECTIONS: u32 = 5;

pub(crate) struct AppState {
    pg_pool: PgPool,
    cache: AppCache,
}

impl AppState {
    pub(crate) async fn new(db_connection: String) -> anyhow::Result<Self> {
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
        let cache = AppCache::new().await;
        Ok(Self { pg_pool, cache })
    }
    pub(crate) fn pg_pool(&self) -> &PgPool {
        &self.pg_pool
    }
    pub(crate) fn ticket_cache(&self) -> &Cache<Uuid, Ticket> {
        &self.cache.tickets
    }
}
