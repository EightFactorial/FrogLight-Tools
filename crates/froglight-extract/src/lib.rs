#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(feature = "cmd")]
pub mod cmd;
#[cfg(feature = "cmd")]
pub use cmd::{extract, logging, main};
