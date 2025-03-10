#![expect(unused_imports)]

use std::{collections::HashSet, fmt::Write, path::Path, sync::Once};

use convert_case::{Case, Casing};
use froglight_dependency::{
    container::DependencyContainer,
    dependency::{
        client::{Translations, TranslationsFile},
        minecraft::DataGenerator,
        vineflower::DecompiledJar,
    },
    version::Version,
};
use froglight_extract::module::ExtractModule;
use tokio::sync::OnceCell;
use tracing::Level;

mod report;
pub(crate) use report::{ItemReport, ItemReports};

use super::ToolConfig;
use crate::module::block::BlockReports;

#[derive(ExtractModule)]
#[module(function = Items::generate)]
pub(crate) struct Items;

impl Items {
    async fn generate(version: &Version, deps: &mut DependencyContainer) -> anyhow::Result<()> {
        let mut directory = std::env::current_dir()?;
        directory.push("crates/froglight-item");

        if !tokio::fs::try_exists(&directory).await? {
            anyhow::bail!("Could not find \"froglight-item\" at \"{}\"", directory.display());
        }

        Self::generate_items(deps, &directory).await?;

        Self::generate_item_traits(version, deps, &directory).await?;

        Ok(())
    }
}

impl Items {
    /// Generate item unit structs.
    async fn generate_items(deps: &mut DependencyContainer, path: &Path) -> anyhow::Result<()> {
        static ONCE: OnceCell<anyhow::Result<()>> = OnceCell::const_new();
        ONCE.get_or_init(async || {
            deps.get_or_retrieve::<Translations>().await?;
            deps.get_or_retrieve::<ItemReports>().await?;
            deps.get_or_retrieve::<BlockReports>().await?;

            let mut sorted = Vec::<String>::new();
            let mut blocks = HashSet::<String>::new();

            for version in deps.get::<ToolConfig>().unwrap().versions.clone() {
                // Get the translations for proper item names.
                let translations = deps
                    .scoped_fut::<Translations, anyhow::Result<TranslationsFile>>(
                        async |translations: &mut Translations, deps| {
                            translations.get_version(&version, deps).await.cloned()
                        },
                    )
                    .await?;

                deps.scoped_fut::<BlockReports, anyhow::Result<()>>(
                    async |reports: &mut BlockReports, deps| {
                        let report = reports.get_version(&version, deps).await?;
                        for item in report.0.keys() {
                            blocks.insert(translations.block_name(item));
                        }
                        Ok(())
                    },
                )
                .await?;

                deps.scoped_fut::<ItemReports, anyhow::Result<()>>(
                    async |reports: &mut ItemReports, deps| {
                        let report = reports.get_version(&version, deps).await?;
                        for item in report.0.keys() {
                            sorted.push(translations.item_name(item));
                        }
                        Ok(())
                    },
                )
                .await?;
            }

            sorted.sort_unstable();
            sorted.dedup();

            let path = path.join("src/generated/item.rs");
            let blocks: String = sorted.into_iter().fold(String::new(), |mut acc, item| {
                if blocks.contains(&item) {
                    acc.push_str("    #[block]\n");
                }

                acc.push_str("    pub struct ");
                acc.push_str(&item);
                acc.push_str(";\n");
                acc
            });

            tokio::fs::write(
                path,
                format!(
                    r"//! This file is generated, do not modify it manually.
//!
//! TODO: Documentation
#![allow(missing_docs)]

froglight_macros::items! {{
    crate,
{blocks}}}
"
                ),
            )
            .await?;

            Ok(())
        })
        .await
        .as_ref()
        .map_or_else(|e| Err(anyhow::anyhow!(e)), |()| Ok(()))
    }
}

impl Items {
    const ITEM_NAME_PADDING: usize = 36;

    /// Generate item trait implementations.
    async fn generate_item_traits(
        version: &Version,
        deps: &mut DependencyContainer,
        path: &Path,
    ) -> anyhow::Result<()> {
        let version_ident =
            format!("froglight_common::version::V{}", version.to_long_string().replace('.', "_"));

        let report = deps.get::<ItemReports>().unwrap();
        let report = report.version(version).unwrap();

        let translations = deps.get::<Translations>().unwrap();
        let translations = translations.version(version).unwrap();

        let implementations =
            report.0.iter().enumerate().fold(String::new(), |mut acc, (i, (item, entry))| {
                let item_name = translations.item_name(item);

                acc.push_str("    ");
                acc.push_str(&item_name);

                // Pad the item name with spaces.
                for _ in 0..Self::ITEM_NAME_PADDING.saturating_sub(item_name.len()) {
                    acc.push(' ');
                }

                // Add the item properties.
                let item_properties = format!(
                    " => {{ properties: {{ ident: \"{item}\", rarity: ItemRarity::{:?} }}",
                    entry.rarity
                );
                acc.push_str(&item_properties);

                acc.push_str(" }");
                if i < report.0.len() {
                    acc.push(',');
                }
                acc.push('\n');

                acc
            });

        let path = path.join(format!(
            "src/generated/v{}/property.rs",
            version.to_long_string().replace('.', "_")
        ));
        tokio::fs::write(
            path,
            format!(
                r"//! This file is generated, do not modify it manually.
//!
//! TODO: Documentation
#![allow(missing_docs)]

#[allow(clippy::wildcard_imports)]
use crate::{{generated::item::*, item::ItemRarity}};

froglight_macros::item_properties! {{
    path = crate,
    version = {version_ident},
{implementations}}}
",
            ),
        )
        .await?;

        Ok(())
    }
}
