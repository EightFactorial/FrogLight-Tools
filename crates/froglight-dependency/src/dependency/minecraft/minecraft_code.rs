//! TODO

use std::{io::Read, path::Path};

use cafebabe::ClassFile;
use froglight_tool_macros::Dependency;
use hashbrown::HashMap;
use zip::ZipArchive;

use crate::{container::DependencyContainer, dependency::yarn::MappedJar, version::Version};

/// Parsed Minecraft code.
#[derive(Clone, Default, PartialEq, Eq, Dependency)]
#[dep(path = crate)]
pub struct MinecraftCode(HashMap<Version, CodeBundle>);

impl MinecraftCode {
    /// Get the [`CodeBundle`] for a given version.
    ///
    /// Returns `None` if the version is not yet known.
    #[must_use]
    pub fn version(&self, version: &Version) -> Option<&CodeBundle> { self.0.get(version) }

    /// Get the [`CodeBundle`] for a given version.
    ///
    /// # Errors
    /// Returns an error if there was an error getting the [`CodeBundle`].
    #[expect(clippy::missing_panics_doc)]
    pub async fn get_version(
        &mut self,
        version: &Version,
        deps: &mut DependencyContainer,
    ) -> anyhow::Result<&CodeBundle> {
        if !self.0.contains_key(version) {
            deps.get_or_retrieve::<MappedJar>().await?;
            deps.scoped_fut::<MappedJar, anyhow::Result<()>>(
                async |jar: &mut MappedJar, deps: &mut DependencyContainer| {
                    let client = jar.get_client(version, deps).await?;
                    self.0.insert(version.clone(), CodeBundle::build_from(client)?);
                    Ok(())
                },
            )
            .await
            .map_err(|err| anyhow::anyhow!("MinecraftCode: {err}"))?;
        }

        Ok(self.version(version).unwrap())
    }
}

/// Parsed Minecraft code for a specific version.
#[derive(Clone, PartialEq, Eq)]
pub struct CodeBundle(HashMap<String, Vec<u8>>);

impl CodeBundle {
    /// Build a [`CodeBundle`] from a decompiled jar.
    fn build_from(jar: &Path) -> anyhow::Result<Self> {
        tracing::debug!("Parsing \"{}\"", jar.display());

        let mut map = HashMap::new();
        let mut zip = ZipArchive::new(std::io::Cursor::new(std::fs::read(jar)?))?;

        for index in 0..zip.len() {
            if !zip.name_for_index(index).is_some_and(|n| {
                Path::new(n).extension().is_some_and(|ext| ext.eq_ignore_ascii_case("class"))
            }) {
                continue;
            }

            let mut file = zip.by_index(index)?;
            let mut file_buf = Vec::new();
            file.read_to_end(&mut file_buf)?;

            map.insert(file.name().to_string(), file_buf);
        }

        Ok(Self(map))
    }

    /// Get the [`ClassFile`] for a given class.
    #[must_use]
    #[expect(clippy::missing_panics_doc)]
    pub fn get(&self, class: &str) -> Option<ClassFile> {
        self.0.get(class).map(|data| cafebabe::parse_class(data).unwrap())
    }
}
