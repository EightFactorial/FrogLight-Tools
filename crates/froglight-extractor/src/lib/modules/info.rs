use serde::{Deserialize, Serialize};

/// A module that extracts the version's `version.json` file.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct InfoModule;
