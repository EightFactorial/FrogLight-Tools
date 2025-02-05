use std::path::{Path, PathBuf};

use froglight_tool_macros::Dependency;
use tokio::process::Command;

use crate::container::DependencyContainer;

/// The `TinyRemapper` jar remapping tool.
///
/// See [`https://github.com/FabricMC/tiny-remapper`][0]
///
/// [0]: https://github.com/FabricMC/tiny-remapper
#[derive(Debug, Clone, PartialEq, Eq, Dependency)]
#[dep(path = crate, retrieve = Self::retrieve)]
pub struct TinyRemapper(PathBuf);

impl TinyRemapper {
    const FILENAME: &'static str = "tiny-remapper-fat.jar";
    const URL: &'static str =
        "https://maven.fabricmc.net/net/fabricmc/tiny-remapper/0.11.0/tiny-remapper-0.11.0.jar";

    async fn retrieve(deps: &mut DependencyContainer) -> anyhow::Result<Self> {
        let path = deps.cache.join(Self::FILENAME);
        if tokio::fs::try_exists(&path).await? {
            tracing::debug!("Using \"{}\"", path.display());
        } else {
            tracing::debug!("Retrieving \"{}\"", Self::URL);

            // Download the file and save it to disk
            let response = deps.client.get(Self::URL).send().await?.bytes().await?;
            tokio::fs::write(&path, response).await?;
        }

        Ok(Self(path))
    }

    /// Remap a jar file using the given mappings.
    ///
    /// # Errors
    /// Returns an error if the remapping fails.
    pub async fn remap_jar(
        &self,
        jar: &Path,
        output: &Path,
        // mappings: &YarnMapping,
    ) -> anyhow::Result<()> {
        if tokio::fs::try_exists(output).await? {
            tracing::debug!("Using \"{}\"", output.display());
            Ok(())
        } else {
            tracing::debug!("Remapping \"{}\"", jar.display());

            let result = Command::new("java")
                .arg("-jar")
                .arg(&self.0)
                .arg(jar)
                .arg(output)
                // .arg(mappings)
                .arg("official")
                .arg("named")
                .output()
                .await?;

            if result.status.success() {
                Ok(())
            } else {
                let stdout = String::from_utf8_lossy(&result.stdout);
                let stderr = String::from_utf8_lossy(&result.stderr);
                Err(anyhow::anyhow!("TinyRemapper failed:\n{stderr}\n{stdout}"))
            }
        }
    }
}
