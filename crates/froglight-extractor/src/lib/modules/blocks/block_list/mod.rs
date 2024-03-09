use std::{borrow::Cow, path::Path};

use cafebabe::{
    bytecode::Opcode,
    constant_pool::{LiteralConstant, Loadable, MemberRef},
};
use froglight_data::Version;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{error, info, warn};

use crate::{
    classmap::ClassMap,
    modules::{code_or_bail, Extract},
};

mod attributes;
use attributes::{BlockType, Instrument, MapColor, PistonBehavior, SoundGroup};

/// A module that extracts the list of blocks.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct BlockListModule;

impl Extract for BlockListModule {
    async fn extract(
        &self,
        version: &Version,
        classmap: &ClassMap,
        _: &Path,
        output: &mut Value,
    ) -> anyhow::Result<()> {
        let Some(class) = classmap.get("net/minecraft/block/Blocks") else {
            anyhow::bail!("Could not find `net/minecraft/block/Blocks` class");
        };

        let code = code_or_bail(&class, "<clinit>")?;

        let mut block_list = Vec::new();
        let mut insn_list = Vec::new();

        // Group insns for easier parsing
        for (_, insn) in &code.opcodes {
            insn_list.push(insn);

            if let Opcode::Putstatic(field) = insn {
                if field.class_name == "net/minecraft/block/Blocks"
                    && field.name_and_type.descriptor == "Lnet/minecraft/block/Block;"
                {
                    let block = parse_insns(
                        &std::mem::take(&mut insn_list),
                        &block_list,
                        version,
                        classmap,
                    );
                    block_list.push(block);
                }
            }
        }

        let mut json_list = Vec::with_capacity(block_list.len());
        for (index, block) in block_list.into_iter().enumerate() {
            if &block.name == "unknown" || &block.field == "unknown" {
                warn!("Unknown block at index {index}");
            }

            // Add the block name to a list
            json_list.push(block.name.clone().into());

            // Add the block data to the output
            output["blocks"]["data"][block.name.as_ref()] = serde_json::json!({
                "id": index,
                // "block_type": block.block_type,
                // "map_color": block.map_color.value().1,
                "collidable": block.collidable,
                // "luminance": block.luminance,
                "resistance": block.resistance,
                "hardness": block.hardness,
                "tool_required": block.tool_required,
                "random_ticks": block.random_ticks,
                "slipperiness": block.slipperiness,
                "velocity_multiplier": block.velocity_multiplier,
                "jump_velocity_multiplier": block.jump_velocity_multiplier,
                // "loot_table_id": block.loot_table_id,
                "opaque": block.opaque,
                "is_air": block.is_air,
                "burnable": block.burnable,
                "liquid": block.liquid,
                "force_not_solid": block.force_not_solid,
                "force_solid": block.force_solid,
                "piston_behavior": block.piston_behavior,
                "block_break_particles": block.block_break_particles,
                // "instrument": block.instrument,
                "replaceable": block.replaceable,
                "dynamic_bounds": block.dynamic_bounds,
            });
        }

        // Add the block list to the output
        output["blocks"]["list"] = Value::Array(json_list);

        Ok(())
    }
}

/// Parse the instructions to get the block data.
#[allow(clippy::too_many_lines)]
fn parse_insns<'a>(
    insns: &[&Opcode<'a>],
    block_list: &[Block],
    version: &Version,
    classmap: &'a ClassMap,
) -> Block<'a> {
    let mut block = Block::default();

    let mut float_storage: Vec<f32> = Vec::with_capacity(2);
    let mut field_storage: Vec<Cow<'_, str>> = Vec::with_capacity(2);

    for (index, insn) in insns.iter().enumerate() {
        if index == 0 {
            // Copy the block name from the LdcW opcode
            if let Opcode::LdcW(Loadable::LiteralConstant(LiteralConstant::String(constant))) = insn
            {
                block.name.clone_from(constant);
                continue;
            }

            // Get the block name from another class's static field
            if let Opcode::Getstatic(MemberRef { class_name, name_and_type }) = insn {
                let Some(other_class) = classmap.get(class_name) else {
                    error!("Block references unknown class: {class_name}");
                    continue;
                };

                let Ok(other_code) = code_or_bail(&other_class, "<clinit>") else {
                    error!("Could not get <clinit> for `{class_name}`");
                    continue;
                };

                // Find the Putstatic opcode for the field
                let mut put_opcode = None;
                for (opcode_index, (_, opcode)) in other_code.opcodes.iter().enumerate() {
                    if let Opcode::Putstatic(other_field) = opcode {
                        if other_field.name_and_type.name == name_and_type.name {
                            put_opcode = Some(opcode_index);
                            break;
                        }
                    }
                }

                // If the Putstatic opcode was found, find the LdcW opcode before it
                if let Some(opcode_index) = put_opcode {
                    let slice = &other_code.opcodes[..opcode_index];
                    let mut found = false;

                    // Find the LdcW opcode and get the block name from it
                    for (_, opcode) in slice.iter().rev() {
                        if let Opcode::Ldc(Loadable::LiteralConstant(LiteralConstant::String(
                            constant,
                        )))
                        | Opcode::LdcW(Loadable::LiteralConstant(LiteralConstant::String(
                            constant,
                        )))
                        | Opcode::Ldc2W(Loadable::LiteralConstant(LiteralConstant::String(
                            constant,
                        ))) = opcode
                        {
                            // Copy the block name
                            let name = constant.clone().to_string();
                            block.name = name.into();

                            // Stop searching
                            found = true;
                            break;
                        }
                    }
                    if found {
                        continue;
                    }

                    error!("Could not find Ldc(W) for `{}` in `{class_name}`", name_and_type.name);
                    continue;
                }

                error!("Could not find Putstatic for `{}` in `{class_name}`", name_and_type.name);
                continue;
            }

            error!("Could not get Block name: {insn:?}");
            continue;
        }

        match insn {
            Opcode::Ldc(Loadable::LiteralConstant(LiteralConstant::Float(float)))
            | Opcode::LdcW(Loadable::LiteralConstant(LiteralConstant::Float(float)))
            | Opcode::Ldc2W(Loadable::LiteralConstant(LiteralConstant::Float(float))) => {
                float_storage.push(*float);
            }
            Opcode::Getstatic(MemberRef { name_and_type, .. }) => {
                field_storage.push(name_and_type.name.clone());
            }
            Opcode::Putstatic(MemberRef { name_and_type, .. }) => {
                block.field.clone_from(&name_and_type.name);
            }
            Opcode::Fconst0 => {
                float_storage.push(0.0);
            }
            Opcode::Fconst1 => {
                float_storage.push(1.0);
            }
            Opcode::Fconst2 => {
                float_storage.push(2.0);
            }
            Opcode::Invokevirtual(member) => {
                #[allow(clippy::match_same_arms)]
                match member.name_and_type.name.as_ref() {
                    "breakInstantly" => {
                        block.break_instantly();
                    }
                    "dynamicBounds" => {
                        block.dynamic_bounds();
                    }
                    "slipperiness" => {
                        let Some(slipperiness) = float_storage.pop() else {
                            error!("Could not get slipperiness constant");
                            continue;
                        };

                        block.slipperiness(slipperiness);
                    }
                    "strength" => match member.name_and_type.descriptor.as_ref() {
                        "(F)Lnet/minecraft/block/AbstractBlock$Settings;" => {
                            let Some(strength) = float_storage.pop() else {
                                info!("{:?}", &insns[index - 5..index + 5]);

                                error!("Could not get strength (F) constant");
                                continue;
                            };

                            block.strength(strength);
                        }
                        "(FF)Lnet/minecraft/block/AbstractBlock$Settings;" => {
                            let Some(resistance) = float_storage.pop() else {
                                error!("Could not get resistance (FF) constant");
                                continue;
                            };
                            let Some(hardness) = float_storage.pop() else {
                                error!("Could not get hardness (FF) constant");
                                continue;
                            };

                            block.strength_hardness(hardness, resistance);
                        }
                        unk => {
                            error!("Unknown `strength` descriptor: {unk}",);
                        }
                    },
                    "noCollision" => {
                        block.no_collision();
                    }
                    "ticksRandomly" => {
                        block.ticks_randomly();
                    }
                    "nonOpaque" => {
                        block.non_opaque();
                    }
                    "velocityMultiplier" => {
                        let Some(velocity_multiplier) = float_storage.pop() else {
                            error!("Could not get velocity_multiplier constant");
                            continue;
                        };

                        block.velocity_multiplier(velocity_multiplier);
                    }
                    "jumpVelocityMultiplier" => {
                        let Some(jump_velocity_multiplier) = float_storage.pop() else {
                            error!("Could not get jump_velocity_multiplier constant");
                            continue;
                        };

                        block.jump_velocity_multiplier(jump_velocity_multiplier);
                    }
                    "air" => {
                        block.air();
                    }
                    "requiresTool" => {
                        block.requires_tool();
                    }
                    "mapColor" => {
                        // block.map_color()
                    }
                    "method_36557" => {
                        let Some(hardness) = float_storage.pop() else {
                            error!("Could not get hardness constant");
                            continue;
                        };

                        block.hardness(hardness);
                    }
                    "method_36558" => {
                        let Some(resistance) = float_storage.pop() else {
                            error!("Could not get resistance constant");
                            continue;
                        };

                        block.resistance(resistance);
                    }
                    "dropsNothing" => {}
                    "dropsLike" => {
                        // block.drops_like()
                    }
                    "method_45477" => {
                        block.no_block_break_particles();
                    }
                    "pistonBehavior" => {
                        // block.piston_behavior()
                    }
                    "burnable" => {
                        block.burnable();
                    }
                    "liquid" => {
                        block.liquid();
                    }
                    "sounds" => {
                        // block.sounds()
                    }
                    "instrument" => {
                        // block.instrument()
                    }
                    "solid" => {
                        block.solid();
                    }
                    "notSolid" => {
                        block.not_solid();
                    }
                    "replaceable" => {
                        block.replaceable();
                    }
                    "noBlockBreakParticles" => {
                        block.no_block_break_particles();
                    }
                    "emissiveLighting" => {}
                    "getDefaultMapColor" => {}
                    "suffocates" => {}
                    "blockVision" => {}
                    "solidBlock" => {}
                    "allowsSpawning" => {}
                    "getDefaultState" => {}
                    "luminance" => {}
                    "offset" => {}
                    "postProcess" => {}
                    "requires" => {}
                    unk => {
                        warn!("Unknown virtual method call: {unk}");
                    }
                }
            }
            Opcode::Invokestatic(member) => {
                #[allow(clippy::match_same_arms)]
                match member.name_and_type.name.as_ref() {
                    // Set the block type
                    "createLogBlock" => {
                        block.block_type = BlockType::Log;
                        block.strength(2.0);
                        // block.sound_group()
                        block.burnable();
                    }
                    "createBambooBlock" => {
                        block.block_type = BlockType::Bamboo;
                        block.strength(2.0);
                        block.burnable();
                    }
                    "createLeavesBlock" => {
                        block.block_type = BlockType::Leaves;
                        block.strength(0.2);
                        block.ticks_randomly();
                        // block.sound_group()
                        block.non_opaque();
                        block.burnable();
                        block.piston_behavior(PistonBehavior::Destroy);
                    }
                    "createBedBlock" => {
                        block.block_type = BlockType::Bed;
                        // block.sound_group()
                        block.strength(0.2);
                        block.non_opaque();
                        block.burnable();
                        block.piston_behavior(PistonBehavior::Destroy);
                    }
                    "createPistonBlock" => {
                        block.block_type = BlockType::Piston;
                        block.strength(1.5);
                        block.piston_behavior(PistonBehavior::Block);
                    }
                    "createStoneButtonBlock" => {
                        block.block_type = BlockType::StoneButton;
                        block.no_collision();
                        block.strength(0.5);
                        block.piston_behavior(PistonBehavior::Destroy);
                    }
                    "createStainedGlassBlock" => {
                        block.block_type = BlockType::StainedGlass;
                        // block.instrument()
                        block.strength(0.3);
                        // block.sound_group()
                        block.non_opaque();
                    }
                    "createFlowerPotBlock" => {
                        block.block_type = BlockType::FlowerPot;
                        block.break_instantly();
                        block.non_opaque();
                        block.piston_behavior(PistonBehavior::Destroy);
                    }
                    "createWoodenButtonBlock" => {
                        block.block_type = BlockType::WoodenButton;
                        block.no_collision();
                        block.strength(0.5);
                        block.piston_behavior(PistonBehavior::Destroy);
                    }
                    "createShulkerBoxBlock" => {
                        block.block_type = BlockType::ShulkerBox;
                        block.solid();
                        block.strength(2.0);
                        block.dynamic_bounds();
                        block.non_opaque();
                        block.piston_behavior(PistonBehavior::Destroy);
                    }
                    "createNetherStemBlock" => {
                        block.block_type = BlockType::NetherStem;
                        block.strength(2.0);
                        // block.sound_group()
                    }
                    "createCandleBlock" => {
                        block.block_type = BlockType::Candle;
                        block.non_opaque();
                        block.strength(0.1);
                        // block.sound_group()
                        block.piston_behavior(PistonBehavior::Destroy);
                    }
                    // Copy properties from another block
                    "copy" | "createStairsBlock" => {
                        let Some(field) = field_storage.pop() else {
                            error!("Could not get field name for copy");
                            continue;
                        };

                        let Some(other) = block_list.iter().find(|other| other.field == field)
                        else {
                            error!("Could not find block for field: {field}");
                            continue;
                        };

                        // Versions before 1.20.3 use the equivalent of `copyShallow`
                        if version.try_ge(&Version::new_rel(1, 20, 3)).unwrap() {
                            block.copy(other);
                        } else {
                            block.copy_shallow(other);
                        }
                    }
                    // Copy properties from another block
                    "copyShallow" | "createOldStairsBlock" => {
                        let Some(field) = field_storage.pop() else {
                            error!("Could not get field name for copy");
                            continue;
                        };

                        let Some(other) = block_list.iter().find(|other| other.field == field)
                        else {
                            error!("Could not find block for field: {field}");
                            continue;
                        };

                        block.copy_shallow(other);
                    }
                    // Luminance
                    "createLightLevelFromLitBlockState" | "getLuminanceSupplier" => {
                        // block.luminance()
                    }
                    // Ignore these methods
                    "create" | "register" => {}
                    unk => {
                        warn!("Unknown static method call: {unk}");
                    }
                }
            }
            _ => {}
        }
    }

    block
}

#[allow(clippy::struct_excessive_bools, clippy::struct_field_names)]
#[derive(Debug, Clone)]
struct Block<'a> {
    name: Cow<'a, str>,
    field: Cow<'a, str>,
    block_type: BlockType,

    _map_color: MapColor,
    collidable: bool,
    sound_group: SoundGroup,
    // luminance: u8,
    resistance: f32,
    hardness: f32,
    tool_required: bool,
    random_ticks: bool,
    slipperiness: f32,
    velocity_multiplier: f32,
    jump_velocity_multiplier: f32,
    _loot_table_id: Option<Cow<'a, str>>,
    opaque: bool,
    is_air: bool,
    burnable: bool,
    // @Deprecated
    liquid: bool,
    // @Deprecated
    force_not_solid: bool,
    force_solid: bool,
    piston_behavior: PistonBehavior,
    block_break_particles: bool,
    instrument: Instrument,
    replaceable: bool,
    dynamic_bounds: bool,
}

impl Default for Block<'_> {
    fn default() -> Self {
        Self {
            name: "UNKNOWN".into(),
            field: "UNKNOWN".into(),
            block_type: BlockType::Block,

            _map_color: MapColor::Clear,
            collidable: true,
            sound_group: SoundGroup::Stone,
            // luminance: 0,
            resistance: 0.0f32,
            hardness: 0.0f32,
            tool_required: false,
            random_ticks: false,
            slipperiness: 0.6f32,
            velocity_multiplier: 1.0f32,
            jump_velocity_multiplier: 1.0f32,
            _loot_table_id: None,
            opaque: true,
            is_air: false,
            burnable: false,
            liquid: false,
            force_not_solid: false,
            force_solid: false,
            piston_behavior: PistonBehavior::Normal,
            block_break_particles: true,
            instrument: Instrument::Harp,
            replaceable: false,
            dynamic_bounds: false,
        }
    }
}

impl Block<'_> {
    fn no_collision(&mut self) {
        self.collidable = false;
        self.opaque = false;
    }

    fn non_opaque(&mut self) { self.opaque = false; }

    fn slipperiness(&mut self, slipperiness: f32) { self.slipperiness = slipperiness; }

    fn velocity_multiplier(&mut self, velocity_multiplier: f32) {
        self.velocity_multiplier = velocity_multiplier;
    }

    fn jump_velocity_multiplier(&mut self, jump_velocity_multiplier: f32) {
        self.jump_velocity_multiplier = jump_velocity_multiplier;
    }

    // fn luminance(&mut self, luminance: u8) { self.luminance = luminance; }

    fn strength_hardness(&mut self, hardness: f32, resistance: f32) {
        self.hardness = hardness;
        self.resistance = resistance;
    }

    fn break_instantly(&mut self) { self.strength_hardness(0.0, 0.0); }

    fn strength(&mut self, strength: f32) { self.strength_hardness(strength, strength) }

    fn ticks_randomly(&mut self) { self.random_ticks = true; }

    fn dynamic_bounds(&mut self) { self.dynamic_bounds = true; }

    fn burnable(&mut self) { self.burnable = true; }

    fn liquid(&mut self) { self.liquid = true; }

    fn solid(&mut self) { self.force_solid = true; }

    fn not_solid(&mut self) { self.force_not_solid = true; }

    fn piston_behavior(&mut self, piston_behavior: PistonBehavior) {
        self.piston_behavior = piston_behavior;
    }

    fn air(&mut self) { self.is_air = true; }

    fn requires_tool(&mut self) { self.tool_required = true; }

    fn hardness(&mut self, hardness: f32) { self.hardness = hardness; }

    fn resistance(&mut self, resistance: f32) { self.resistance = resistance; }

    fn no_block_break_particles(&mut self) { self.block_break_particles = false; }

    #[allow(dead_code)]
    fn instrument(&mut self, instrument: Instrument) { self.instrument = instrument; }

    fn replaceable(&mut self) { self.replaceable = true; }

    fn copy(&mut self, other: &Block) {
        self.copy_shallow(other);
        self.jump_velocity_multiplier = other.jump_velocity_multiplier;
    }

    fn copy_shallow(&mut self, other: &Block) {
        self.hardness = other.hardness;
        self.resistance = other.resistance;
        self.collidable = other.collidable;
        self.random_ticks = other.random_ticks;
        // self.luminance = other.luminance;
        self.sound_group = other.sound_group;
        self.slipperiness = other.slipperiness;
        self.velocity_multiplier = other.velocity_multiplier;
        self.dynamic_bounds = other.dynamic_bounds;
        self.opaque = other.opaque;
        self.is_air = other.is_air;
        self.burnable = other.burnable;
        self.liquid = other.liquid;
        self.force_not_solid = other.force_not_solid;
        self.force_solid = other.force_solid;
        self.piston_behavior = other.piston_behavior;
        self.tool_required = other.tool_required;
        self.block_break_particles = other.block_break_particles;
        self.instrument = other.instrument;
        self.replaceable = other.replaceable;
    }
}
