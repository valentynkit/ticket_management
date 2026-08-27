use ticket_management::AppConfig;
use tracing::{error, info};
use tracing_subscriber::{fmt::time, EnvFilter};

#[tokio::main]
async fn main() {
    // Config comes first: it carries the log filter, so tracing cannot be up yet.
    // Failures go to stderr and kill the process — a misconfigured server must
    // refuse to start, not fail on request 400.
    let config = match AppConfig::load() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("configuration error: {err}");
            let mut source = std::error::Error::source(&err);
            while let Some(cause) = source {
                eprintln!("  caused by: {cause}");
                source = cause.source();
            }
            std::process::exit(1);
        }
    };

    logging_init(config.log_filter());
    info!(environment = config.environment().as_str(), "starting");

    if let Err(err) = ticket_management::run(&config).await {
        // `{:#}` renders the whole anyhow context chain on one line.
        error!(error = format!("{err:#}"), "server exited with an error");
        std::process::exit(1);
    }
}

fn logging_init(default_filter: &str) {
    tracing_subscriber::fmt()
        .with_env_filter(
            // RUST_LOG still wins, so you can raise verbosity without editing config.
            EnvFilter::try_from_default_env().unwrap_or_else(|_| default_filter.into()),
        )
        .with_timer(time::uptime())
        .init();
}
