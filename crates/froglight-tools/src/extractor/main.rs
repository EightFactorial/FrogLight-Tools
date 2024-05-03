#![doc = include_str!("README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

use clap::Parser;
use clap_verbosity_flag::Verbosity;
use tracing::debug;

mod cli;

#[tokio::main]
async fn main() {
    let args = cli::ExtractArguments::parse();

    setup_tracing(&args.verbose);
    debug!("Arguments: {:?}", args);
}

/// Setup tracing with the given verbosity.
fn setup_tracing(verbose: &Verbosity) {
    tracing_subscriber::fmt()
        .with_max_level(verbose.log_level().map(|l| match l {
            clap_verbosity_flag::Level::Error => tracing::Level::ERROR,
            clap_verbosity_flag::Level::Warn => tracing::Level::WARN,
            clap_verbosity_flag::Level::Info => tracing::Level::INFO,
            clap_verbosity_flag::Level::Debug => tracing::Level::DEBUG,
            clap_verbosity_flag::Level::Trace => tracing::Level::TRACE,
        }))
        .init();
}
