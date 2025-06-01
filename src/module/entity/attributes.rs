use std::{ops::RangeInclusive, path::Path};

use cafebabe::{
    bytecode::Opcode,
    constant_pool::{LiteralConstant, Loadable, MemberRef},
};
use convert_case::{Case, Casing};
use froglight_dependency::{
    container::DependencyContainer, dependency::minecraft::MinecraftCode, version::Version,
};
use tokio::sync::OnceCell;

use super::Entities;
use crate::{
    class_helper::{ClassHelper, OwnedConstant},
    ToolConfig,
};

impl Entities {
    /// Generate entity attribute structs.
    pub(super) async fn generate_entity_attributes(
        deps: &mut DependencyContainer,
        path: &Path,
    ) -> anyhow::Result<()> {
        static ONCE: OnceCell<anyhow::Result<()>> = OnceCell::const_new();
        ONCE.get_or_init(async || {
            let mut sorted = Vec::new();

            deps.get_or_retrieve::<ToolConfig>().await?;
            deps.scoped_fut::<ToolConfig, anyhow::Result<()>>(async |config, deps| {
                for version in &config.versions {
                    for attribute in Self::extract_entity_attributes(version, deps).await? {
                        sorted.push(attribute.identifier.to_case(Case::Pascal));
                    }
                }
                Ok(())
            })
            .await?;

            sorted.sort_unstable();
            sorted.dedup();

            let path = path.join("src/entity_attribute/generated/attribute.rs");
            let attributes: String = sorted.into_iter().fold(String::new(), |mut acc, attrib| {
                acc.push_str("    pub struct ");
                acc.push_str(&attrib);
                acc.push_str("Attribute(f64);\n");
                acc
            });

            tokio::fs::write(
                path,
                format!(
                    r"//! This file is generated, do not modify it manually.
//!
//! TODO: Documentation
#![allow(missing_docs)]

froglight_macros::entity_attributes! {{
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

    /// Generate status effect trait implementations.
    pub(super) async fn generate_entity_attribute_properties(
        version: &Version,
        deps: &mut DependencyContainer,
        path: &Path,
    ) -> anyhow::Result<()> {
        let version_ident =
            format!("froglight_common::version::V{}", version.to_long_string().replace('.', "_"));
        let path = path.join(format!(
            "src/entity_attribute/generated/v{}.rs",
            version.to_long_string().replace('.', "_")
        ));

        let mut implementations = String::new();
        for EntityAttribute { identifier, translation, default, range } in
            Self::extract_entity_attributes(version, deps).await?
        {
            fn round(d: f64) -> f64 { (d * 10000.0).round() / 10000.0 }

            let attribute_name = identifier.to_case(Case::Pascal);
            let default = format!("{}f64", round(default));
            let range = format!("{}f64..={}f64", round(*range.start()), round(*range.end()));

            implementations
                .push_str(&format!("    {attribute_name}Attribute => {{ properties: {{ ident: \"minecraft:{identifier}\", key: \"minecraft.{translation}\", default: {default}, range: {range} }} }},\n"));
        }

        tokio::fs::write(
            path,
            format!(
                r"//! This file is generated, do not modify it manually.
//!
//! TODO: Documentation
#![allow(missing_docs)]

#[allow(clippy::wildcard_imports)]
use super::attribute::*;

froglight_macros::entity_attribute_properties! {{
    path = crate,
    version = {version_ident},
{implementations}}}
",
            ),
        )
        .await?;

        Ok(())
    }

    async fn extract_entity_attributes(
        version: &Version,
        deps: &mut DependencyContainer,
    ) -> anyhow::Result<Vec<EntityAttribute>> {
        const ENTITY_ATTRIBUTES: &str = "net/minecraft/entity/attribute/EntityAttributes";
        const REGISTRY_ENTRY_DESCRIPTOR: &str = "Lnet/minecraft/registry/entry/RegistryEntry;";

        let mut attributes = Vec::new();

        deps.get_or_retrieve::<MinecraftCode>().await?;
        deps.scoped_fut::<MinecraftCode, anyhow::Result<_>>(async |jars, deps| {
            let jar = jars.get_version(version, deps).await?;
            let class = jar.get(ENTITY_ATTRIBUTES).ok_or_else(|| {
                anyhow::anyhow!("Packets: Could not find \"{ENTITY_ATTRIBUTES}\" class!")
            })?;

            let mut attribute = EntityAttributeBuilder::default();
            let mut constants = Vec::<OwnedConstant>::new();

            let initial = class.class_code().bytecode.as_ref().unwrap();
            let initial = initial.opcodes.iter().map(|(_, opcode)| opcode).collect::<Vec<_>>();
            class.iter_code_recursive(&initial, jar, |opcode| match opcode {
                Opcode::Iconst0 => constants.push(OwnedConstant::Integer(0)),
                Opcode::Iconst1 => constants.push(OwnedConstant::Integer(1)),
                Opcode::Iconst2 => constants.push(OwnedConstant::Integer(2)),
                Opcode::Iconst3 => constants.push(OwnedConstant::Integer(3)),
                Opcode::Iconst4 => constants.push(OwnedConstant::Integer(4)),
                Opcode::Iconst5 => constants.push(OwnedConstant::Integer(5)),
                Opcode::Fconst0 => constants.push(OwnedConstant::Float(0.0)),
                Opcode::Fconst1 => constants.push(OwnedConstant::Float(1.0)),
                Opcode::Fconst2 => constants.push(OwnedConstant::Float(2.0)),
                Opcode::Dconst0 => constants.push(OwnedConstant::Double(0.0)),
                Opcode::Dconst1 => constants.push(OwnedConstant::Double(1.0)),
                Opcode::Ldc(Loadable::LiteralConstant(constant))
                | Opcode::LdcW(Loadable::LiteralConstant(constant))
                | Opcode::Ldc2W(Loadable::LiteralConstant(constant)) => {
                    match constant {
                        // Look for the first string constant and set it as the identifier
                        LiteralConstant::String(string) if attribute.identifier.is_none() => {
                            attribute.identifier = Some(string.to_string());
                        }
                        // Look for the second string constant and set it as the translation key
                        LiteralConstant::String(string) if attribute.translation.is_none() => {
                            attribute.translation = Some(string.to_string());
                        }
                        other => {
                            constants.push(OwnedConstant::from(other));
                        }
                    }
                }
                // Finish the entity type and push it to the list
                Opcode::Putstatic(MemberRef { class_name, name_and_type })
                    if class_name == ENTITY_ATTRIBUTES
                        && name_and_type.descriptor == REGISTRY_ENTRY_DESCRIPTOR =>
                {
                    for constant in constants.drain(..) {
                        match constant {
                            OwnedConstant::Double(d) if attribute.default.is_none() => {
                                attribute.default = Some(d);
                            }
                            OwnedConstant::Double(d) if attribute.range.is_none() => {
                                attribute.range = Some(d..=f64::MAX);
                            }
                            OwnedConstant::Double(d)
                                if attribute
                                    .range
                                    .as_ref()
                                    .is_some_and(|r| r.end() == &f64::MAX) =>
                            {
                                attribute.range =
                                    Some(*attribute.range.as_ref().unwrap().start()..=d);
                            }
                            _ => {}
                        }
                    }

                    attributes.push(core::mem::take(&mut attribute).into());
                }
                _ => {}
            });

            Ok(())
        })
        .await?;

        Ok(attributes)
    }
}

// -------------------------------------------------------------------------------------------------

struct EntityAttribute {
    identifier: String,
    translation: String,
    default: f64,
    range: RangeInclusive<f64>,
}

#[derive(Default)]
struct EntityAttributeBuilder {
    identifier: Option<String>,
    translation: Option<String>,
    default: Option<f64>,
    range: Option<RangeInclusive<f64>>,
}
impl From<EntityAttributeBuilder> for EntityAttribute {
    fn from(builder: EntityAttributeBuilder) -> Self {
        Self {
            identifier: builder.identifier.expect("EntityAttributeBuilder: Identifier is None!"),
            translation: builder.translation.expect("EntityAttributeBuilder: Translation is None!"),
            default: builder.default.expect("EntityAttributeBuilder: Default is None!"),
            range: builder.range.expect("EntityAttributeBuilder: Range is None!"),
        }
    }
}
