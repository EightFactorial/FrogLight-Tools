use serde::{Deserialize, Serialize};

/// A module that appends debug information to the output.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct DebugModule;
