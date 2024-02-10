use std::sync::Arc;

use super::{DataBundle, Generate};

/// A module that generates network data.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NetworkModule;

impl Generate for NetworkModule {
    async fn generate(&self, _bundle: Arc<DataBundle>) -> anyhow::Result<()> { Ok(()) }
}
