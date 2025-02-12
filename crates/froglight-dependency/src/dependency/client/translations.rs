use std::{io::Read, sync::Arc};

use froglight_tool_macros::Dependency;
use hashbrown::HashMap;
use zip::ZipArchive;

use crate::{
    container::DependencyContainer, dependency::minecraft::MinecraftJar, version::Version,
};

/// A collection of [`TranslationsFile`]s.
#[derive(Debug, Default, Clone, PartialEq, Eq, Dependency)]
#[dep(path = crate)]
pub struct Translations(HashMap<Version, TranslationsFile>);

impl Translations {
    /// Get the [`TranslationsFile`] for the given version.
    ///
    /// Returns `None` if the translations are not yet known.
    #[must_use]
    pub fn version(&self, version: &Version) -> Option<&TranslationsFile> { self.0.get(version) }

    /// Get the [`TransaltionsFile`] for the given version.
    ///
    /// # Errors
    /// Returns an error if there was an error retrieving the data.
    #[expect(clippy::missing_panics_doc)]
    pub async fn get_version(
        &mut self,
        version: &Version,
        deps: &mut DependencyContainer,
    ) -> anyhow::Result<&TranslationsFile> {
        if !self.0.contains_key(version) {
            deps.get_or_retrieve::<MinecraftJar>().await?;
            deps.scoped_fut::<MinecraftJar, anyhow::Result<()>>(
                async |jar: &mut MinecraftJar, deps: &mut DependencyContainer| {
                    let client = jar.get_client(version, deps).await?;

                    let mut zip = ZipArchive::new(std::fs::File::open(client)?)?;
                    match zip.by_name("assets/minecraft/lang/en_us.json") {
                        Err(err) => anyhow::bail!("Translations: {err}"),
                        Ok(mut file) => {
                            let mut data = String::new();
                            file.read_to_string(&mut data)?;
                            let translations = serde_json::from_str(&data)?;

                            self.0
                                .insert(version.clone(), TranslationsFile(Arc::new(translations)));
                        }
                    }

                    Ok(())
                },
            )
            .await
            .map_err(|err| anyhow::anyhow!("Translations: {err}"))?;
        }

        Ok(self.version(version).unwrap())
    }
}

/// A file containing translations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslationsFile(Arc<HashMap<String, String>>);

impl std::ops::Deref for TranslationsFile {
    type Target = HashMap<String, String>;
    fn deref(&self) -> &Self::Target { &self.0 }
}
