mod values;
use serde::{Deserialize, Serialize};
use sqlx::types::time::OffsetDateTime;
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

#[derive(Serialize, Clone, sqlx::FromRow)]
pub(crate) struct Ticket {
    id: TicketId,
    pub(crate) title: Title,
    pub(crate) description: Description,
    pub(crate) status: Status,
    #[serde(with = "time::serde::rfc3339")]
    pub(crate) created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub(crate) updated_at: OffsetDateTime,
}

impl Ticket {
    pub(crate) fn new(id: TicketId, draft: TicketDraft) -> Self {
        let TicketDraft { title, description } = draft;
        let created_at = OffsetDateTime::now_utc();
        let updated_at = OffsetDateTime::now_utc();

        Self {
            id,
            title,
            description,
            status: Status::Todo,
            created_at,
            updated_at,
        }
    }
}
