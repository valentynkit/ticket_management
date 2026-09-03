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

impl TicketDraft {
    pub(crate) fn title(&self) -> &Title {
        &self.title
    }
    pub(crate) fn description(&self) -> &Description {
        &self.description
    }
}
#[derive(Serialize, Debug, Clone, sqlx::FromRow)]
pub(crate) struct Ticket {
    pub(crate) id: TicketId,
    pub(crate) title: Title,
    pub(crate) description: Description,
    pub(crate) status: Status,
    #[serde(with = "time::serde::rfc3339")]
    pub(crate) created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub(crate) updated_at: OffsetDateTime,
}
