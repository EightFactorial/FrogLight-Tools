//! TODO

use std::path::{Path, PathBuf};

use froglight_tool_macros::Dependency;
use hashbrown::HashMap;
use tokio::process::Command;

use super::MinecraftJar;
use crate::{container::DependencyContainer, version::Version};

/// Paths to Minecraft's built-in data generators.
#[derive(Debug, Default, Clone, PartialEq, Eq, Dependency)]
#[dep(path = crate)]
pub struct DataGenerator(HashMap<Version, PathBuf>);

impl DataGenerator {
    /// Get the [`Path`] of the data generator for the given version.
    ///
    /// Returns `None` if the path is not yet known.
    #[must_use]
    pub fn version(&self, version: &Version) -> Option<&Path> {
        self.0.get(version).map(PathBuf::as_path)
    }

    /// Get the [`Path`] of the data generator for the given version.
    ///
    /// # Errors
    /// Returns an error if there was an error generating the data.
    #[expect(clippy::missing_panics_doc)]
    pub async fn get_version(
        &mut self,
        version: &Version,
        deps: &mut DependencyContainer,
    ) -> anyhow::Result<&Path> {
        if !self.0.contains_key(version) {
            deps.get_or_retrieve::<MinecraftJar>().await?;
            deps.scoped_fut::<MinecraftJar, anyhow::Result<()>>(
                async |jar: &mut MinecraftJar, deps: &mut DependencyContainer| {
                    let server = jar.get_server(version, deps).await?;
                    self.0.insert(version.clone(), Self::run_generator(server, &deps.cache).await?);
                    Ok(())
                },
            )
            .await?;
        }

        Ok(self.version(version).unwrap())
    }
}

impl DataGenerator {
    const GENERATOR_CACHE: &str = "generator-cache";

    async fn run_generator(jar: &Path, cache: &Path) -> anyhow::Result<PathBuf> {
        let out = jar.with_file_name(format!(
            "{}-generated",
            jar.file_name().unwrap().to_string_lossy().split_once('.').unwrap().0
        ));

        if tokio::fs::try_exists(&out).await? {
            tracing::debug!("Using \"{}\"", out.display());
        } else {
            tracing::debug!("Generating \"{}\"", jar.display());
            tokio::fs::create_dir_all(&out).await?;

            let cache = cache.join(Self::GENERATOR_CACHE);
            if !tokio::fs::try_exists(&cache).await? {
                tokio::fs::create_dir_all(&cache).await?;
            }

            let process = Command::new("java")
                .arg("-DbundlerMainClass=net.minecraft.data.Main")
                .arg("-jar")
                .arg(jar)
                .arg("--output")
                .arg(&out)
                .arg("--all")
                .current_dir(cache)
                .output()
                .await?;

            if !process.status.success() {
                let stdout = String::from_utf8_lossy(&process.stdout);
                let stderr = String::from_utf8_lossy(&process.stderr);
                anyhow::bail!("DataGenerator failed:\n{stderr}\n{stdout}");
            }
        }

        Ok(out)
    }
}
