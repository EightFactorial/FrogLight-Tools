#![expect(unused_imports)]

use std::{fmt::Write, path::Path, sync::Once};

use attribute::ParsedBlockReport;
use convert_case::{Case, Casing};
use froglight_dependency::{container::DependencyContainer, version::Version};
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
    async fn generate(_v: &Version, deps: &mut DependencyContainer) -> anyhow::Result<()> {
        let mut directory = std::env::current_dir()?;
        directory.push("crates/froglight-block");

        if !tokio::fs::try_exists(&directory).await? {
            anyhow::bail!("Could not find \"froglight-block\" at \"{}\"", directory.display());
        }

        Self::generate_attributes(deps, &directory).await?;
        Self::generate_blocks(deps, &directory).await?;

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
    ///
    /// TODO: Don't create block names from the block identifier,
    /// use the name from the translations instead.
    async fn generate_blocks(deps: &mut DependencyContainer, path: &Path) -> anyhow::Result<()> {
        static ONCE: OnceCell<anyhow::Result<()>> = OnceCell::const_new();
        ONCE.get_or_init(async || {
            let mut sorted: Vec<String> = Vec::new();
            for version in deps.get::<ToolConfig>().unwrap().versions.clone() {
                deps.scoped_fut::<BlockReports, anyhow::Result<()>>(
                    async |reports: &mut BlockReports, deps| {
                        let report = reports.get_version(&version, deps).await?;
                        sorted.extend(report.0.keys().cloned());
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
                acc.push_str(&block.split(':').next_back().unwrap().to_case(Case::Pascal));
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
