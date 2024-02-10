use std::sync::Arc;

use super::{util::package_path, DataBundle, Generate};

mod packets;
mod states;
mod versions;

/// A module that generates protocol data.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProtocolModule;

impl Generate for ProtocolModule {
    async fn generate(&self, bundle: Arc<DataBundle>) -> anyhow::Result<()> {
        let mut path = package_path("froglight-protocol", &bundle.workspace)?.to_path_buf();
        path.push("src");

        versions::generate(&path, &bundle).await
    }
}
