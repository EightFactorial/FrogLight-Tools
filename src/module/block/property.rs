#![allow(dead_code)]

use std::collections::HashMap;

use cafebabe::{
    bytecode::Opcode,
    constant_pool::{LiteralConstant, Loadable},
    ClassFile,
};
use froglight_dependency::dependency::minecraft::minecraft_code::CodeBundle;

use crate::class_helper::ClassHelper;

pub(crate) struct BlockPropertyBundle(HashMap<String, BlockData>);

impl BlockPropertyBundle {
    pub(crate) fn parse(classes: &CodeBundle) -> anyhow::Result<Self> {
        let Some(class) = classes.get("net/minecraft/block/Blocks") else {
            anyhow::bail!("Blocks: Unable to find `net/minecraft/block/Blocks.class`!");
        };

        let code = class.class_code();

        // Separate each block instance into separate bundles of code
        let mut bundles = Vec::new();
        {
            tracing::trace!("---");
            let mut opcodes = Vec::new();
            for (_, opcode) in &code.bytecode.as_ref().unwrap().opcodes {
                opcodes.push(opcode);
                if let Opcode::Putstatic(member) = opcode {
                    let opcodes = std::mem::take(&mut opcodes);
                    if member.class_name == class.this_class
                        && member.name_and_type.descriptor == "Lnet/minecraft/block/Block;"
                    {
                        tracing::trace!("Found Static: {}", member.name_and_type.name);
                        bundles.push(opcodes);
                    }
                }
            }
            tracing::trace!("---");
        }

        // Parse each block bundle into a `BlockData` instance
        let blocks = bundles
            .into_iter()
            .map(|block| BlockData::parse_bundle(&class, &block, classes))
            .collect::<anyhow::Result<_>>()?;

        Ok(Self(blocks))
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct BlockData {
    pub sound_group: String,
    pub piston_behaviour: PistonBehaviour,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum PistonBehaviour {
    #[default]
    Normal,
    Destroy,
    Block,
    Ignore,
    PushOnly,
}

impl BlockData {
    fn parse_bundle(
        class: &ClassFile,
        code: &[&Opcode<'_>],
        classes: &CodeBundle,
    ) -> anyhow::Result<(String, Self)> {
        let mut identifier = String::new();
        let mut block = Self::default();

        // Parse all opcodes that create a block
        let mut index = 0usize;
        class.iter_code_recursive(code, classes, |opcode: &Opcode<'_>| {
            tracing::trace!("{index}: {opcode:?}");

            match (index, opcode) {
                (0, Opcode::LdcW(Loadable::LiteralConstant(LiteralConstant::String(string)))) => {
                    identifier = string.to_string();
                }
                (0, Opcode::Getstatic(member))
                    if member.class_name == "net/minecraft/block/BlockKeys" =>
                {
                    // TODO: Actually parse the `RegistryKey` instance
                    identifier = member.name_and_type.name.to_lowercase();
                }
                // Set the default block values based on the block type
                (_, Opcode::New(class_type)) => Self::handle_blocktype(class_type, &mut block),
                // Set the `BlockSoundGroup` for the block
                (_, Opcode::Getstatic(member))
                    if member.class_name == "net/minecraft/sound/BlockSoundGroup" =>
                {
                    block.sound_group = member.name_and_type.name.to_string();
                }
                // Set the `PistonBehavior` for the block
                (_, Opcode::Getstatic(member))
                    if member.class_name == "net/minecraft/block/piston/PistonBehavior" =>
                {
                    match member.name_and_type.name.as_ref() {
                        "NORMAL" => block.piston_behaviour = PistonBehaviour::Normal,
                        "DESTROY" => block.piston_behaviour = PistonBehaviour::Destroy,
                        "BLOCK" => block.piston_behaviour = PistonBehaviour::Block,
                        "IGNORE" => block.piston_behaviour = PistonBehaviour::Ignore,
                        "PUSH_ONLY" => block.piston_behaviour = PistonBehaviour::PushOnly,
                        other => unreachable!("Unknown piston behaviour: \"{other}\""),
                    }
                }
                // Handle various settings functions
                (_, Opcode::Invokevirtual(member))
                    if member.class_name == "net/minecraft/block/Block$Settings" => {}
                _other => {}
            }

            index += 1;
        });
        tracing::trace!("VVV");

        if identifier.is_empty() {
            anyhow::bail!("BlockData: Unable to find block identifier!");
        }

        tracing::info!("Found Block: \"{}\"", identifier);
        tracing::debug!("{block:#?}");
        tracing::info!("---");

        Ok((identifier, block))
    }

    #[expect(clippy::match_same_arms)]
    fn handle_blocktype(block_type: &str, _block: &mut BlockData) {
        match block_type {
            "net/minecraft/block/AmethystClusterBlock" => {}
            "net/minecraft/block/AttachedStemBlock" => {}
            "net/minecraft/block/BannerBlock" => {}
            "net/minecraft/block/BrushableBlock" => {}
            "net/minecraft/block/ButtonBlock" => {}
            "net/minecraft/block/CampfireBlock" => {}
            "net/minecraft/block/CandleCakeBlock" => {}
            "net/minecraft/block/ChestBlock" => {}
            "net/minecraft/block/ChorusFlowerBlock" => {}
            "net/minecraft/block/ColoredFallingBlock" => {}
            "net/minecraft/block/CommandBlock" => {}
            "net/minecraft/block/ConcretePowderBlock" => {}
            "net/minecraft/block/CoralBlock" => {}
            "net/minecraft/block/CoralBlockBlock" => {}
            "net/minecraft/block/CoralFanBlock" => {}
            "net/minecraft/block/CoralWallFanBlock" => {}
            "net/minecraft/block/DoorBlock" => {}
            "net/minecraft/block/DyedCarpetBlock" => {}
            "net/minecraft/block/ExperienceDroppingBlock" => {}
            "net/minecraft/block/EyeblossomBlock" => {}
            "net/minecraft/block/FenceGateBlock" => {}
            "net/minecraft/block/FlowerBlock" => {}
            "net/minecraft/block/FlowerPotBlock" => {}
            "net/minecraft/block/FluidBlock" => {}
            "net/minecraft/block/FungusBlock" => {}
            "net/minecraft/block/HangingSignBlock" => {}
            "net/minecraft/block/InfestedBlock" => {}
            "net/minecraft/block/LeveledCauldronBlock" => {}
            "net/minecraft/block/MossBlock" => {}
            "net/minecraft/block/MushroomPlantBlock" => {}
            "net/minecraft/block/OxidizableBlock" => {}
            "net/minecraft/block/OxidizableBulbBlock" => {}
            "net/minecraft/block/OxidizableDoorBlock" => {}
            "net/minecraft/block/OxidizableGrateBlock" => {}
            "net/minecraft/block/OxidizableSlabBlock" => {}
            "net/minecraft/block/OxidizableStairsBlock" => {}
            "net/minecraft/block/OxidizableTrapdoorBlock" => {}
            "net/minecraft/block/ParticleLeavesBlock" => {}
            "net/minecraft/block/PistonBlock" => {}
            "net/minecraft/block/PressurePlateBlock" => {}
            "net/minecraft/block/PropaguleBlock" => {}
            "net/minecraft/block/RotatedInfestedBlock" => {}
            "net/minecraft/block/SaplingBlock" => {}
            "net/minecraft/block/ShulkerBoxBlock" => {}
            "net/minecraft/block/SignBlock" => {}
            "net/minecraft/block/SkullBlock" => {}
            "net/minecraft/block/SlabBlock" => {}
            "net/minecraft/block/StainedGlassPaneBlock" => {}
            "net/minecraft/block/StairsBlock" => {}
            "net/minecraft/block/StemBlock" => {}
            "net/minecraft/block/TorchBlock" => {}
            "net/minecraft/block/TrapdoorBlock" => {}
            "net/minecraft/block/TripwireBlock" => {}
            "net/minecraft/block/WallBannerBlock" => {}
            "net/minecraft/block/WallBlock" => {}
            "net/minecraft/block/WallHangingSignBlock" => {}
            "net/minecraft/block/WallSignBlock" => {}
            "net/minecraft/block/WallSkullBlock" => {}
            "net/minecraft/block/WallTorchBlock" => {}
            "net/minecraft/block/WeightedPressurePlateBlock" => {}
            "net/minecraft/block/WitherRoseBlock" => {}
            // Ignore these
            "net/minecraft/block/MultifaceGrower"
            | "net/minecraft/block/SculkVeinBlock$SculkVeinGrowChecker" => {}
            // Warn about unknown block classes
            unk if unk.starts_with("net/minecraft/block") => {
                tracing::warn!("Unknown block class: \"{unk}\"");
            }
            _ => {}
        }
    }
}
