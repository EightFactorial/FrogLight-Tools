use std::path::{Path, PathBuf};

use froglight_tool_macros::Dependency;
use tokio::process::Command;

use crate::container::DependencyContainer;

/// The `Vineflower` decompiler
///
/// See [`https://github.com/Vineflower/vineflower`][0]
///
/// [0]: https://github.com/Vineflower/vineflower
#[derive(Debug, Clone, PartialEq, Eq, Dependency)]
#[dep(path = crate, retrieve = Self::retrieve)]
pub struct Vineflower(PathBuf);

impl Vineflower {
    const FILENAME: &'static str = "vineflower.jar";
    const URL: &'static str =
        "https://github.com/Vineflower/vineflower/releases/download/1.10.1/vineflower-1.10.1.jar";

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

    /// Decompile a jar file using Vineflower.
    ///
    /// # Errors
    /// Returns an error if the decompiling fails.
    pub async fn decompile_jar(&self, jar: &Path, output: &Path) -> anyhow::Result<()> {
        if tokio::fs::try_exists(output).await? {
            tracing::debug!("Using \"{}\"", output.display());
            Ok(())
        } else {
            tracing::debug!("Decompiling \"{}\"", jar.display());
            tokio::fs::create_dir(&output).await?;

            let process =
                Command::new("java").arg("-jar").arg(&self.0).arg(jar).arg(output).output().await?;

            if process.status.success() {
                Ok(())
            } else {
                let stdout = String::from_utf8_lossy(&process.stdout);
                let stderr = String::from_utf8_lossy(&process.stderr);
                Err(anyhow::anyhow!("Vineflower failed:\n{stderr}\n{stdout}"))
            }
        }
    }
}
