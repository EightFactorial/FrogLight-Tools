#![doc = include_str!("../README.md")]

use clap::Parser;

mod command;
use command::GeneratorArgs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _args = GeneratorArgs::parse();

    Ok(())
}
