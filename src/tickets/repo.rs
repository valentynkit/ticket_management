use sqlx::PgPool;
use thiserror::Error;

use crate::domain::ticket::{
    Description, Status, Ticket, TicketDraft, TicketId, TicketPatch, Title,
};

#[derive(Error, Debug)]
pub(crate) enum StoreError {
    #[error("ticket not found for id: {0}")]
    NotFound(TicketId),
    #[error(transparent)]
    DbDriverInternal(#[from] sqlx::Error),
}

pub(super) async fn patch_ticket(
    pool: &PgPool,
    id: TicketId,
    patch: TicketPatch,
) -> Result<(), StoreError> {
    let TicketPatch {
        title,
        description,
        status,
    } = patch;

    // COALESCE keeps the stored value when a field is absent from the payload, so the
    // whole patch is one atomic statement instead of a read-modify-write.
    let rows_affected = sqlx::query!(
        r#"
            UPDATE tickets
            SET title       = COALESCE($2, title),
                description = COALESCE($3, description),
                status      = COALESCE($4, status),
                updated_at  = now()
            WHERE id = $1
        "#,
        id.0,
        title.map(|value| value.0),
        description.map(|value| value.0),
        status as Option<Status>
    )
    .execute(pool)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(StoreError::NotFound(id));
    }
    Ok(())
}

pub(super) async fn get_ticket(pool: &PgPool, id: TicketId) -> Result<Ticket, StoreError> {
    sqlx::query_as!(
        Ticket,
        r#"
                SELECT id AS "id: TicketId", 
                title AS "title: Title",
                description as "description: Description", 
                status as "status: Status", 
                created_at, 
                updated_at FROM tickets WHERE id = $1
            "#,
        id.0
    )
    .fetch_optional(pool)
    .await?
    .ok_or(StoreError::NotFound(id))
}

pub(super) async fn add_ticket(pool: &PgPool, draft: TicketDraft) -> Result<TicketId, StoreError> {
    let title = &draft.title().0;
    let description = &draft.description().0;
    let rec = sqlx::query!(
        r#"
            INSERT INTO tickets (title, description)
            VALUES ($1, $2)
            RETURNING id
        "#,
        title,
        description
    )
    .fetch_one(pool)
    .await?;
    Ok(TicketId::from(rec.id))
}
