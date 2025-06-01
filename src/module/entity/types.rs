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
use tracing::warn;

use super::Entities;
use crate::{
    class_helper::{ClassHelper, OwnedConstant},
    ToolConfig,
};

impl Entities {
    /// Generate entity type unit structs.
    pub(super) async fn generate_entity_types(
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
    pub(super) async fn generate_entity_type_properties(
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

    async fn extract_entity_types(
        version: &Version,
        deps: &mut DependencyContainer,
    ) -> anyhow::Result<Vec<EntityType>> {
        const ENTITY_TYPE: &str = "net/minecraft/entity/EntityType";
        const ENTITY_TYPE_DESCRIPTOR: &str = "Lnet/minecraft/entity/EntityType;";
        const ENTITY_BUILDER: &str = "net/minecraft/entity/EntityType$Builder";
        const SPAWN_GROUP: &str = "net/minecraft/entity/SpawnGroup";

        let mut entities = Vec::new();

        deps.get_or_retrieve::<MinecraftCode>().await?;
        deps.scoped_fut::<MinecraftCode, anyhow::Result<_>>(async |jars, deps| {
            let jar = jars.get_version(version, deps).await?;
            let class = jar.get(ENTITY_TYPE).ok_or_else(|| {
                anyhow::anyhow!("Packets: Could not find \"{ENTITY_TYPE}\" class!")
            })?;

            let mut entity = EntityTypeBuilder::default();
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
                // Look for the first string constant and set it as the identifier
                Opcode::Ldc(Loadable::LiteralConstant(constant))
                | Opcode::LdcW(Loadable::LiteralConstant(constant))
                | Opcode::Ldc2W(Loadable::LiteralConstant(constant)) => {
                    if let LiteralConstant::String(string) = constant
                        && constants.is_empty()
                        && entity.identifier.is_none()
                    {
                        entity.identifier = Some(string.clone().into_owned());
                    } else {
                        constants.push(constant.into());
                    }
                }
                // Look for references to the spawn group and set it
                Opcode::Getstatic(MemberRef { class_name, name_and_type })
                    if class_name == SPAWN_GROUP =>
                {
                    entity.spawn_group = Some(name_and_type.name.to_lowercase());
                }
                // Match any entity builder methods called
                Opcode::Invokevirtual(MemberRef { class_name, name_and_type })
                    if class_name == ENTITY_BUILDER =>
                {
                    match name_and_type.name.as_ref() {
                        "dimensions" => match (constants.pop(), constants.pop()) {
                            (
                                Some(OwnedConstant::Float(height)),
                                Some(OwnedConstant::Float(width)),
                            ) => {
                                entity.dimensions = Some((width, height));
                            }
                            (Some(..), Some(..)) => {
                                panic!("EntityType: Dimensions are incorrect type!")
                            }
                            _ => panic!("EntityType: Dimensions expected but not found!"),
                        },
                        "eyeHeight" => match constants.pop() {
                            Some(OwnedConstant::Float(height)) => {
                                entity.eye_height = Some(height);
                            }
                            Some(..) => panic!("EntityType: EyeHeight is incorrect type!"),
                            _ => panic!("EntityType: EyeHeight expected but not found!"),
                        },
                        "makeFireImmune" => entity.fire_immune = Some(true),
                        // Ignore known methods that are not used
                        "allowSpawningInside"
                        | "attachment"
                        | "disableSaving"
                        | "disableSummon"
                        | "dropsNothing"
                        | "maxTrackingRange"
                        | "nameTagAttachment"
                        | "passengerAttachments"
                        | "spawnableFarFromPlayer"
                        | "spawnBoxScale"
                        | "trackingTickInterval"
                        | "vehicleAttachment" => {}
                        // Warn about unknown methods
                        unk => warn!("EntityType: Unknown builder function \"{unk}\""),
                    }
                }
                // Finish the entity type and push it to the list
                Opcode::Putstatic(MemberRef { class_name, name_and_type })
                    if class_name == ENTITY_TYPE
                        && name_and_type.descriptor == ENTITY_TYPE_DESCRIPTOR =>
                {
                    constants.clear();
                    entities.push(core::mem::take(&mut entity).into());
                }
                _ => {}
            });

            Ok(())
        })
        .await?;

        Ok(entities)
    }
}

// -------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub(super) struct EntityType {
    pub identifier: String,
    pub spawn_group: String,
    pub fire_immune: bool,

    pub dimensions: (f32, f32),
    pub eye_height: f32,
}

#[derive(Debug, Default, Clone, PartialEq)]
struct EntityTypeBuilder {
    pub identifier: Option<String>,
    pub spawn_group: Option<String>,
    pub fire_immune: Option<bool>,

    pub dimensions: Option<(f32, f32)>,
    pub eye_height: Option<f32>,
}
impl From<EntityTypeBuilder> for EntityType {
    fn from(builder: EntityTypeBuilder) -> Self {
        Self {
            identifier: builder.identifier.expect("EntityTypeBuilder: Identifier is None!"),
            spawn_group: builder.spawn_group.expect("EntityTypeBuilder: SpawnGroup is None!"),
            fire_immune: builder.fire_immune.unwrap_or(false),
            dimensions: builder.dimensions.expect("EntityTypeBuilder: Dimensions is None!"),
            eye_height: builder.eye_height.unwrap_or_else(|| builder.dimensions.unwrap().1 * 0.85),
        }
    }
}
