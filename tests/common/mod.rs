use reqwest::Client;
use tokio::net::TcpListener;

/// Port 0 lets the OS pick a free port, so tests run in parallel and never collide
/// with a dev server. Binding happens here (synchronously) rather than inside the
/// spawned task — once `bind` returns, the socket is listening and the kernel
/// queues connections, so there is no race with `serve` being polled.
pub async fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(ticket_management::serve(listener));
    format!("http://{addr}")
}

pub async fn setup() -> (Client, String) {
    (reqwest::Client::new(), spawn_app().await)
}
