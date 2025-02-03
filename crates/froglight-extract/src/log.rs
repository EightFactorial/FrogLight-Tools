//! Logging utilities.

/// Initialize logging with the default environment filter.
///
/// See [`EnvFilter::from_default_env`](tracing_subscriber::EnvFilter::from_default_env)
/// for more information.
pub fn init() {
    let filter = tracing_subscriber::EnvFilter::from_default_env();
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}
