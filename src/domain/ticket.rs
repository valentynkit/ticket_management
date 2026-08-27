mod values;
use serde::{Deserialize, Serialize};
pub(crate) use values::{Description, Status, TicketId, Title};

#[derive(Deserialize, Default)]
pub(crate) struct TicketPatch {
    pub(crate) title: Option<Title>,
    pub(crate) description: Option<Description>,
    pub(crate) status: Option<Status>,
}

#[derive(Deserialize)]
pub(crate) struct TicketDraft {
    title: Title,
    description: Description,
}

#[derive(Serialize, Clone)]
pub(crate) struct Ticket {
    id: TicketId,
    pub(crate) title: Title,
    pub(crate) description: Description,
    pub(crate) status: Status,
}

impl Ticket {
    pub(crate) fn new(id: TicketId, draft: TicketDraft) -> Self {
        let TicketDraft { title, description } = draft;
        Self {
            id,
            title,
            description,
            status: Status::ToDo,
        }
    }
}
