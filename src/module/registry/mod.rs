use std::path::Path;

use froglight_dependency::{container::DependencyContainer, version::Version};
use froglight_extract::module::ExtractModule;
use report::RegistryItem;

mod report;

#[derive(ExtractModule)]
#[module(function = Registry::generate)]
pub(crate) struct Registry;

impl Registry {
    async fn generate(version: &Version, deps: &mut DependencyContainer) -> anyhow::Result<()> {
        let mut directory = std::env::current_dir()?;
        directory.push("crates/froglight-registry");

        if !tokio::fs::try_exists(&directory).await? {
            anyhow::bail!("Could not find \"froglight-registry\" at \"{}\"", directory.display());
        }

        Self::generate_registries(version, deps, &directory).await?;

        Ok(())
    }

    async fn generate_registries(
        version: &Version,
        deps: &mut DependencyContainer,
        path: &Path,
    ) -> anyhow::Result<()> {
        let report = Self::get_report(version, deps).await?;

        let mut registries = String::new();
        for RegistryItem { name, default, values } in report.registries {
            // Set the default value, if it exists
            let default = if let Some(default) = default {
                format!("Some(Identifier::const_new(\"{default}\"))")
            } else {
                String::from("None")
            };

            // Wrap all values in `Identifier::const_new("value")`
            let values = values
                .into_iter()
                .map(|s| format!("Identifier::const_new(\"{s}\"),\n"))
                .collect::<Vec<String>>()
                .join("                ");

            registries.push_str(&format!(
                "        storage.register_with_default(
            Identifier::const_new(\"{name}\"),
            {default},
            alloc::vec![
                {values}            ].into_boxed_slice()
        );\n",
            ));
        }

        let version_ident =
            format!("froglight_common::version::V{}", version.to_long_string().replace('.', "_"));

        let path = path.join(format!(
            "src/registry/generated/v{}.rs",
            version.to_long_string().replace('.', "_")
        ));

        tokio::fs::write(
            path,
            format!(
                r"//! This file is generated, do not modify it manually.
//!
//! TODO: Documentation
#![allow(missing_docs)]

use froglight_common::prelude::*;

use crate::prelude::*;

impl RegistryTrait<{version_ident}> for Vanilla {{
    #[allow(clippy::too_many_lines)]
    fn register(storage: &mut RegistryStorage<{version_ident}>) {{
{registries}
    }}
}}
",
            ),
        )
        .await?;

        Ok(())
    }
}
