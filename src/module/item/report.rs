use std::{collections::HashMap, ops::RangeInclusive, path::Path};

use cafebabe::{
    bytecode::Opcode,
    constant_pool::{LiteralConstant, Loadable, MemberRef, NameAndType},
    ClassFile,
};
use froglight_dependency::{
    container::{Dependency, DependencyContainer},
    dependency::minecraft::{minecraft_code::CodeBundle, DataGenerator, MinecraftCode},
    version::Version,
};
use indexmap::IndexMap;

use crate::class_helper::ClassHelper;

/// A collection of [`ItemReport`]s.
#[derive(Default, Dependency)]
pub(crate) struct ItemReports(HashMap<Version, ItemReport>);

impl ItemReports {
    /// Get the [`ItemReport`] for the given version.
    ///
    /// Returns `None` if the report does not already exist.
    #[inline]
    #[must_use]
    pub(crate) fn version(&self, version: &Version) -> Option<&ItemReport> { self.0.get(version) }

    /// Retrieve the [`ItemReport`] for the given version.
    ///
    /// # Errors
    /// Returns an error if the report could not be retrieved.
    pub(crate) async fn get_version(
        &mut self,
        version: &Version,
        deps: &mut DependencyContainer,
    ) -> anyhow::Result<&ItemReport> {
        if !self.0.contains_key(version) {
            deps.get_or_retrieve::<MinecraftCode>().await?;
            deps.scoped_fut::<MinecraftCode, anyhow::Result<()>>(
                async |code: &mut MinecraftCode, deps| {
                    let bundle = code.get_version(version, deps).await?;
                    self.0.insert(version.clone(), Self::parse_class(bundle).await?);
                    Ok(())
                },
            )
            .await?;
        }

        Ok(self.version(version).unwrap())
    }

    #[expect(clippy::unused_async)]
    async fn parse_class(classes: &CodeBundle) -> anyhow::Result<ItemReport> {
        let items = classes
            .get("net/minecraft/item/Items")
            .ok_or_else(|| anyhow::anyhow!("Could not find class \"net/minecraft/item/Items\"!"))?;
        let blocks = classes.get("net/minecraft/block/Blocks").ok_or_else(|| {
            anyhow::anyhow!("Could not find class \"net/minecraft/block/Blocks\"!")
        })?;

        let mut report = ItemReport::default();

        let mut name = Option::<String>::None;
        let mut rarity = ItemRarity::default();

        let initial = items.class_code().bytecode.as_ref().unwrap();
        let initial = initial.opcodes.iter().map(|(_, opcode)| opcode).collect::<Vec<_>>();
        items.iter_code_recursive(&initial, classes, |op| match op {
            Opcode::Getstatic(MemberRef { class_name, name_and_type }) => {
                if name.is_none() && name_and_type.descriptor == "Lnet/minecraft/block/Block;" {
                    name = Some(Self::block_name(&blocks, name_and_type));
                }

                if class_name == "net/minecraft/util/Rarity" {
                    match name_and_type.name.as_ref() {
                        "COMMON" => rarity = ItemRarity::Common,
                        "UNCOMMON" => rarity = ItemRarity::Uncommon,
                        "RARE" => rarity = ItemRarity::Rare,
                        "EPIC" => rarity = ItemRarity::Epic,
                        unk => panic!("Unknown rarity: \"{unk}\""),
                    }
                }
            }
            Opcode::LdcW(Loadable::LiteralConstant(LiteralConstant::String(string))) => {
                if name.is_none() {
                    name = Some(string.to_string());
                }
            }
            Opcode::Putstatic(MemberRef { class_name, name_and_type }) => {
                if name_and_type.descriptor == "Lnet/minecraft/item/Item;"
                    && class_name == &items.this_class
                {
                    let name = name.take().expect("Could not find name!");
                    let rarity = std::mem::take(&mut rarity);

                    report.0.insert(format!("minecraft:{name}"), ItemReportEntry { rarity });
                }
            }
            _ => {}
        });

        Ok(report)
    }

    fn block_name(blocks: &ClassFile, block: &NameAndType) -> String {
        let mut name = Option::<String>::None;

        let code = blocks.class_code().bytecode.as_ref().unwrap();
        for (_, op) in &code.opcodes {
            match op {
                Opcode::LdcW(Loadable::LiteralConstant(LiteralConstant::String(string))) => {
                    name = Some(string.to_string());
                }
                Opcode::Putstatic(MemberRef { name_and_type, .. }) => {
                    if name_and_type.name == block.name {
                        return name.expect("Created block but did not find name!");
                    }
                }

                _ => {}
            }
        }

        panic!("Could not find block name!")
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct ItemReport(pub IndexMap<String, ItemReportEntry>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ItemReportEntry {
    pub(crate) rarity: ItemRarity,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ItemRarity {
    #[default]
    Common,
    Uncommon,
    Rare,
    Epic,
}
