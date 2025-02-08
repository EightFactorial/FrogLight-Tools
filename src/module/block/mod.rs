use std::{collections::HashSet, ops::RangeInclusive};

use attribute::BlockAttribute;
use convert_case::{Case, Casing};
use froglight_dependency::{
    container::{Dependency, DependencyContainer},
    dependency::minecraft::{DataGenerator, MinecraftCode},
    version::Version,
};
use froglight_extract::module::ExtractModule;

mod attribute;
pub(crate) use attribute::BlockAttributeBundle;

mod property;
pub(crate) use property::BlockPropertyBundle;

use crate::ToolConfig;

#[derive(ExtractModule)]
#[module(function = Blocks::generate)]
pub(crate) struct Blocks;

impl Blocks {
    async fn generate(version: &Version, deps: &mut DependencyContainer) -> anyhow::Result<()> {
        // Prepare the `BlockAttributeBundle`
        deps.get_or_retrieve::<DataGenerator>().await?;
        let attributes = deps
            .scoped_fut::<DataGenerator, anyhow::Result<BlockAttributeBundle>>(
                async |data: &mut DataGenerator, deps| {
                    BlockAttributeBundle::parse(data.get_version(version, deps).await?).await
                },
            )
            .await?;

        // Collect all unique attributes
        let attrs = deps.get_or_retrieve_mut::<BlockAttributes>().await?;
        for block in &attributes.0 {
            for attr in &block.attributes {
                attrs.0.insert(attr.clone());
            }
        }

        // Prepare the `BlockPropertyBundle`
        deps.get_or_retrieve::<MinecraftCode>().await?;
        let properties = deps
            .scoped_fut::<MinecraftCode, anyhow::Result<BlockPropertyBundle>>(
                async |code: &mut MinecraftCode, deps| {
                    BlockPropertyBundle::parse(code.get_version(version, deps).await?)
                },
            )
            .await?;

        // Generate the blocks
        Self::generate_blocks(version, &attributes, &properties).await
    }

    async fn generate_blocks(
        version: &Version,
        _attributes: &BlockAttributeBundle,
        _properties: &BlockPropertyBundle,
    ) -> anyhow::Result<()> {
        let project_dir = std::env::current_dir()?;
        let mut crate_dir = project_dir.join("crates").join("froglight-block");

        if !tokio::fs::try_exists(&crate_dir).await? {
            anyhow::bail!("Blocks: Unable to find `froglight-block` crate!");
        }

        crate_dir.push("src/generated");
        crate_dir.push(format!("{}.rs", version.to_long_string()));
        tracing::info!("Generating blocks at \"{}\"", crate_dir.display());

        Ok(())
    }
}

#[derive(Default, Dependency, ExtractModule)]
#[module(function = BlockAttributes::generate)]
pub(crate) struct BlockAttributes(HashSet<BlockAttribute>);

impl BlockAttributes {
    async fn generate(version: &Version, deps: &mut DependencyContainer) -> anyhow::Result<()> {
        let config = deps.get::<ToolConfig>().unwrap();
        if config.versions.last() != Some(version) {
            tracing::debug!("Skipping \"blockattributes\" until all versions are processed");
            return Ok(());
        }

        let project_dir = std::env::current_dir()?;
        let mut crate_dir = project_dir.join("crates").join("froglight-block");

        if !tokio::fs::try_exists(&crate_dir).await? {
            anyhow::bail!("Blocks: Unable to find `froglight-block` crate!");
        }

        crate_dir.push("src/generated/attribute.rs");
        tracing::info!("Generating block attributes at \"{}\"", crate_dir.display());

        // ----

        let attributes = deps.take::<Self>().unwrap_or_default();
        if attributes.0.is_empty() {
            tracing::warn!("BlockAttributes are empty, did you forget to generate blocks?");
            return Ok(());
        }

        let attributes: Vec<_> =
            attributes.0.into_iter().map(BlockAttributeEnum::from_attr).collect();
        let mut attributes: Vec<String> =
            attributes.iter().map(|(name, attr)| attr.as_enum(name, &attributes)).collect();
        attributes.sort_unstable();
        tracing::debug!("Attributes: {attributes:#?}");

        Ok(())
    }
}

#[derive(PartialEq, Eq, Hash)]
enum BlockAttributeEnum {
    Bool,
    Int(RangeInclusive<u32>),
    Enum(Vec<String>),
}

impl BlockAttributeEnum {
    fn from_attr(attr: BlockAttribute) -> (String, Self) {
        if attr.values.len() == 2 && attr.values[0] == "true" && attr.values[1] == "false" {
            return (attr.name, Self::Bool);
        }

        if let Ok(values) = attr.values.iter().map(|v| v.parse()).collect::<Result<Vec<u32>, _>>() {
            let min = values.iter().min().copied().unwrap_or(0);
            let max = values.iter().max().copied().unwrap_or(0);
            return (attr.name, Self::Int(min..=max));
        }

        (attr.name, Self::Enum(attr.values))
    }

    fn as_enum(&self, name: &str, others: &[(String, BlockAttributeEnum)]) -> String {
        let mut ident = name.to_case(Case::Pascal);
        if others.iter().filter(|(n, _)| n == name).count() > 1 {
            match self {
                BlockAttributeEnum::Int(range) => {
                    ident = format!("{ident}Range{}To{}", range.start(), range.end());
                }
                BlockAttributeEnum::Enum(items) => {
                    let items =
                        items.iter().map(|v| v.to_case(Case::Pascal)).collect::<Vec<_>>().join("_");
                    ident = format!("{ident}Enum_{items}");
                }
                BlockAttributeEnum::Bool => {}
            }
        }

        match self {
            BlockAttributeEnum::Bool => format!("pub enum {ident} {{ True, False }}"),
            BlockAttributeEnum::Int(range) => {
                format!(
                    "pub enum {ident} {{ {} }}",
                    range.clone().map(|v| format!("_{v}")).collect::<Vec<_>>().join(", ")
                )
            }
            BlockAttributeEnum::Enum(items) => {
                format!(
                    "pub enum {ident} {{ {} }}",
                    items.iter().map(|v| v.to_case(Case::Pascal)).collect::<Vec<_>>().join(", ")
                )
            }
        }
    }
}
