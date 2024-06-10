use cafebabe::{
    attributes::AttributeData,
    bytecode::Opcode,
    constant_pool::{LiteralConstant, Loadable, MemberRef},
};
use hashbrown::HashMap;
use tracing::error;

use super::Packets;
use crate::{bundle::ExtractBundle, sources::get_method_code};

impl Packets {
    /// Append the packet class to the output.
    ///
    /// All packets must be iterated over,
    /// since the same packet can be used multiple times.
    pub(super) fn append_packet_class(
        packet: &str,
        class: &str,
        data: &mut ExtractBundle<'_>,
    ) -> bool {
        let mut found_matching = false;
        if let Some(packets) = data.output["packets"].as_object_mut() {
            for state in packets.values_mut() {
                if let Some(state) = state.as_object_mut() {
                    for direction in state.values_mut() {
                        if let Some(packet_data) = direction.get_mut(packet) {
                            if packet_data.get("java_class").is_none() {
                                packet_data["java_class"] = class.to_string().into();
                                found_matching = true;
                            } else {
                                error!("Packet class already set for \"{packet}\"?");
                            }
                        }
                    }
                }
            }
        }
        found_matching
    }

    const STATIC_METHOD: &'static str = "<clinit>";
    const PACKET_TYPE: &'static str = "Lnet/minecraft/network/packet/PacketType;";

    /// Extract packets from a packet registry class.
    ///
    /// Returns `None` if the class cannot be found,
    /// if there are no packets, or if there are any errors.
    pub(super) fn packets_in_class(
        class: &str,
        data: &ExtractBundle<'_>,
    ) -> Option<HashMap<String, String>> {
        let classfile = data.jar_container.get(class)?.parse();
        let code = get_method_code(&classfile, Self::STATIC_METHOD)?;

        let mut map = HashMap::new();

        // Get the packet registry key and static field name
        //
        // "minecraft:game_profile": "GAME_PROFILE"
        let mut packet_name = None;
        for (_, op) in &code.opcodes {
            match op {
                Opcode::LdcW(Loadable::LiteralConstant(LiteralConstant::String(name)))
                | Opcode::Ldc(Loadable::LiteralConstant(LiteralConstant::String(name))) => {
                    packet_name = Some(format!("minecraft:{name}"));
                }
                Opcode::Putstatic(MemberRef { class_name, name_and_type }) => {
                    if name_and_type.descriptor == Self::PACKET_TYPE {
                        if let Some(packet_name) = packet_name.take() {
                            map.insert(packet_name, name_and_type.name.to_string());
                        } else {
                            error!("Failed to identify packet name in \"{class_name}\"");
                            return None;
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
        for (_, field_name) in &mut map {
            // Find the field that matches the field name found earlier
            if let Some(field) = classfile.fields.iter().find(|field| field.name == *field_name) {
                // Find the signature attribute of the field
                if let Some(attribute) =
                    field.attributes.iter().find(|attr| attr.name == "Signature")
                {
                    let AttributeData::Signature(signature) = &attribute.data else {
                        error!("Failed to identify field signature for \"{field_name}\"");
                        return None;
                    };

                    // Get the real packet type from the field descriptor
                    let descriptor = signature.split("<L").last()?.split(';').next()?;
                    *field_name = descriptor.to_string();
                } else {
                    error!("Failed to identify field descriptor for \"{field_name}\"");
                    return None;
                }
            } else {
                error!("Failed to identify packet class for \"{field_name}\"");
                return None;
            }
        }

        // Return `None` if no packets were found
        if map.is_empty() {
            error!("Failed to identify packets in \"{class}\"");
            None
        } else {
            Some(map)
        }
    }
}
