#![doc = include_str!("README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

use clap::Parser;
use tracing::{debug, info};

mod cli;
use cli::ExtractArguments;

mod logging;

#[tokio::main]
async fn main() {
    let args = ExtractArguments::parse();

    logging::setup(&args.verbose);
    info!("Version: {}", args.version);
    debug!("Modules: {:?}", args.modules);
}
