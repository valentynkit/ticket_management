use reqwest::Client;
use tokio::net::TcpListener;

pub async fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, ticket_management::app())
            .await
            .unwrap();
    });
    format!("http://{addr}")
}

pub async fn setup() -> (Client, String) {
    (reqwest::Client::new(), spawn_app().await)
}
