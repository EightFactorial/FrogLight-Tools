#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub use inventory;

#[cfg(feature = "cmd")]
pub mod cmd;
#[cfg(feature = "cmd")]
pub use cmd::main;

pub mod module;
pub use module::extract;

pub mod json;
