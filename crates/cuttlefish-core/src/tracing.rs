//! Tracing and logging setup for the Cuttlefish platform.

use tracing_subscriber::{EnvFilter, fmt, prelude::*};

/// Initialize tracing and logging for the application.
///
/// Uses `RUST_LOG` environment variable for filter (default: "info").
/// Output format depends on `CUTTLEFISH_ENV`:
/// - "production" → JSON format
/// - otherwise → pretty/human-readable format
pub fn init_tracing() {
    let env = std::env::var("CUTTLEFISH_ENV").unwrap_or_else(|_| "development".to_string());
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    if env == "production" {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().json())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().pretty())
            .init();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_tracing_can_be_called() {
        init_tracing();
    }
}
