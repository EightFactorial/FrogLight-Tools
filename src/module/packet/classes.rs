use std::collections::HashMap;

use cafebabe::{
    bytecode::Opcode,
    constant_pool::{LiteralConstant, Loadable, MemberRef},
    ClassFile,
};
use froglight_dependency::{
    container::DependencyContainer,
    dependency::minecraft::{minecraft_code::CodeBundle, MinecraftCode},
    version::Version,
};
use indexmap::IndexMap;

use super::Packets;
use crate::class_helper::ClassHelper;

impl Packets {
    pub(super) async fn extract_packet_classes(
        version: &Version,
        deps: &mut DependencyContainer,
    ) -> anyhow::Result<HashMap<String, NetworkState>> {
        const STATE_PATH: &str = "net/minecraft/network/state/";

        let mut states = HashMap::new();

        deps.get_or_retrieve::<MinecraftCode>().await?;
        deps.scoped_fut::<MinecraftCode, anyhow::Result<()>>(
            async |code: &mut MinecraftCode, deps| {
                let bundle = code.get_version(version, deps).await?;

                for class in
                    bundle.get_filter(|(c, _)| c.starts_with(STATE_PATH) && !c.contains('$'))
                {
                    if class.methods.iter().any(|m| m.name == "<clinit>") {
                        states.extend_one(Self::extract_state_packets(class, bundle).await?);
                    }
                }

                Ok(())
            },
        )
        .await?;

        Ok(states)
    }

    async fn extract_state_packets(
        class: ClassFile<'_>,
        classes: &CodeBundle,
    ) -> anyhow::Result<(String, NetworkState)> {
        const NETWORK_PHASE: &str = "net/minecraft/network/NetworkPhase";

        // -= 1.21.4
        const NETWORK_STATE_BUILDER_OLD: &str = "net/minecraft/network/NetworkStateBuilder";
        // += 1.21.5
        const NETWORK_STATE_BUILDER: &str = "net/minecraft/network/state/NetworkStateBuilder";

        // -= 1.21.4
        const NETWORK_STATE_FACTORY_DESCRIPTOR_OLD: &str =
            "Lnet/minecraft/network/NetworkState$Factory;";
        // += 1.21.5
        const NETWORK_STATE_FACTORY_DESCRIPTOR: &str =
            "Lnet/minecraft/network/state/NetworkStateFactory;";
        // += 1.21.5
        const AWARE_NETWORK_STATE_FACTORY_DESCRIPTOR: &str =
            "Lnet/minecraft/network/state/ContextAwareNetworkStateFactory;";

        const PACKET_CODEC_DESCRIPTOR: &str = "Lnet/minecraft/network/codec/PacketCodec;";
        const PACKET_TYPE_DESCRIPTOR: &str = "Lnet/minecraft/network/packet/PacketType;";

        let mut state_name = Option::None;
        let mut state = NetworkState::default();

        let mut dir_name = Option::None;
        let mut packet_name = Option::None;
        let mut packets = IndexMap::new();

        let initial = class.class_code().bytecode.as_ref().unwrap();
        let initial = initial.opcodes.iter().map(|(_, opcode)| opcode).collect::<Vec<_>>();
        class.iter_code_recursive(&initial, classes, |op| match op {
            Opcode::Invokestatic(MemberRef { class_name, name_and_type })
                if (class_name == NETWORK_STATE_BUILDER_OLD
                    || class_name == NETWORK_STATE_BUILDER) =>
            {
                match name_and_type.name.as_ref() {
                    "s2c" => dir_name = Some("s2c"),
                    "c2s" | "contextAwareC2S" => dir_name = Some("c2s"),
                    unk => panic!("PacketStateBuilder: Unknown method `{unk}`!"),
                }
            }
            Opcode::Getstatic(MemberRef { class_name, name_and_type })
                if class_name == NETWORK_PHASE =>
            {
                state_name = Some(Self::phase_name(&name_and_type.name.to_lowercase()));
            }
            Opcode::Getstatic(MemberRef { class_name, name_and_type })
                if name_and_type.descriptor == PACKET_TYPE_DESCRIPTOR =>
            {
                packet_name = Some(Self::packet_name(class_name, &name_and_type.name, classes));
            }
            Opcode::Getstatic(MemberRef { class_name, name_and_type })
                if name_and_type.descriptor == PACKET_CODEC_DESCRIPTOR =>
            {
                match core::mem::take(&mut packet_name) {
                    Some(name) => packets.insert(
                        format!("minecraft:{name}"),
                        PacketClass {
                            class: class_name.to_string(),
                            codec: Some(name_and_type.name.to_string()),
                        },
                    ),
                    None => panic!("PacketStateBuilder: Could not find packet name!"),
                };
            }
            Opcode::Invokespecial(MemberRef { class_name, .. })
                if class_name
                    == "net/minecraft/network/packet/s2c/play/BundleDelimiterS2CPacket" =>
            {
                packets.insert(
                    String::from("minecraft:bundle"),
                    PacketClass { class: class_name.to_string(), codec: None },
                );
            }
            Opcode::Putstatic(MemberRef { name_and_type, .. })
                if (name_and_type.descriptor == NETWORK_STATE_FACTORY_DESCRIPTOR_OLD
                    || name_and_type.descriptor == NETWORK_STATE_FACTORY_DESCRIPTOR
                    || name_and_type.descriptor == AWARE_NETWORK_STATE_FACTORY_DESCRIPTOR) =>
            {
                match dir_name {
                    Some("c2s") => state.c2s = core::mem::take(&mut packets),
                    Some("s2c") => state.s2c = core::mem::take(&mut packets),
                    Some(..) => unreachable!(),
                    None => panic!("PacketStateBuilder: Directory name is None!"),
                }
            }
            _ => {}
        });

        Ok((state_name.expect("PacketStateBuilder: Name is None!"), state))
    }

    fn phase_name(state_name: &str) -> String {
        match state_name {
            "handshaking" => String::from("handshake"),
            "configuration" => String::from("config"),
            other => other.to_string(),
        }
    }

    fn packet_name(class_name: &str, field_name: &str, classes: &CodeBundle) -> String {
        let Some(class) = classes.get(class_name) else {
            panic!("PacketStateBuilder: Could not find class `{class_name}`!");
        };

        let mut temp_constant = Option::<String>::None;
        let mut constant = Option::<String>::None;

        let initial = class.class_code().bytecode.as_ref().unwrap();
        let initial = initial.opcodes.iter().map(|(_, opcode)| opcode).collect::<Vec<_>>();
        class.iter_code_recursive(&initial, classes, |op| match op {
            Opcode::Ldc(Loadable::LiteralConstant(LiteralConstant::String(constant)))
            | Opcode::LdcW(Loadable::LiteralConstant(LiteralConstant::String(constant)))
            | Opcode::Ldc2W(Loadable::LiteralConstant(LiteralConstant::String(constant)))
                if temp_constant.is_none() =>
            {
                temp_constant = Some(constant.to_string());
            }
            Opcode::Putstatic(MemberRef { class_name, name_and_type })
                if **class_name == *class.this_class && name_and_type.name == field_name =>
            {
                constant = core::mem::take(&mut temp_constant);
            }
            Opcode::Putstatic(..) => temp_constant = None,
            _ => {}
        });

        constant.expect("PacketStateBuilder: Could not find Packet identifier!")
    }
}

// -------------------------------------------------------------------------------------------------

#[derive(Debug, Default)]
pub(super) struct NetworkState {
    pub(super) c2s: IndexMap<String, PacketClass>,
    pub(super) s2c: IndexMap<String, PacketClass>,
}

#[derive(Debug)]
pub(super) struct PacketClass {
    pub(super) class: String,
    pub(super) codec: Option<String>,
}
