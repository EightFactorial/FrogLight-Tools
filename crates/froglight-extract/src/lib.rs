#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(feature = "cmd")]
pub mod cmd;
#[cfg(all(feature = "cmd", feature = "logging"))]
pub use cmd::logging;
#[cfg(feature = "cmd")]
pub use cmd::{extract, main};

pub mod module;
