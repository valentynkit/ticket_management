use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

mod common;

#[tokio::test]
async fn create_ticket_returns_201() {
    let body = serde_json::json!({
        "title": "test title",
        "description": "test description"
    });

    let response = ticket_management::app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/ticket")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body = String::from_utf8_lossy(&bytes);
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
}

#[tokio::test]
async fn create_ticket_returns_422() {
    let body = serde_json::json!({
        "wrong_title": "test title",
        "wrong_description": "test description"
    });

    let response = ticket_management::app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/ticket")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body = String::from_utf8_lossy(&bytes);
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
}

#[tokio::test]
async fn create_ticket_returns_validation_error() {
    let body = serde_json::json!({
        "title": "t",
        "description": "te"
    });

    let response = ticket_management::app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/ticket")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body = String::from_utf8_lossy(&bytes);
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
}

#[tokio::test]
async fn wrong_path_returns_404() {
    let response = ticket_management::app()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/wrongpath")
                .header("content-type", "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body = String::from_utf8_lossy(&bytes);
    assert_eq!(status, StatusCode::NOT_FOUND, "body: {body}");
}

#[tokio::test]
async fn get_without_create_returns_404() {
    let (client, address) = common::setup().await;
    let request_url = format!("{address}/ticket/1");
    let response = client.get(request_url).send().await.unwrap();
    let status = response.status();
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn full_roundtrip_returs_ticket() {
    let (client, address) = common::setup().await;
    let request_url = format!("{address}/ticket/0");
    let response = client.get(request_url).send().await.unwrap();
    let status = response.status();
    assert_eq!(status, StatusCode::NOT_FOUND);

    let body = serde_json::json!({
        "title": "title",
        "description": "description"
    });

    let request_url = format!("{address}/ticket");
    let response = client.post(request_url).json(&body).send().await.unwrap();
    let status = response.status();
    let id: u64 = response.json().await.unwrap();

    assert_eq!(status, StatusCode::CREATED);

    let request_url = format!("{address}/ticket/{id}");
    let response = client.get(request_url).send().await.unwrap();
    let status = response.status();
    assert_eq!(status, StatusCode::OK);
}
