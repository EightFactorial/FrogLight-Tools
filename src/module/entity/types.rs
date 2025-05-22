use cafebabe::{
    bytecode::Opcode,
    constant_pool::{LiteralConstant, Loadable, MemberRef},
};
use froglight_dependency::{
    container::DependencyContainer, dependency::minecraft::MinecraftCode, version::Version,
};
use tracing::warn;

use super::Entities;
use crate::class_helper::ClassHelper;

impl Entities {
    pub(super) async fn extract_entity_types(
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
                | Opcode::LdcW(Loadable::LiteralConstant(constant)) => {
                    if let LiteralConstant::String(string) = constant
                        && constants.is_empty()
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
                                entity.dimensions = (Some(width), Some(height));
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

    pub dimensions: (Option<f32>, Option<f32>),
    pub eye_height: Option<f32>,
}
impl From<EntityTypeBuilder> for EntityType {
    fn from(builder: EntityTypeBuilder) -> Self {
        Self {
            identifier: builder.identifier.expect("EntityTypeBuilder: Identifier is None!"),
            spawn_group: builder.spawn_group.expect("EntityTypeBuilder: SpawnGroup is None!"),
            fire_immune: builder.fire_immune.unwrap_or(false),
            dimensions: (
                builder.dimensions.0.expect("EntityTypeBuilder: Width is None!"),
                builder.dimensions.1.expect("EntityTypeBuilder: Height is None!"),
            ),
            eye_height: builder.eye_height.unwrap_or_else(|| builder.dimensions.1.unwrap() * 0.85),
        }
    }
}

#[expect(dead_code)]
#[derive(Clone, Debug)]
enum OwnedConstant {
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    String(String),
    StringBytes(Vec<u8>),
}

impl From<&LiteralConstant<'_>> for OwnedConstant {
    fn from(constant: &LiteralConstant) -> Self {
        match constant {
            LiteralConstant::Integer(value) => OwnedConstant::Integer(*value),
            LiteralConstant::Float(value) => OwnedConstant::Float(*value),
            LiteralConstant::Long(value) => OwnedConstant::Long(*value),
            LiteralConstant::Double(value) => OwnedConstant::Double(*value),
            LiteralConstant::String(string) => OwnedConstant::String(string.clone().into_owned()),
            LiteralConstant::StringBytes(bytes) => OwnedConstant::StringBytes(bytes.to_vec()),
        }
    }
}
