#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

mod version;
pub use version::MinecraftVersion;

pub mod manifests;
