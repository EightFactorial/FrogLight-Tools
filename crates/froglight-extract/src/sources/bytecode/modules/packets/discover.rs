use anyhow::bail;
use cafebabe::{
    bytecode::Opcode,
    constant_pool::{LiteralConstant, Loadable, MemberRef},
};
use hashbrown::HashMap;

use super::Packets;
use crate::{
    bundle::ExtractBundle,
    bytecode::ClassContainer,
    sources::{get_class_field, get_class_method_code, get_field_signature},
};

impl Packets {
    /// Packet registry names and classes
    const PACKET_CLASSES: [&'static str; 8] = [
        "net/minecraft/network/packet/LoginPackets",
        "net/minecraft/network/packet/CommonPackets",
        "net/minecraft/network/packet/PingPackets",
        "net/minecraft/network/packet/HandshakePackets",
        "net/minecraft/network/packet/CookiePackets",
        "net/minecraft/network/packet/PlayPackets",
        "net/minecraft/network/packet/StatusPackets",
        "net/minecraft/network/packet/ConfigPackets",
    ];

    const REGISTRY_METHOD: &'static str = "<clinit>";

    const PACKET_OBJ_TYPE: &'static str = "Lnet/minecraft/network/packet/PacketType;";

    pub(super) fn discover_classes(
        data: &ExtractBundle<'_>,
    ) -> anyhow::Result<HashMap<String, String>> {
        let mut class_map = HashMap::new();

        for class in Self::PACKET_CLASSES {
            let Some(classfile) = data.jar_container.get(class).map(ClassContainer::parse) else {
                bail!("Packet class \"{class}\" not found in jar");
            };
            let Some(code) = get_class_method_code(&classfile, Self::REGISTRY_METHOD) else {
                bail!("Packet class \"{class}\" has no \"{}\" method", Self::REGISTRY_METHOD);
            };

            // Get the packet registry key and static field name
            //
            // "minecraft:game_profile": "GAME_PROFILE"
            let mut name = None;
            let mut name_map = HashMap::new();
            for (_, op) in &code.opcodes {
                match op {
                    // Find the packet registry name
                    Opcode::LdcW(Loadable::LiteralConstant(LiteralConstant::String(
                        const_name,
                    )))
                    | Opcode::Ldc(Loadable::LiteralConstant(LiteralConstant::String(const_name))) =>
                    {
                        name = Some(format!("minecraft:{const_name}"));
                    }
                    // Find the packet type
                    Opcode::Putstatic(MemberRef { class_name, name_and_type }) => {
                        if name_and_type.descriptor == Self::PACKET_OBJ_TYPE {
                            if let Some(name) = name.take() {
                                name_map.insert(name, &name_and_type.name);
                            } else {
                                bail!("Failed to identify packet name in \"{class_name}\"");
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Use the field name to get the packet type
            //
            // "minecraft:game_profile":
            // "net/minecraft/network/packet/s2c/login/LoginSuccessS2CPacket"
            for (packet_key, field_name) in name_map {
                // Find the field that matches the field name found earlier
                let Some(field) = get_class_field(&classfile, field_name) else {
                    bail!("Failed to get field \"{field_name}\" in \"{}\"", classfile.this_class);
                };

                // Get the field signature
                let Some(signature) = get_field_signature(field) else {
                    bail!(
                        "Failed to get field signature for \"{field_name}\" in \"{}\"",
                        classfile.this_class
                    );
                };

                // Get the real packet type from the field descriptor
                let descriptor = signature.split("<L").last().unwrap().split(';').next().unwrap();
                class_map.insert(packet_key, descriptor.to_string());
            }
        }

        Ok(class_map)
    }
}
