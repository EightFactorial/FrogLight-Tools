//! Logging utilities.

/// Initialize logging with the default environment filter.
pub fn init() {
    let filter = tracing_subscriber::EnvFilter::from_default_env();
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}
