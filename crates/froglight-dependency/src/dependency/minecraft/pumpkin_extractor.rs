//! TODO

use std::{
    io::{Cursor, Read},
    path::{Path, PathBuf},
};

use froglight_tool_macros::Dependency;
use hashbrown::HashMap;
use tokio::{fs::File, process::Command};
use zip::ZipArchive;

use crate::{
    container::DependencyContainer,
    dependency::yarn::{FabricMaven, YarnMaven},
    version::Version,
};

/// Data extracted using Pumpkin's Extractor.
#[derive(Debug, Default, Clone, PartialEq, Eq, Dependency)]
#[dep(path = crate)]
pub struct PumpkinExtractor {
    versions: HashMap<Version, PathBuf>,
}

impl PumpkinExtractor {
    /// Get the [`Path`] for the given version data.
    ///
    /// Returns `None` if no data has been extracted yet.
    #[must_use]
    pub fn version(&self, version: &Version) -> Option<&Path> {
        self.versions.get(version).map(|v| &**v)
    }

    /// Get the [`Path`] for the given version.
    ///
    /// # Errors
    /// Returns an error if there was an error extracting the data.
    #[expect(clippy::missing_panics_doc)]
    pub async fn get_version(
        &mut self,
        version: &Version,
        deps: &mut DependencyContainer,
    ) -> anyhow::Result<&Path> {
        if !self.versions.contains_key(version) {
            Self::extract_version(version, deps)
                .await
                .map_err(|err| anyhow::anyhow!("Pumpkin Extractor: {err}"))?;
        }

        Ok(self.version(version).unwrap())
    }
}

impl PumpkinExtractor {
    /// The directory to store all pumpkin files.
    pub const CACHE_DIR: &str = "pumpkin";
    /// The extractor repository zip file.
    pub const REPOSITORY: &str =
        "https://github.com/Pumpkin-MC/Extractor/archive/refs/heads/master.zip";

    async fn extract_version(
        version: &Version,
        deps: &mut DependencyContainer,
    ) -> anyhow::Result<()> {
        let version_str = version.to_short_string();
        let cache = deps.cache.join(Self::CACHE_DIR);

        if !tokio::fs::try_exists(&cache).await? {
            tokio::fs::create_dir_all(&cache).await?;

            // Download and open the repository ZIP
            let response = deps.client.get(Self::REPOSITORY).send().await?.bytes().await?;
            let mut zip = ZipArchive::new(Cursor::new(response))?;

            // Extract all files into the cache directory
            let mut buffer = Vec::with_capacity(1024);
            for index in 0..zip.len() {
                let mut entry = zip.by_index(index)?;
                let path = cache
                    .join(format!("./{}", entry.name().trim_start_matches("Extractor-master")));

                if entry.is_file() {
                    entry.read_to_end(&mut buffer)?;
                    tokio::fs::write(path, &mut buffer).await?;
                } else if entry.is_dir() {
                    tokio::fs::create_dir_all(path).await?;
                }
                buffer.clear();
            }

            // Make sure that `gradlew` is executable
            let file = File::open(cache.join("gradlew")).await?;

            #[cfg(target_family = "unix")]
            {
                use std::os::unix::fs::PermissionsExt;

                let mut perms = file.metadata().await?.permissions();
                perms.set_mode(0o744);

                file.set_permissions(perms).await?;
            }
        }

        // Retrieve the yarn and fabric-api versions.
        let Some(yarn) = deps.get_or_retrieve::<YarnMaven>().await?.get_build(version) else {
            anyhow::bail!("No Yarn version found for {version}");
        };
        let Some(fabric) = deps.get_or_retrieve::<FabricMaven>().await?.get_build(version) else {
            anyhow::bail!("No Fabric-API version found for {version}");
        };

        // Fill the gradle template
        let mut gradle = Self::GRADLE_TEMPLATE.replace("{MINECRAFT_VER}", &version_str);
        gradle = gradle.replace("{YARN_VER}", &yarn).replace("{FABRIC_VER}", &fabric);
        // Write the template file
        tokio::fs::write(cache.join("gradle.properties"), gradle.into_bytes()).await?;

        // Run the extractor
        let process =
            Command::new("./gradlew").arg("runServer").current_dir(&cache).output().await?;

        if process.status.success() {
            // Cache the generated output

            let _ver_cache = deps.cache.join(version_str).join(Self::CACHE_DIR);
            // tokio::fs::rename(cache.join("pumpkin_extractor_output"), ver_cache).await?;

            Ok(())
        } else {
            // Delete the partial output

            let stdout = String::from_utf8_lossy(&process.stdout);
            let stderr = String::from_utf8_lossy(&process.stderr);
            Err(anyhow::anyhow!("DataGenerator failed:\n{stderr}\n{stdout}"))
        }
    }

    const GRADLE_TEMPLATE: &str = "# Done to increase the memory available to gradle.
org.gradle.jvmargs=-Xmx1G
org.gradle.parallel=true
# Fabric Properties
# check these on https://modmuss50.me/fabric.html
minecraft_version={MINECRAFT_VER}
yarn_mappings={YARN_VER}
loader_version=0.16.10
kotlin_loader_version=1.13.2+kotlin.2.1.20
# Mod Properties
mod_version=1.0-SNAPSHOT
maven_group=de.snowii
archives_base_name=extractor
fabric_version={FABRIC_VER}
";
}
