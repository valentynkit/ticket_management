use std::time::Duration;

use moka::future::{Cache, CacheBuilder};
use uuid::Uuid;

use crate::domain::ticket::Ticket;

pub(crate) struct AppCache {
    pub(crate) tickets: Cache<Uuid, Ticket>,
}

impl AppCache {
    pub(crate) async fn new() -> AppCache {
        let tickets = create_ticket_cache().await;
        Self { tickets }
    }
}

async fn create_ticket_cache() -> Cache<Uuid, Ticket> {
    Cache::builder()
        .max_capacity(1000)
        .time_to_live(Duration::from_secs(30))
        .time_to_idle(Duration::from_secs(15))
        .eviction_listener(|key, value, cause| {
            println!("Evicted user {}: {:?} - cause {:?}", key, value, cause);
        })
        .build()
}
