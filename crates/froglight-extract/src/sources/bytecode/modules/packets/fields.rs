use std::borrow::Cow;

use cafebabe::{
    attributes::AttributeData,
    bytecode::Opcode,
    constant_pool::{BootstrapArgument, InvokeDynamic, MemberRef, NameAndType},
    descriptor::{FieldType, Ty},
    ClassFile, MethodInfo,
};
use tracing::{debug, error, trace, warn};

use super::Packets;
use crate::{
    bundle::ExtractBundle,
    sources::{bytecode::traits::FieldTypeTrait, get_method_code},
};

impl Packets {
    pub(super) fn get_packet_fields(
        packets: Vec<(String, String)>,
        data: &ExtractBundle<'_>,
    ) -> Option<Vec<(String, Vec<String>)>> {
        let mut packet_fields = Vec::with_capacity(packets.len());
        for (packet_key, packet_class) in packets {
            let Some(fields) = Self::packet_fields(&packet_class, data) else {
                error!("Failed to get packet fields for \"{packet_key}\"");
                return None;
            };
            packet_fields.push((packet_key, fields));
        }
        Some(packet_fields)
    }

    const CODEC_FIELD_NAME: &'static str = "CODEC";

    /// Get the fields of a packet class
    fn packet_fields(packet_class: &str, data: &ExtractBundle<'_>) -> Option<Vec<String>> {
        let classfile = data.jar_container.get(packet_class)?.parse();

        let Some(field) =
            classfile.fields.iter().find(|field| field.name == Self::CODEC_FIELD_NAME)
        else {
            error!("Failed to find codec field in \"{packet_class}\"");
            return Some(Vec::new());
        };
        let Some(attribute) = field.attributes.iter().find(|a| a.name == "Signature") else {
            error!("Failed to find signature attribute in codec field");
            return None;
        };
        let AttributeData::Signature(signature) = &attribute.data else {
            error!("Failed to get signature info from codec field");
            return None;
        };

        let parameters_method = signature
            .trim_start_matches("Lnet/minecraft/network/codec/PacketCodec<")
            .trim_end_matches(";>;");
        let parameters = format!("{};", parameters_method.split_once(";L")?.0);
        let parameters = Cow::Borrowed(parameters.as_str());
        let parameters = FieldType::parse(&parameters)?;

        // TODO: Detect codec type and construction, choose the correct method to parse

        // Find and parse the method with matching parameters
        if let Some(method) = classfile
            .methods
            .iter()
            .find(|method| method.descriptor.parameters == vec![parameters.clone()])
        {
            return Self::fields_from_opcodes(&classfile, method);
        }

        // Unit packets have no fields
        if parameters == FieldType::Ty(Ty::Object(Cow::Borrowed("io/netty/buffer/ByteBuf"))) {
            warn!("Is \"{packet_class}\" a Unit?");
            return Some(Vec::new());
        }

        // Log an error and return an empty list
        error!("Failed to find method for \"{packet_class}\"");
        debug!("Parameters: {parameters:?}");
        Some(Vec::new())
    }

    /// Get the fields from the opcodes of a method
    fn fields_from_opcodes(class: &ClassFile<'_>, method: &MethodInfo) -> Option<Vec<String>> {
        let code = get_method_code(method)?;
        let mut fields = Vec::new();

        trace!("Class: {}", class.this_class);
        for (_, op) in &code.opcodes {
            match op {
                // Resolve the dynamic method call and find the type, if any
                Opcode::Invokedynamic(InvokeDynamic { attr_index, name_and_type }) => {
                    if let Some(bootstrap) =
                        class.attributes.iter().find(|attr| attr.name == "BootstrapMethods")
                    {
                        let AttributeData::BootstrapMethods(bootstrap) = &bootstrap.data else {
                            error!("    Dyn Missing :: Mislabeled :: {}", name_and_type.name);
                            continue;
                        };

                        // Iterate over the bootstrap methods to find method handles
                        if let Some(attr) = bootstrap.get(*attr_index as usize) {
                            for arg in &attr.arguments {
                                if let BootstrapArgument::MethodHandle(handle) = arg {
                                    // Find the type read from the method call
                                    if let Some(field) =
                                        Self::name_to_type(&handle.class_name, &handle.member_ref)
                                    {
                                        fields.push(field);
                                    }
                                }
                            }
                        } else {
                            error!("    Dyn Missing :: {attr_index} :: {}", name_and_type.name);
                        }
                    } else {
                        error!("    Dyn Missing :: None :: {}", name_and_type.name);
                    }
                }
                // Find the type read from the method call, if any
                Opcode::Invokeinterface(MemberRef { class_name, name_and_type }, ..)
                | Opcode::Invokespecial(MemberRef { class_name, name_and_type })
                | Opcode::Invokestatic(MemberRef { class_name, name_and_type })
                | Opcode::Invokevirtual(MemberRef { class_name, name_and_type }) => {
                    if let Some(field) = Self::name_to_type(class_name, name_and_type) {
                        fields.push(field);
                    }
                }
                _ => {}
            }
        }

        Some(fields)
    }

    /// Get the returned type from:
    /// 1. class that owns the method
    /// 2. the name of the method
    fn name_to_type(class_name: &str, name_and_type: &NameAndType) -> Option<String> {
        match class_name {
            "io/netty/buffer/ByteBuf"
            | "net/minecraft/network/PacketByteBuf"
            | "net/minecraft/network/RegistryByteBuf" => match &*name_and_type.name {
                "decodeAsJson" => Some(String::from("Json")),
                "readBlockPos" => Some(String::from("BlockPos")),
                "readBlockHitResult" => Some(String::from("BlockHitResult")),
                "readBoolean" => Some(String::from("bool")),
                "readByte" | "readUnsignedByte" => Some(String::from("u8")),
                "readByteArray" => Some(String::from("Vec<u8>")),
                "readChunkPos" => Some(String::from("ChunkPos")),
                "readCollection" | "readList" => Some(String::from("Vec")),
                "readDouble" => Some(String::from("f64")),
                "readEnumConstant" => Some(String::from("Enum")),
                "readEnumSet" => Some(String::from("BitSet")),
                "readFloat" => Some(String::from("f32")),
                "readIdentifier" => Some(String::from("ResourceKey")),
                "readInstant" | "readLong" => Some(String::from("i64")),
                "readInt" => Some(String::from("i32")),
                "readIntArray" | "readIntList" => Some(String::from("Vec<i32>")),
                "readLongArray" => Some(String::from("Vec<i64>")),
                "readMap" => Some(String::from("HashMap")),
                "readNbt" => Some(String::from("Nbt")),
                "readNullable" | "readOptional" => Some(String::from("Option")),
                "readShort" => Some(String::from("i16")),
                "readString" => Some(String::from("String")),
                "readUnsignedShort" => Some(String::from("u16")),
                "readUuid" => Some(String::from("Uuid")),
                "readVarInt" => Some(String::from("VarInt")),
                "readVarLong" => Some(String::from("VarLong")),
                _ => {
                    trace!("    {class_name} :: {}", name_and_type.name);
                    None
                }
            },
            "net/minecraft/network/packet/c2s/handshake/ConnectionIntent"
            | "net/minecraft/util/math/Direction"
            | "net/minecraft/world/Difficulty" => {
                if name_and_type.name == "byId" {
                    Some(String::from("VarInt"))
                } else {
                    trace!("    {class_name} :: {}", name_and_type.name);
                    None
                }
            }
            "net/minecraft/entity/EquipmentSlot" => {
                if name_and_type.name == "values" {
                    Some(String::from("Equipment"))
                } else {
                    trace!("    {class_name} :: {}", name_and_type.name);
                    None
                }
            }
            "net/minecraft/network/packet/s2c/login/LoginQueryRequestS2CPacket"
            | "net/minecraft/network/packet/c2s/login/LoginQueryResponseC2SPacket" => {
                if name_and_type.name == "readPayload" {
                    Some(String::from("UnsizedByteBuffer"))
                } else {
                    trace!("    {class_name} :: {}", name_and_type.name);
                    None
                }
            }
            "net/minecraft/network/codec/PacketCodecs" => {
                if name_and_type.name == "registryValue" {
                    Some(String::from("VarInt"))
                } else {
                    trace!("    {class_name} :: {}", name_and_type.name);
                    None
                }
            }
            "net/minecraft/text/Text$Serialization" => {
                if name_and_type.name == "fromLenientJson" {
                    Some(String::from("Text"))
                } else {
                    trace!("    {class_name} :: {}", name_and_type.name);
                    None
                }
            }
            "com/google/common/collect/Lists"
            | "net/minecraft/network/codec/PacketCodec"
            | "net/minecraft/util/math/MathHelper"
            | "com/mojang/datafixers/util/Pair"
            | "java/util/function/Function"
            | "java/util/List"
            | "java/util/Optional" => None,
            _ => {
                trace!("    {class_name} :: {}", name_and_type.name);
                None
            }
        }
    }

    const FIELDS_ENTRY: &'static str = "fields";

    /// Append the packet fields to the output.
    ///
    /// All packets must be iterated over,
    /// since the same packet can be used multiple times.
    pub(super) fn append_packet_fields(
        packet: &str,
        fields: Vec<String>,
        data: &mut ExtractBundle<'_>,
    ) -> bool {
        let mut found_matching = false;
        if let Some(packets) = data.output["packets"].as_object_mut() {
            for state in packets.values_mut() {
                if let Some(state) = state.as_object_mut() {
                    for direction in state.values_mut() {
                        if let Some(packet_data) = direction.get_mut(packet) {
                            if packet_data.get(Self::FIELDS_ENTRY).is_none() {
                                packet_data[Self::FIELDS_ENTRY] = fields.clone().into();
                                found_matching = true;
                            } else {
                                error!("Packet fields already set for \"{packet}\"?");
                            }
                        }
                    }
                }
            }
        }
        found_matching
    }
}
