use std::path::Path;

use froglight_dependency::container::DependencyContainer;
use tokio::sync::OnceCell;

use super::Packets;
use crate::{module::packet::VersionCodecs, ToolConfig};

mod module;

impl Packets {
    pub(super) async fn generate_packets(
        deps: &mut DependencyContainer,
        path: &Path,
    ) -> anyhow::Result<()> {
        static ONCE: OnceCell<anyhow::Result<()>> = OnceCell::const_new();
        ONCE.get_or_init(async || {
            for version in deps.get::<ToolConfig>().unwrap().versions.clone() {
                let version_dir =
                    path.join(format!("v{}", version.to_long_string().replace('.', "_")));
                // Create the version directory if it does not exist
                if !tokio::fs::try_exists(&version_dir).await? {
                    tokio::fs::create_dir_all(&version_dir).await?;
                }

                // Generate the `mod.rs` file for the version module
                Self::generate_version_module(&version, &version_dir).await?;

                let codecs = deps.get_or_retrieve::<VersionCodecs>().await?.clone();
                let ver_codecs = codecs.version(&version).unwrap();

                // Generate the `mod.rs` and packet files for the state
                for (state, _packets) in ver_codecs.iter() {
                    let state_dir = version_dir.join(state.to_string().to_lowercase());
                    Self::generate_state_module(state, ver_codecs, &version, deps, &state_dir)
                        .await?;
                }
            }

            Ok(())
        })
        .await
        .as_ref()
        .map_or_else(|e| Err(anyhow::anyhow!(e)), |()| Ok(()))
    }
}
