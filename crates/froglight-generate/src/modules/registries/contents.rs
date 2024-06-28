use std::path::Path;

use froglight_extract::bundle::ExtractBundle;
use tracing::warn;

use super::Registries;
use crate::bundle::GenerateBundle;

impl Registries {
    pub(super) async fn create_registries_contents(
        reg_path: &Path,
        generate: &GenerateBundle<'_>,
        extract: &ExtractBundle<'_>,
    ) -> anyhow::Result<()> {
        // Create the generated registries
        let gen_path = reg_path.join("generated");
        if !gen_path.exists() {
            warn!("Creating generated directory: \"{}\"", gen_path.display());
            tokio::fs::create_dir(&gen_path).await?;
        }
        super::generated::create_generated(&gen_path, generate, extract).await?;

        // Create versioned implementations of the registries
        let ver_path = reg_path.join(generate.version.base.to_long_string());
        if !ver_path.exists() {
            warn!("Creating registry version directory: \"{}\"", ver_path.display());
            tokio::fs::create_dir(&ver_path).await?;
        }
        super::version::create_versioned(&ver_path, generate, extract).await
    }
}
