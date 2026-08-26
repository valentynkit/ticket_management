use axum::http::Request;
use ticket_management::TicketDraft;
use tower::ServiceExt;
mod common;

#[tokio::test]
async fn create_ticket_returns_201() {
    let title = "test title";
    let description = "test description";
    let draft = TicketDraft::new(title.into(), description.into());
    let response = ticket_management::app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/ticket")
                .header("content_type", "application/json")
                .body(draft.into())
                .unwrap(),
        )
        .await
        .unwrap();
    common::setup();
    let result = ticket_management::app().await;
    assert_eq!(result, Ok(()));
    Ok(())
}
