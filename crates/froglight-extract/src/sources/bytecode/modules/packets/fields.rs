use std::borrow::Cow;

use cafebabe::{
    attributes::AttributeData,
    bytecode::Opcode,
    constant_pool::{BootstrapArgument, InvokeDynamic, MemberRef, NameAndType},
    descriptor::FieldType,
    ClassFile, MethodInfo,
};
use tracing::{debug, error, trace, warn};

use super::{codec::CodecConstructor, Packets};
use crate::{
    bundle::ExtractBundle,
    sources::{bytecode::traits::FieldTypeTrait, get_method_code},
};

impl Packets {
    /// Get the fields of a packet class
    pub(super) fn get_packet_fields(
        packets: Vec<(String, String)>,
        data: &ExtractBundle<'_>,
    ) -> Option<Vec<(String, Vec<String>)>> {
        let mut packet_fields = Vec::with_capacity(packets.len());
        for (key, class) in packets {
            let Some(fields) = Self::packet_fields(&class, data) else {
                error!("Failed to get packet fields for \"{key}\"");
                return None;
            };
            packet_fields.push((key, fields));
        }
        Some(packet_fields)
    }

    /// Get the fields of a packet class
    fn packet_fields(class: &str, data: &ExtractBundle<'_>) -> Option<Vec<String>> {
        let classfile = data.jar_container.get(class)?.parse();
        trace!("Class: {}", classfile.this_class);

        // Detect codec type and construction, choose the correct method to parse
        match Self::get_codec_type(&classfile) {
            // Units have no fields
            Some(CodecConstructor::Unit) => Some(Vec::new()),
            // Parse the fields from the codec's `encode`/`decode` methods
            Some(CodecConstructor::CreateCodec(_)) => Self::parse_codec_method(&classfile, data),
            // TODO: Parse the tuple constructor
            Some(CodecConstructor::Tuple(_)) => {
                warn!("`Tuple` codec not implemented: \"{}\"", classfile.this_class);
                Some(Vec::new())
            }
            // TODO: Parse the xmap constructor
            Some(CodecConstructor::XMap(_)) => {
                warn!("`XMap` codec not implemented: \"{}\"", classfile.this_class);
                Some(Vec::new())
            }
            None => {
                // Don't emit a warning if the class is a bundle.
                // Bundles don't have a codec field.
                if !classfile.this_class.contains("Bundle") {
                    warn!(
                        "No `{}` field found in \"{}\"",
                        Self::CODEC_FIELD_NAME,
                        classfile.this_class
                    );
                }
                Some(Vec::new())
            }
        }
    }

    const SIGNATURE_ATTRIB_NAME: &'static str = "Signature";

    const CODEC_PREFIX: &'static str = "Lnet/minecraft/network/codec/PacketCodec<";
    const CODEC_SUFFIX: &'static str = ";>;";

    /// Look at the codec field signature to see what it was constructed using
    ///
    /// Then find a method that uses the same parameters
    /// and parse the fields from it.
    // TODO: I don't think is correct, it always uses a `PacketByteBuf`...
    fn parse_codec_method(
        classfile: &ClassFile<'_>,
        _data: &ExtractBundle<'_>,
    ) -> Option<Vec<String>> {
        let Some(field) =
            classfile.fields.iter().find(|&field| field.name == Self::CODEC_FIELD_NAME)
        else {
            error!("Failed to find codec field in \"{}\"", classfile.this_class);
            return Some(Vec::new());
        };
        let Some(attribute) =
            field.attributes.iter().find(|&a| a.name == Self::SIGNATURE_ATTRIB_NAME)
        else {
            error!("Failed to find signature attribute in codec field");
            return None;
        };
        let AttributeData::Signature(signature) = &attribute.data else {
            error!("Failed to get signature info from codec field");
            return None;
        };

        let parameters_method =
            signature.trim_start_matches(Self::CODEC_PREFIX).trim_end_matches(Self::CODEC_SUFFIX);
        let parameters = format!("{};", parameters_method.split_once(";L")?.0);
        let parameters = Cow::Borrowed(parameters.as_str());
        let parameters = FieldType::parse(&parameters)?;

        if let Some(fields) = classfile
            .methods
            .iter()
            .find(|&method| method.descriptor.parameters == vec![parameters.clone()])
            .and_then(|method| Self::fields_from_method(classfile, method))
        {
            Some(fields)
        } else {
            error!("Failed to find method for \"{}\"", classfile.this_class);
            debug!("Parameters: {parameters:?}");
            None
        }
    }

    const BOOTSRAP_FIELD_NAME: &'static str = "BootstrapMethods";

    /// Get a list of packet fields from a method
    ///
    /// Iterates over the opcodes of the method to find when
    /// specific methods are called, which are then used to
    /// determine what data types packets are composed of.
    ///
    /// A packet that calls `ByteBuffer.readString`
    /// *probably* contains a string.
    fn fields_from_method(class: &ClassFile<'_>, method: &MethodInfo) -> Option<Vec<String>> {
        let code = get_method_code(method)?;
        let mut fields = Vec::new();

        for (_, op) in &code.opcodes {
            match op {
                // Find the type read from the method call, if any
                Opcode::Invokeinterface(MemberRef { class_name, name_and_type }, ..)
                | Opcode::Invokespecial(MemberRef { class_name, name_and_type })
                | Opcode::Invokestatic(MemberRef { class_name, name_and_type })
                | Opcode::Invokevirtual(MemberRef { class_name, name_and_type }) => {
                    if let Some(field) = Self::name_to_type(class_name, name_and_type) {
                        fields.push(field);
                    }
                }
                // Resolve the dynamic method call and find the type, if any
                //
                // I'm only guessing `dynamic` means to look at the bootstrap methods...
                Opcode::Invokedynamic(InvokeDynamic { attr_index, name_and_type }) => {
                    if let Some(bootstrap) =
                        class.attributes.iter().find(|attr| attr.name == Self::BOOTSRAP_FIELD_NAME)
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
                _ => {}
            }
        }

        Some(fields)
    }

    const BYTEBUF_TYPE: &'static str = "io/netty/buffer/ByteBuf";
    const PACKETBYTEBUF_TYPE: &'static str = "net/minecraft/network/PacketByteBuf";
    const REGISTRYBYTEBUF_TYPE: &'static str = "net/minecraft/network/RegistryByteBuf";

    /// Determine the type by:
    /// 1. class that owns the method
    /// 2. the name of the method
    fn name_to_type(class_name: &str, name_and_type: &NameAndType) -> Option<String> {
        match class_name {
            Self::BYTEBUF_TYPE | Self::PACKETBYTEBUF_TYPE | Self::REGISTRYBYTEBUF_TYPE => {
                match &*name_and_type.name {
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
                }
            }
            "net/minecraft/network/packet/c2s/handshake/ConnectionIntent"
            | "net/minecraft/util/math/Direction"
            | "net/minecraft/world/Difficulty" => {
                if name_and_type.name != "byId" {
                    trace!("    {class_name} :: {}", name_and_type.name);
                }
                None
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
            | "java/lang/Object"
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
