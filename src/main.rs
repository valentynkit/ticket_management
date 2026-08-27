use config::Config;
use ticket_management::{AppConfig, AppError};
use tracing::{error, info};
use tracing_subscriber::{fmt::time, EnvFilter};

#[tokio::main]
async fn main() {
    logging_init();
    info!("logging_init");
    let config = AppConfig::load().unwrap();
    if let Err(err) = ticket_management::run(config).await {
        process_shutdown(err).await;
    }
}

async fn process_shutdown(err: AppError) {
    let err = err.inner().to_string();
    error!(err, "got the error");
}

fn logging_init() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ticket_management=debug,axum=debug".into()),
        )
        .with_timer(time::uptime())
        .init();
}
