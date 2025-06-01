use std::path::Path;

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
use crate::{class_helper::ClassHelper, ToolConfig};

impl Entities {
    /// Generate status effect unit structs.
    pub(super) async fn generate_status_effects(
        deps: &mut DependencyContainer,
        path: &Path,
    ) -> anyhow::Result<()> {
        static ONCE: OnceCell<anyhow::Result<()>> = OnceCell::const_new();
        ONCE.get_or_init(async || {
            let mut sorted = Vec::new();

            deps.get_or_retrieve::<ToolConfig>().await?;
            deps.scoped_fut::<ToolConfig, anyhow::Result<()>>(async |config, deps| {
                for version in &config.versions {
                    for entity_type in Self::extract_status_effects(version, deps).await? {
                        sorted.push(entity_type.identifier.to_case(Case::Pascal));
                    }
                }
                Ok(())
            })
            .await?;

            sorted.sort_unstable();
            sorted.dedup();

            let path = path.join("src/status_effect/generated/effect.rs");
            let effects: String = sorted.into_iter().fold(String::new(), |mut acc, effect| {
                acc.push_str("    pub struct ");
                acc.push_str(&effect);
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

froglight_macros::status_effects! {{
    crate,
{effects}}}
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
    pub(super) async fn generate_status_effect_properties(
        version: &Version,
        deps: &mut DependencyContainer,
        path: &Path,
    ) -> anyhow::Result<()> {
        let version_ident =
            format!("froglight_common::version::V{}", version.to_long_string().replace('.', "_"));
        let path = path.join(format!(
            "src/status_effect/generated/v{}.rs",
            version.to_long_string().replace('.', "_")
        ));

        let mut implementations = String::new();
        for StatusEffect { identifier, category, color } in
            Self::extract_status_effects(version, deps).await?
        {
            let effect_name = identifier.to_case(Case::Pascal);
            let category_name = category.to_case(Case::Pascal);
            implementations
                .push_str(&format!("    {effect_name} => {{ properties: {{ ident: \"minecraft:{identifier}\", category: {category_name}, color: {color:#x} }} }},\n"));
        }

        tokio::fs::write(
            path,
            format!(
                r"//! This file is generated, do not modify it manually.
//!
//! TODO: Documentation
#![allow(missing_docs)]

#[allow(clippy::wildcard_imports)]
use super::effect::*;

froglight_macros::status_effect_properties! {{
    path = crate,
    version = {version_ident},
{implementations}}}
",
            ),
        )
        .await?;

        Ok(())
    }

    async fn extract_status_effects(
        version: &Version,
        deps: &mut DependencyContainer,
    ) -> anyhow::Result<Vec<StatusEffect>> {
        const STATUS_EFFECTS: &str = "net/minecraft/entity/effect/StatusEffects";
        const REGISTRY_ENTRY_DESCRIPTOR: &str = "Lnet/minecraft/registry/entry/RegistryEntry;";
        const STATUS_EFFECT_CATEGORY: &str = "net/minecraft/entity/effect/StatusEffectCategory";

        let mut effects = Vec::new();

        deps.get_or_retrieve::<MinecraftCode>().await?;
        deps.scoped_fut::<MinecraftCode, anyhow::Result<_>>(async |jars, deps| {
            let jar = jars.get_version(version, deps).await?;
            let class = jar.get(STATUS_EFFECTS).ok_or_else(|| {
                anyhow::anyhow!("Packets: Could not find \"{STATUS_EFFECTS}\" class!")
            })?;

            let mut effect = StatusEffectBuilder::default();

            let initial = class.class_code().bytecode.as_ref().unwrap();
            let initial = initial.opcodes.iter().map(|(_, opcode)| opcode).collect::<Vec<_>>();
            class.iter_code_recursive(&initial, jar, |opcode| match opcode {
                Opcode::Ldc(Loadable::LiteralConstant(constant))
                | Opcode::LdcW(Loadable::LiteralConstant(constant))
                | Opcode::Ldc2W(Loadable::LiteralConstant(constant)) => {
                    match constant {
                        // Look for the first string constant and set it as the identifier
                        LiteralConstant::String(string) if effect.identifier.is_none() => {
                            effect.identifier = Some(string.to_string());
                        }
                        // Look for the first integer constant and set it as the color
                        LiteralConstant::Integer(integer) if effect.color.is_none() => {
                            // Look for the first integer constant and set it as the color
                            effect.color = Some(*integer as u32);
                        }
                        _ => {}
                    }
                }
                // Look for references to the effect category and set it
                Opcode::Getstatic(MemberRef { class_name, name_and_type })
                    if class_name == STATUS_EFFECT_CATEGORY =>
                {
                    effect.category = Some(name_and_type.name.to_lowercase());
                }
                // Finish the entity type and push it to the list
                Opcode::Putstatic(MemberRef { class_name, name_and_type })
                    if class_name == STATUS_EFFECTS
                        && name_and_type.descriptor == REGISTRY_ENTRY_DESCRIPTOR =>
                {
                    effects.push(core::mem::take(&mut effect).into());
                }
                _ => {}
            });

            Ok(())
        })
        .await?;

        Ok(effects)
    }
}

// -------------------------------------------------------------------------------------------------

struct StatusEffect {
    identifier: String,
    category: String,
    color: u32,
}

#[derive(Default)]
struct StatusEffectBuilder {
    identifier: Option<String>,
    category: Option<String>,
    color: Option<u32>,
}
impl From<StatusEffectBuilder> for StatusEffect {
    fn from(builder: StatusEffectBuilder) -> Self {
        Self {
            identifier: builder.identifier.expect("StatusEffectBuilder: Identifier is None!"),
            category: builder.category.expect("StatusEffectBuilder: Category is None!"),
            color: builder.color.expect("StatusEffectBuilder: Color is None!"),
        }
    }
}
