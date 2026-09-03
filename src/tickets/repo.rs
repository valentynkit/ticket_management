use std::sync::Arc;

use sqlx::PgPool;
use thiserror::Error;

use crate::{
    cache,
    domain::ticket::{Description, Status, Ticket, TicketDraft, TicketId, TicketPatch, Title},
    state::AppState,
};

#[derive(Error, Debug)]
pub(crate) enum StoreError {
    #[error("ticket not found for id: {0}")]
    NotFound(TicketId),
    #[error("request violates constraint `{0}`")]
    Constraint(String),
    #[error(transparent)]
    DbDriverInternal(sqlx::Error),
}

impl From<sqlx::Error> for StoreError {
    fn from(err: sqlx::Error) -> Self {
        // 23514 check_violation / 23502 not_null_violation: the payload broke a column
        // constraint, so it is the caller's problem, not a server fault.
        let violated = err
            .as_database_error()
            .filter(|db| matches!(db.code().as_deref(), Some("23514" | "23502")))
            .map(|db| db.constraint().unwrap_or("unknown").to_owned());

        violated.map_or_else(|| Self::DbDriverInternal(err), Self::Constraint)
    }
}

pub(super) async fn patch_ticket(
    state: &AppState,
    id: TicketId,
    patch: TicketPatch,
) -> Result<(), StoreError> {
    let TicketPatch {
        title,
        description,
        status,
    } = patch;

    let cache = state.ticket_cache();
    let pool = state.pg_pool();

    // COALESCE keeps the stored value when a field is absent from the payload, so the
    // whole patch is one atomic statement instead of a read-modify-write.
    let ticket = sqlx::query_as!(
        Ticket,
        r#"
            UPDATE tickets
            SET title       = COALESCE($2, title),
                description = COALESCE($3, description),
                status      = COALESCE($4, status),
                updated_at  = now()
            WHERE id = $1
            RETURNING id AS "id: TicketId", 
            title AS "title: Title", 
            description AS "description: Description",
            status AS "status: Status", 
            updated_at, 
            created_at
        "#,
        id.0,
        title.map(|value| value.0),
        description.map(|value| value.0),
        status as Option<Status>
    )
    .fetch_optional(pool)
    .await?
    .ok_or(StoreError::NotFound(id))?;

    cache.insert(id.0, ticket).await;
    Ok(())
}

pub(super) async fn get_ticket(state: &AppState, id: TicketId) -> Result<Ticket, Arc<StoreError>> {
    // check cache
    let cache = state.ticket_cache();
    let pool = state.pg_pool();
    cache
        .try_get_with(id.0, async {
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
        })
        .await
}

pub(super) async fn delete_ticket(state: &AppState, id: TicketId) -> Result<(), StoreError> {
    let cache = state.ticket_cache();
    let pool = state.pg_pool();
    let rows_affected = sqlx::query!("DELETE FROM tickets WHERE id = $1", id.0)
        .execute(pool)
        .await?
        .rows_affected();

    if rows_affected == 0 {
        return Err(StoreError::NotFound(id));
    }
    cache.invalidate(&id.0).await;
    Ok(())
}

pub(super) async fn list_tickets(
    pool: &PgPool,
    after: Option<TicketId>,
    limit: i64,
) -> Result<Vec<Ticket>, StoreError> {
    // Keyset pagination: uuidv7 sorts by creation time, so the primary key doubles as
    // the cursor and the index seeks straight to it instead of counting past OFFSET rows.
    let tickets = sqlx::query_as!(
        Ticket,
        r#"
            SELECT id AS "id: TicketId",
                   title AS "title: Title",
                   description AS "description: Description",
                   status AS "status: Status",
                   created_at,
                   updated_at
            FROM tickets
            WHERE $1::uuid IS NULL OR id > $1
            ORDER BY id
            LIMIT $2
        "#,
        after.map(|cursor| cursor.0),
        limit
    )
    .fetch_all(pool)
    .await?;

    Ok(tickets)
}

pub(super) async fn ping(pool: &PgPool) -> Result<(), StoreError> {
    sqlx::query!("SELECT 1 AS one").fetch_one(pool).await?;
    Ok(())
}

pub(super) async fn add_ticket(
    state: &AppState,
    draft: TicketDraft,
) -> Result<TicketId, StoreError> {
    let title = &draft.title().0;
    let description = &draft.description().0;
    let pool = state.pg_pool();
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
