//! TODO
#![feature(try_trait_v2)]

mod cli;
pub use cli::CliArgs;

mod datamap;
pub use datamap::DataMap;

mod config;
pub use config::{Config, VersionTuple};

mod modules;
pub use modules::{BlockGenerator, PacketGenerator, RegistryGenerator};
