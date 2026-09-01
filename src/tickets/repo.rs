use thiserror::Error;

use std::collections::BTreeMap;

use crate::domain::ticket::{Ticket, TicketDraft, TicketId, TicketPatch};

pub(crate) struct TicketStore {
    tickets: BTreeMap<TicketId, Ticket>,
}
#[derive(Error, Debug)]
pub(crate) enum StoreError {
    #[error("ticket not found for id: {0}")]
    NotFound(TicketId),
}
impl TicketStore {
    pub(crate) const fn new() -> Self {
        Self {
            tickets: BTreeMap::new(),
        }
    }

    pub(super) fn add_ticket(&mut self, draft: TicketDraft) -> TicketId {
        let id = TicketId::new();
        let ticket = Ticket::new(id, draft);
        self.tickets.insert(id, ticket);
        id
    }

    pub(super) fn patch_ticket(
        &mut self,
        id: TicketId,
        patch: TicketPatch,
    ) -> Result<(), StoreError> {
        let Some(ticket) = self.tickets.get_mut(&id) else {
            return Err(StoreError::NotFound(id));
        };
        let TicketPatch {
            title,
            description,
            status,
        } = patch;

        if let Some(value) = title {
            ticket.title = value;
        }

        if let Some(value) = description {
            ticket.description = value;
        }

        if let Some(value) = status {
            ticket.status = value;
        }

        Ok(())
    }
    pub(super) fn get_ticket(&self, id: TicketId) -> Result<&Ticket, StoreError> {
        self.tickets.get(&id).ok_or(StoreError::NotFound(id))
    }
}
