use std::sync::Mutex;

use crate::tickets::repo::TicketStore;

pub(crate) struct AppState {
    /// TODO: rewrite to RwLock
    pub(crate) store: Mutex<TicketStore>,
}

impl AppState {
    pub(crate) const fn new() -> Self {
        let store = Mutex::new(TicketStore::new());
        Self { store }
    }
}
