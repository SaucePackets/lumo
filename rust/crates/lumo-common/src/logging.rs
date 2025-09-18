use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use std::sync::Once;

static INIT: Once = Once::new();

/// Setup logging for the application
pub fn setup_logging() -> eyre::Result<()> {
    INIT.call_once(|| {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_target(false)
                    .compact(),
            )
            .with(
                EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| EnvFilter::new("info")),
            )
            .try_init()
            .expect("Failed to initialize logging");
    });

    Ok(())
}

/// Setup logging with custom level
pub fn setup_logging_with_level(level: &str) -> eyre::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .compact(),
        )
        .with(EnvFilter::new(level))
        .try_init()?;

    Ok(())
}