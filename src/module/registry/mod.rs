use std::{path::Path, sync::Arc};

use convert_case::{Case, Casing};
use froglight_dependency::{
    container::DependencyContainer, dependency::mojang::VersionManifest, version::Version,
};
use froglight_extract::module::ExtractModule;
use structure::{DataStructures, RegistryField, RegistryItem, VersionStructures};

use super::ToolConfig;

mod structure;

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

        let mut versions: Vec<_> = deps.get::<ToolConfig>().unwrap().versions.clone();

        // Sort versions using the manifest.
        let manifest = deps.get_or_retrieve::<VersionManifest>().await?;
        versions.sort_by(|a, b| manifest.compare(a, b).unwrap());

        // Generate registries using the current and previous versions.
        let previous = versions.iter().position(|v| v == version).and_then(|i| versions.get(i - 1));
        Self::generate_registries(version, previous, deps, &directory)
            .await
            .map_err(|e| anyhow::anyhow!("Registry: {e}"))?;

        Ok(())
    }
}

impl Registry {
    /// Generate registries.
    async fn generate_registries(
        version: &Version,
        previous: Option<&Version>,
        deps: &mut DependencyContainer,
        path: &Path,
    ) -> anyhow::Result<()> {
        let path =
            path.join(format!("src/generated/v{}.rs", version.to_long_string().replace('.', "_")));

        // if tokio::fs::try_exists(&path).await? {
        //     return Ok(());
        // }

        tracing::debug!("Generating registry file \"{}\"", path.display());

        // Tell the macro where the previous version is to inherit from.
        let inheritance = previous
            .map(|p| format!("    inherit = super::v{},\n", p.to_long_string().replace('.', "_")))
            .unwrap_or_default();

        // Build the registry macro contents.
        let (current, previous) = Self::get_structures(version, previous, deps).await?;

        let mut generated = String::new();
        let registry = current
            .0
            .iter()
            .map(|(path, current)| (path, current, previous.as_ref().and_then(|p| p.0.get(path))))
            .fold(String::new(), |mut acc, (path, current, previous)| {
                let ident = Self::path_to_ident(path);

                if Some(current) == previous {
                    acc.push_str(&format!("        struct {ident} {{ Inherited }}"));
                } else {
                    acc.push_str(&format!("        struct {ident} {{ "));
                    Self::print_registry_item(&ident, current, previous, &mut acc, &mut generated);
                    acc.pop();
                    acc.pop();
                    acc.push_str(" }");
                }

                acc.push_str("\n            => \"minecraft:foo\": todo!(),");
                acc.push_str("\n            => \"minecraft:bar\": todo!(),");
                acc.push('\n');

                acc
            });
        generated.pop();

        // Get the version path and feature.
        let version_path =
            format!("froglight_common::version::V{}", version.to_long_string().replace('.', "_"));
        let feature = format!("\"v{}\"", version.to_long_string().replace('.', "_"));

        tokio::fs::write(
            path,
            format!(
                r"//! This file is generated, do not modify it manually.
//!
//! TODO: Documentation
#![allow(missing_docs)]

froglight_macros::registry_values! {{
    path = crate,
    feature = {feature},
    version = {version_path},
{inheritance}    Attribute => {{
{registry}    }},
    GeneratedType => {{
{generated}
    }}
}}"
            ),
        )
        .await?;

        Ok(())
    }

    fn path_to_ident(path: &Path) -> String {
        if let Some(parent) = path.parent() {
            format!(
                "{}{}",
                path.strip_prefix(parent).unwrap().to_string_lossy().to_case(Case::Pascal),
                parent.to_string_lossy().replace('/', "_").to_case(Case::Pascal)
            )
        } else {
            path.to_string_lossy().to_case(Case::Pascal)
        }
        .replace("Tags", "Tag")
    }

    const WORDS: usize = 4;

    /// Take the last `N` words, separated by capital letters.
    fn trim_ident(ident: String) -> String {
        let skip = ident.chars().filter(|c| c.is_uppercase()).count().saturating_sub(Self::WORDS);
        let ident = ident.split_inclusive(char::is_uppercase).skip(skip).collect::<String>();
        ident
            .trim_start_matches(char::is_lowercase)
            .trim_end_matches(char::is_uppercase)
            .to_string()
    }

    fn print_registry_item(
        parent: &str,
        item: &RegistryItem,
        previous: Option<&RegistryItem>,
        item_struct: &mut String,
        generated: &mut String,
    ) {
        for (field_name, field_type, previous_type) in
            item.0.iter().map(|(n, t)| (n, t, previous.and_then(|p| p.0.get(n))))
        {
            item_struct.push_str(&field_name.replace(':', "_").to_case(Case::Snake));
            item_struct.push_str(": ");

            Self::print_registry_field(
                parent,
                field_name,
                field_type,
                previous_type,
                item_struct,
                generated,
            );

            item_struct.push_str(", ");
        }
    }

    #[allow(clippy::too_many_lines)]
    fn print_registry_field(
        parent: &str,
        field_name: &str,
        field: &RegistryField,
        previous: Option<&RegistryField>,
        item_struct: &mut String,
        generated: &mut String,
    ) {
        match field {
            RegistryField::Bool => item_struct.push_str("bool"),
            RegistryField::Float => item_struct.push_str("f32"),
            RegistryField::Integer => item_struct.push_str("i32"),
            RegistryField::String => item_struct.push_str("String"),
            RegistryField::Vec(field) => {
                item_struct.push_str("Vec<");
                Self::print_registry_field(
                    parent,
                    field_name,
                    field,
                    if let Some(RegistryField::Vec(previous)) = previous {
                        Some(previous)
                    } else {
                        None
                    },
                    item_struct,
                    generated,
                );
                item_struct.push('>');
            }
            RegistryField::Item(item) => {
                let item_ident =
                    Self::trim_ident(format!("{parent}{}", field_name.to_case(Case::Pascal)));

                item_struct.push_str(&item_ident);

                if generated.contains(&item_ident) {
                    return;
                }

                let mut item_string = String::new();
                Self::print_registry_item(
                    &item_ident,
                    item,
                    if let Some(RegistryField::Item(previous)) = previous {
                        Some(previous)
                    } else {
                        None
                    },
                    &mut item_string,
                    generated,
                );

                generated.push_str("        struct ");
                generated.push_str(&item_ident);
                generated.push_str(" { ");
                if Some(field) == previous {
                    generated.push_str("Inherited  ");
                } else {
                    generated.push_str(&item_string);
                }
                generated.pop();
                generated.pop();
                generated.push_str(" },\n");
            }
            RegistryField::Enum(variants) => {
                let enum_ident =
                    Self::trim_ident(format!("{parent}{}", field_name.to_case(Case::Pascal)));

                item_struct.push_str(&enum_ident);

                if generated.contains(&enum_ident) {
                    return;
                }

                let mut enum_string = String::new();
                for (index, variant) in variants.iter().enumerate() {
                    let variant_type = match variant {
                        RegistryField::Bool => "Bool",
                        RegistryField::Float => "Float",
                        RegistryField::Integer => "Integer",
                        RegistryField::String => "String",
                        RegistryField::Enum(..) => "Enum",
                        RegistryField::Item(..) => "Item",
                        RegistryField::Vec(..) => "Vec",
                        RegistryField::Null => unreachable!(),
                    };

                    enum_string.push_str(variant_type);
                    enum_string.push('(');
                    Self::print_registry_field(
                        &enum_ident,
                        variant_type,
                        variant,
                        previous.and_then(|p| {
                            if let RegistryField::Enum(variants) = p {
                                variants.get(index)
                            } else {
                                None
                            }
                        }),
                        &mut enum_string,
                        generated,
                    );
                    enum_string.push_str("), ");
                }

                generated.push_str("        enum ");
                generated.push_str(&enum_ident);
                generated.push_str(" { ");
                if Some(field) == previous {
                    generated.push_str("Inherited  ");
                } else {
                    generated.push_str(&enum_string);
                }
                generated.pop();
                generated.pop();
                generated.push_str(" },\n");
            }
            RegistryField::Null => item_struct.push_str("Null"),
        }
    }

    /// Get the structures for the current and previous versions.
    async fn get_structures(
        current: &Version,
        previous: Option<&Version>,
        deps: &mut DependencyContainer,
    ) -> anyhow::Result<(Arc<VersionStructures>, Option<Arc<VersionStructures>>)> {
        deps.get_or_retrieve::<DataStructures>().await?;
        deps
            .scoped_fut::<DataStructures, anyhow::Result<(Arc<VersionStructures>, Option<Arc<VersionStructures>>)>>(
                async |data: &mut DataStructures, deps| {
                    let current = data.get_version(current, deps).await?.clone();
                    if let Some(previous) = previous {
                        Ok((current, Some(data.get_version(previous, deps).await?.clone())))
                    } else {
                        Ok((current, None))
                    }

                },
            )
            .await
    }
}
