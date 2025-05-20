use std::collections::HashMap;

use cafebabe::{bytecode::Opcode, constant_pool::MemberRef, ClassFile};
use froglight_dependency::{
    container::DependencyContainer,
    dependency::minecraft::{minecraft_code::CodeBundle, MinecraftCode},
    version::Version,
};
use tracing::error;

use super::Packets;
use crate::class_helper::ClassHelper;

impl Packets {
    pub(super) async fn extract_packet_classes(
        version: &Version,
        deps: &mut DependencyContainer,
    ) -> anyhow::Result<Vec<String>> {
        let _packets = Self::extract_state_packets(version, deps).await?;

        Ok(Vec::new())
    }

    async fn extract_state_packets(
        version: &Version,
        deps: &mut DependencyContainer,
    ) -> anyhow::Result<HashMap<String, Vec<String>>> {
        const PACKET_STATE_PREFIX: &str = "net/minecraft/network/state/";

        let mut packets = HashMap::with_capacity(5);
        let states = Self::extract_network_states(version, deps).await?;

        deps.get_or_retrieve::<MinecraftCode>().await?;
        deps.scoped_fut::<MinecraftCode, anyhow::Result<_>>(async |jars, deps| {
            let jar = jars.get_version(version, deps).await?;

            for state in states {
                let mut class_iter = jar.get_filter(|(n, _)| {
                    n.strip_prefix(PACKET_STATE_PREFIX).map_or(false, |s| {
                        !s.contains(['/', '$'])
                            && if state == "status" {
                                s.to_lowercase().contains("query")
                            } else {
                                s.to_lowercase().contains(&state)
                            }
                    })
                });

                if let Some(class) = class_iter.next() {
                    packets.insert(
                        state.clone(),
                        Self::extract_state_packet_classes(class, jar).await?,
                    );
                } else {
                    anyhow::bail!("Network state \"{state}\" didn't match any classes!");
                }

                if class_iter.next().is_some() {
                    anyhow::bail!("Network state \"{state}\" matched multiple classes!");
                }
            }

            Ok(())
        })
        .await?;

        Ok(packets)
    }

    async fn extract_state_packet_classes(
        class: ClassFile<'_>,
        jar: &CodeBundle,
    ) -> anyhow::Result<Vec<String>> {
        const NETWORK_STATE_BUILDER: &str = "net/minecraft/network/state/NetworkStateBuilder";

        let packets = Vec::new();

        let mut direction = None;

        let initial = class.class_code().bytecode.as_ref().unwrap();
        let initial = initial.opcodes.iter().map(|(_, opcode)| opcode).collect::<Vec<_>>();
        class.iter_code_recursive(&initial, jar, |opcode| match opcode {
            Opcode::Invokestatic(MemberRef { class_name, name_and_type })
                if class_name == NETWORK_STATE_BUILDER =>
            {
                match name_and_type.name.as_ref() {
                    "c2s" => direction = Some(true),
                    "s2c" => direction = Some(false),
                    unk => error!("Unknown NetworkStateBuilder method: \"{unk}\""),
                }
            }
            Opcode::Invokeinterface(MemberRef { class_name, name_and_type }, _)
                if class_name == NETWORK_STATE_BUILDER =>
            {
                match name_and_type.name.as_ref() {
                    "add" => {}
                    "addBundle" => {}
                    unk => error!("Unknown NetworkStateBuilder method: \"{unk}\""),
                }
            }
            Opcode::Getstatic(MemberRef { .. }) => {}
            _ => {}
        });

        Ok(packets)
    }
}
