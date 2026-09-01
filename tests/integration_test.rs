use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use uuid::Uuid;

mod common;

#[tokio::test]
async fn create_ticket_returns_201() {
    let body = serde_json::json!({
        "title": "test title",
        "description": "test description"
    });

    let response = common::app()
        .await
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

    let response = common::app()
        .await
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

    let response = common::app()
        .await
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
    let response = common::app()
        .await
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
    let request_url = format!("{address}/ticket/{}", Uuid::nil());
    let response = client.get(request_url).send().await.unwrap();
    let status = response.status();
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn full_roundtrip_returs_ticket() {
    let (client, address) = common::setup().await;
    let request_url = format!("{address}/ticket/{}", Uuid::nil());
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
    let id: Uuid = response.json().await.unwrap();

    assert_eq!(status, StatusCode::CREATED);

    let request_url = format!("{address}/ticket/{id}");
    let response = client.get(request_url).send().await.unwrap();
    let status = response.status();
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn get_missing_ticket_returns_typed_error_body() {
    let (client, address) = common::setup().await;
    let missing = Uuid::nil();
    let response = client
        .get(format!("{address}/ticket/{missing}"))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["error"], format!("ticket {missing} not found"));
}

#[tokio::test]
async fn patch_missing_ticket_returns_404() {
    let (client, address) = common::setup().await;
    let response = client
        .patch(format!("{address}/ticket/{}", Uuid::nil()))
        .json(&serde_json::json!({ "title": "renamed" }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn patch_updates_ticket_and_returns_204() {
    let (client, address) = common::setup().await;

    let response = client
        .post(format!("{address}/ticket"))
        .json(&serde_json::json!({ "title": "before", "description": "description" }))
        .send()
        .await
        .unwrap();
    let id: Uuid = response.json().await.unwrap();

    let response = client
        .patch(format!("{address}/ticket/{id}"))
        .json(&serde_json::json!({ "title": "after", "status": "in_progress" }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let ticket: serde_json::Value = client
        .get(format!("{address}/ticket/{id}"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(ticket["title"], "after");
    assert_eq!(ticket["status"], "in_progress");
    assert_eq!(ticket["description"], "description");
}

#[tokio::test]
async fn delete_removes_the_ticket() {
    let (client, address) = common::setup().await;

    let id: Uuid = client
        .post(format!("{address}/ticket"))
        .json(&serde_json::json!({ "title": "doomed", "description": "description" }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let response = client
        .delete(format!("{address}/ticket/{id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let response = client
        .get(format!("{address}/ticket/{id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_missing_ticket_returns_404() {
    let (client, address) = common::setup().await;
    let response = client
        .delete(format!("{address}/ticket/{}", Uuid::nil()))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_is_empty_on_a_fresh_database() {
    let (client, address) = common::setup().await;
    let page: serde_json::Value = client
        .get(format!("{address}/ticket"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(page["items"].as_array().unwrap().len(), 0);
    assert!(page["next_cursor"].is_null());
}

#[tokio::test]
async fn list_pages_through_every_ticket_in_creation_order() {
    let (client, address) = common::setup().await;

    for n in 0..5 {
        let response = client
            .post(format!("{address}/ticket"))
            .json(&serde_json::json!({
                "title": format!("ticket {n}"),
                "description": format!("description {n}")
            }))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    let mut seen: Vec<String> = Vec::new();
    let mut cursor: Option<String> = None;

    loop {
        let url = match &cursor {
            Some(after) => format!("{address}/ticket?limit=2&after={after}"),
            None => format!("{address}/ticket?limit=2"),
        };
        let page: serde_json::Value = client.get(url).send().await.unwrap().json().await.unwrap();

        for item in page["items"].as_array().unwrap() {
            seen.push(item["title"].as_str().unwrap().to_owned());
        }

        match page["next_cursor"].as_str() {
            Some(next) => cursor = Some(next.to_owned()),
            None => break,
        }
    }

    // uuidv7 is time-ordered, so keyset order is creation order.
    assert_eq!(
        seen,
        vec!["ticket 0", "ticket 1", "ticket 2", "ticket 3", "ticket 4"]
    );
}

#[tokio::test]
async fn list_limit_is_clamped() {
    let (client, address) = common::setup().await;

    for n in 0..3 {
        client
            .post(format!("{address}/ticket"))
            .json(&serde_json::json!({
                "title": format!("ticket {n}"),
                "description": format!("description {n}")
            }))
            .send()
            .await
            .unwrap();
    }

    let page: serde_json::Value = client
        .get(format!("{address}/ticket?limit=99999"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(page["items"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn health_reports_the_database() {
    let (client, address) = common::setup().await;
    let response = client
        .get(format!("{address}/health"))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
