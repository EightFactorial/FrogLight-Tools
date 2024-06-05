//! Logging utilities.

use clap_verbosity_flag::{Level as ClapLevel, Verbosity};
use tracing::Level as TracingLevel;

/// Setup [`tracing`] with the given [`Verbosity`].
pub fn setup(verbose: &Verbosity) {
    let Some(clap_level) = verbose.log_level() else {
        // Logging disabled
        return;
    };

    let trace_level = match clap_level {
        ClapLevel::Error => TracingLevel::ERROR,
        ClapLevel::Warn => TracingLevel::WARN,
        ClapLevel::Info => TracingLevel::INFO,
        ClapLevel::Debug => TracingLevel::DEBUG,
        ClapLevel::Trace => TracingLevel::TRACE,
    };

    // Configure the tracing subscriber
    match clap_level {
        ClapLevel::Error | ClapLevel::Warn | ClapLevel::Info => {
            tracing_subscriber::fmt().with_max_level(trace_level).without_time().init();
        }
        ClapLevel::Debug | ClapLevel::Trace => {
            tracing_subscriber::fmt().with_max_level(trace_level).init();
        }
    }
}
