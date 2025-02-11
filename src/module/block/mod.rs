#![expect(unused_imports)]

use std::{fmt::Write, path::Path, sync::Once};

use froglight_dependency::{container::DependencyContainer, version::Version};
use froglight_extract::module::ExtractModule;
use tokio::sync::OnceCell;
use tracing::Level;

mod attribute;
pub(crate) use attribute::{BlockAttributes, BlockReports};

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

        let attrs = deps.get_or_retrieve::<BlockAttributes>().await?.clone();
        Self::generate_attributes(attrs, &directory).await?;

        Ok(())
    }
}

impl Blocks {
    async fn generate_attributes(attrs: BlockAttributes, path: &Path) -> anyhow::Result<()> {
        static ONCE: OnceCell<anyhow::Result<()>> = OnceCell::const_new();
        ONCE.get_or_init(async || {
            let mut sorted: Vec<_> = attrs.0.iter().collect();
            sorted.sort_unstable_by(|a, b| match a.name.cmp(&b.name) {
                std::cmp::Ordering::Equal => a.values.cmp(&b.values),
                other => other,
            });

            let path = path.join("src/generated/attribute.rs");
            let attributes: String = sorted.into_iter().fold(String::new(), |mut acc, attr| {
                acc.write_str("    ").unwrap();
                acc.write_str(&attrs.as_enum_macro(attr)).unwrap();
                acc.write_str(",\n").unwrap();
                acc
            });

            tokio::fs::write(
                path,
                format!(
                    r"//! This file is generated, do not modify it manually.
//!
//! TODO: Documentation

froglight_macros::block_attributes! {{
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
}
