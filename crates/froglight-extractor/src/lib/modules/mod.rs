//! Modules for extracting data from a source.

use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

mod info;
pub use info::InfoModule;

mod debug;
pub use debug::DebugModule;

/// A module to use for extracting data.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, EnumString,
)]
#[serde(rename_all = "lowercase")]
#[allow(missing_docs)]
pub enum ExtractModule {
    Debug(DebugModule),
    Info(InfoModule),
}
