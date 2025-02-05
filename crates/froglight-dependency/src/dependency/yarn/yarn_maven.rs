use froglight_tool_macros::Dependency;
use serde::{Deserialize, Serialize};

use crate::container::DependencyContainer;

/// The yarn maven repository.
///
/// Contains information on all yarn builds.
#[derive(Debug, Clone, PartialEq, Eq, Dependency, Serialize, Deserialize)]
#[dep(path = crate, retrieve = Self::retrieve)]
pub struct YarnMaven();

impl YarnMaven {
    #[expect(clippy::unused_async)]
    async fn retrieve(_deps: &mut DependencyContainer) -> anyhow::Result<Self> { todo!() }
}
