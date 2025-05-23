use std::path::Path;

use convert_case::{Case, Casing};
use froglight_dependency::{container::DependencyContainer, version::Version};
use froglight_extract::module::ExtractModule;
use tokio::sync::OnceCell;
use types::EntityType;

use crate::ToolConfig;

mod types;

#[derive(ExtractModule)]
#[module(function = Entities::generate)]
pub(crate) struct Entities;

impl Entities {
    async fn generate(version: &Version, deps: &mut DependencyContainer) -> anyhow::Result<()> {
        let mut directory = std::env::current_dir()?;
        directory.push("crates/froglight-entity");

        if !tokio::fs::try_exists(&directory).await? {
            anyhow::bail!("Could not find \"froglight-entity\" at \"{}\"", directory.display());
        }

        Self::generate_entity_types(deps, &directory).await?;
        Self::generate_entity_type_properties(version, deps, &directory).await?;

        Ok(())
    }

    /// Generate entity type unit structs.
    async fn generate_entity_types(
        deps: &mut DependencyContainer,
        path: &Path,
    ) -> anyhow::Result<()> {
        static ONCE: OnceCell<anyhow::Result<()>> = OnceCell::const_new();
        ONCE.get_or_init(async || {
            let mut sorted = Vec::new();

            deps.get_or_retrieve::<ToolConfig>().await?;
            deps.scoped_fut::<ToolConfig, anyhow::Result<()>>(async |config, deps| {
                for version in &config.versions {
                    for entity_type in Self::extract_entity_types(version, deps).await? {
                        sorted.push(entity_type.identifier.to_case(Case::Pascal));
                    }
                }
                Ok(())
            })
            .await?;

            sorted.sort_unstable();
            sorted.dedup();

            let path = path.join("src/entity_type/generated/entity.rs");
            let entities: String = sorted.into_iter().fold(String::new(), |mut acc, entity| {
                acc.push_str("    pub struct ");
                acc.push_str(&entity);
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

froglight_macros::entity_types! {{
    crate,
{entities}}}
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

    /// Generate entity trait implementations.
    async fn generate_entity_type_properties(
        version: &Version,
        deps: &mut DependencyContainer,
        path: &Path,
    ) -> anyhow::Result<()> {
        let version_ident =
            format!("froglight_common::version::V{}", version.to_long_string().replace('.', "_"));
        let path = path.join(format!(
            "src/entity_type/generated/v{}.rs",
            version.to_long_string().replace('.', "_")
        ));

        let mut implementations = String::new();
        for EntityType { identifier, spawn_group, fire_immune, dimensions, eye_height } in
            Self::extract_entity_types(version, deps).await?
        {
            let entity_name = identifier.to_case(Case::Pascal);
            let dimensions = format!("[{}f32, {}f32, {eye_height}f32]", dimensions.0, dimensions.1);
            implementations
                .push_str(&format!("    {entity_name} => {{ properties: {{ ident: \"minecraft:{identifier}\", group: \"minecraft:{spawn_group}\", dimensions: {dimensions}, fire_immune: {fire_immune} }} }},\n"));
        }

        tokio::fs::write(
            path,
            format!(
                r"//! This file is generated, do not modify it manually.
//!
//! TODO: Documentation
#![allow(missing_docs)]

#[allow(clippy::wildcard_imports)]
use super::entity::*;

froglight_macros::entity_type_properties! {{
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
