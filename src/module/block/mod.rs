#![expect(unused_imports)]

use std::{fmt::Write, path::Path, sync::Once};

use attribute::{BlockAttributeData, ParsedBlockReport};
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

mod attribute;
pub(crate) use attribute::{BlockAttributes, BlockReports};

use super::ToolConfig;

mod property;
// pub(crate) use property::BlockProperties;

#[derive(ExtractModule)]
#[module(function = Blocks::generate)]
pub(crate) struct Blocks;

impl Blocks {
    async fn generate(version: &Version, deps: &mut DependencyContainer) -> anyhow::Result<()> {
        let mut directory = std::env::current_dir()?;
        directory.push("crates/froglight-block");

        if !tokio::fs::try_exists(&directory).await? {
            anyhow::bail!("Could not find \"froglight-block\" at \"{}\"", directory.display());
        }

        Self::generate_attributes(deps, &directory).await?;
        Self::generate_blocks(deps, &directory).await?;

        Self::generate_block_traits(version, deps, &directory).await?;

        Ok(())
    }
}

impl Blocks {
    /// Generate block attribute enums.
    async fn generate_attributes(
        deps: &mut DependencyContainer,
        path: &Path,
    ) -> anyhow::Result<()> {
        static ONCE: OnceCell<anyhow::Result<()>> = OnceCell::const_new();
        ONCE.get_or_init(async || {
            let attrs = deps.get_or_retrieve::<BlockAttributes>().await?;

            let mut sorted: Vec<_> = attrs.0.iter().collect();
            sorted.sort_unstable_by(|a, b| match a.name.cmp(&b.name) {
                std::cmp::Ordering::Equal => a.values.cmp(&b.values),
                other => other,
            });

            let path = path.join("src/generated/attribute.rs");
            let attributes: String = sorted.into_iter().fold(String::new(), |mut acc, attr| {
                acc.push_str("    ");
                acc.push_str(&attrs.as_enum_macro(attr));
                acc.push_str(",\n");
                acc
            });

            tokio::fs::write(
                path,
                format!(
                    r"//! This file is generated, do not modify it manually.
//!
//! TODO: Documentation
#![allow(missing_docs, non_camel_case_types)]

froglight_macros::block_attributes! {{
    crate,
{attributes}}}
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

    /// Generate block unit structs.
    async fn generate_blocks(deps: &mut DependencyContainer, path: &Path) -> anyhow::Result<()> {
        static ONCE: OnceCell<anyhow::Result<()>> = OnceCell::const_new();
        ONCE.get_or_init(async || {
            let mut sorted: Vec<String> = Vec::new();
            for version in deps.get::<ToolConfig>().unwrap().versions.clone() {
                // Get the translations for proper block names.
                deps.get_or_retrieve::<Translations>().await?;
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
                        for block in report.0.keys() {
                            sorted.push(translations.block_name(block));
                        }
                        Ok(())
                    },
                )
                .await?;
            }

            sorted.sort_unstable();
            sorted.dedup();

            let path = path.join("src/generated/block.rs");
            let blocks: String = sorted.into_iter().fold(String::new(), |mut acc, block| {
                acc.push_str("    pub struct ");
                acc.push_str(&block);
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

froglight_macros::blocks! {{
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

impl Blocks {
    const BLOCK_NAME_PADDING: usize = 36;
    const BLOCK_PROPERTY_PADDING: usize = 90;

    /// Generate block trait implementations.
    async fn generate_block_traits(
        version: &Version,
        deps: &mut DependencyContainer,
        path: &Path,
    ) -> anyhow::Result<()> {
        let version_ident =
            format!("froglight_common::version::V{}", version.to_long_string().replace('.', "_"));

        let attributes = deps.get::<BlockAttributes>().unwrap();

        let report = deps.get::<BlockReports>().unwrap();
        let report = report.version(version).unwrap();

        let translations = deps.get::<Translations>().unwrap();
        let translations = translations.version(version).unwrap();

        let implementations =
            report.0.iter().enumerate().fold(String::new(), |mut acc, (i, (block, entry))| {
                let entry_data = BlockAttributeData::from_parsed(block, entry);

                let block_name = translations.block_name(block);
                let block_attrs: Vec<_> =
                    entry_data.attributes.iter().map(|a| attributes.as_enum_name(a)).collect();

                acc.push_str("    ");
                acc.push_str(&block_name);

                // Pad the block name with spaces.
                for _ in 0..Self::BLOCK_NAME_PADDING.saturating_sub(block_name.len()) {
                    acc.push(' ');
                }

                // Add the block properties.
                let block_properties = format!(
                    " => {{ properties: {{ ident: \"{block}\", default: {} }}",
                    entry_data.default_state - entry_data.blockstate_ids.min().unwrap()
                );
                acc.push_str(&block_properties);

                if entry.properties.is_empty() {
                    acc.push(' ');
                } else {
                    acc.push_str(", ");

                    // Pad the block properties with spaces.
                    for _ in 0..Self::BLOCK_PROPERTY_PADDING.saturating_sub(block_properties.len())
                    {
                        acc.push(' ');
                    }

                    // Add the block properties.
                    acc.push_str(&format!(
                        "attributes: {{ ({}): ({}) }} ",
                        entry
                            .properties
                            .keys()
                            .map(|a| format!("\"{a}\""))
                            .collect::<Vec<_>>()
                            .join(", "),
                        block_attrs.join(", ")
                    ));
                }

                acc.push('}');
                if i < report.0.len() {
                    acc.push(',');
                }
                acc.push('\n');

                acc
            });

        let path =
            path.join(format!("src/generated/{}.rs", version.to_long_string().replace('.', "_")));
        tokio::fs::write(
            path,
            format!(
                r"//! This file is generated, do not modify it manually.
//!
//! TODO: Documentation
#![allow(missing_docs)]

froglight_macros::block_properties! {{
    crate,
    version = {version_ident},
{implementations}}}
",
            ),
        )
        .await?;

        Ok(())
    }
}
