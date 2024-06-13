#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![feature(iterator_try_collect)]

pub mod bundle;
pub mod bytecode;
pub(crate) mod consts;
pub mod sources;
