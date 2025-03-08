use std::{io::Read, sync::Arc};

use convert_case::{Case, Casing};
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

    /// Get the [`TranslationsFile`] for the given version.
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

impl TranslationsFile {
    /// Get the `identifier` of a block.
    ///
    /// Uses the translation if available,
    /// otherwise falls back to the input key.
    #[expect(clippy::missing_panics_doc)]
    pub fn block_name(&self, block: &str) -> String {
        if let Some(translation) = self.get(&format!("block.{}", block.replace(':', "."))) {
            translation.replace(['\''], "_").to_case(Case::Pascal)
        } else {
            if !block.contains("wall") {
                tracing::warn!("No translation found for block: \"{block}\"");
            }

            block.split(':').next_back().unwrap().to_case(Case::Pascal)
        }
    }

    /// Get the `identifier` of an item.
    ///
    /// Uses the translation if available,
    /// otherwise falls back to the input key.
    #[expect(clippy::missing_panics_doc)]
    pub fn item_name(&self, item: &str) -> String {
        let formatted_item = item.replace(':', ".");

        if let Some(translation) = self.get(&format!("item.{formatted_item}")) {
            if self.0.values().filter(|v| *v == translation).count() == 1 {
                // If the translation is unique, use it.
                return translation.replace(['\''], "").to_case(Case::Pascal);
            }
        } else if let Some(translation) = self.get(&format!("block.{formatted_item}")) {
            // If the item is a block, use the block translation.
            return translation.replace(['\''], "_").to_case(Case::Pascal);
        }

        tracing::warn!("No translation found for item: \"{item}\"");
        item.split(':').next_back().unwrap().to_case(Case::Pascal)
    }
}

impl std::ops::Deref for TranslationsFile {
    type Target = HashMap<String, String>;
    fn deref(&self) -> &Self::Target { &self.0 }
}
