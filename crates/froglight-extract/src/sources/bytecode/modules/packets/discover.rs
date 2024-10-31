use anyhow::bail;
use cafebabe::{
    bytecode::Opcode,
    constant_pool::{LiteralConstant, Loadable, MemberRef},
};

use super::Packets;
use crate::{
    bundle::ExtractBundle,
    sources::helpers::{get_class_field, get_class_method, get_code, get_signature},
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

    const REGISTRY_METHOD_NAME: &'static str = "<clinit>";
    const PACKETTYPE_OBJECT: &'static str = "Lnet/minecraft/network/packet/PacketType;";

    pub(super) fn discover_classes(data: &ExtractBundle) -> anyhow::Result<Vec<(String, String)>> {
        let mut class_list = Vec::new();

        for class in Self::PACKET_CLASSES {
            let classfile = data.jar_container.get_class_err(class)?;
            let method = get_class_method(&classfile, Self::REGISTRY_METHOD_NAME, None)?;
            let code = get_code(&method.attributes)?;

            // Get the packet registry key and static field name
            //
            // ("minecraft:game_profile", "GAME_PROFILE")
            let mut name = None;
            let mut name_list = Vec::new();
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
                    // Find the packet field name
                    Opcode::Putstatic(MemberRef { class_name, name_and_type }) => {
                        if name_and_type.descriptor == Self::PACKETTYPE_OBJECT {
                            if let Some(name) = name.take() {
                                name_list.push((name, name_and_type.name.to_string()));
                            } else {
                                bail!("Failed to identify packet name in \"{class_name}\"");
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Use the field name to get the packet class
            //
            // ("minecraft:game_profile":
            // "net/minecraft/network/packet/s2c/login/LoginSuccessS2CPacket")
            for (_, field_name) in &mut name_list {
                // Find the field that matches the field name found earlier
                let field = get_class_field(&classfile, field_name)?;
                let signature = get_signature(&field.attributes)?;

                // Get the real packet class from the field descriptor
                let descriptor = signature.split("<L").last().unwrap().split(';').next().unwrap();
                *field_name = descriptor.to_string();
            }
            class_list.extend(name_list);
        }

        Ok(class_list)
    }
}
