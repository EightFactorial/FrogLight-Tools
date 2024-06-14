use anyhow::bail;
use cafebabe::{
    bytecode::Opcode,
    constant_pool::{BootstrapArgument, InvokeDynamic, MemberRef, MethodHandle, NameAndType},
    ClassFile,
};
use tracing::{error, trace};

use super::Packets;
use crate::{
    bundle::ExtractBundle,
    bytecode::ClassContainer,
    sources::{
        bytecode::modules::packets::constructor::CodecConstructor, get_class_bootstrap,
        get_class_method, get_method_code,
    },
};

impl Packets {
    pub(super) fn parse_create(
        codec: CodecConstructor<'_>,
        index: usize,
        data: &ExtractBundle<'_>,
    ) -> anyhow::Result<Vec<String>> {
        let CodecConstructor::Create(class_name) = codec else { panic!("Expected create codec") };

        let Some(classfile) =
            data.jar_container.get(class_name.as_ref()).map(ClassContainer::parse)
        else {
            bail!("Packet class \"{class_name}\" not found in jar");
        };
        let Some(method) = get_class_method(&classfile, Self::CODEC_METHOD) else {
            error!(
                "Packet class \"{}\" has no \"{}\" method",
                classfile.this_class,
                Self::CODEC_METHOD
            );
            return Ok(Vec::new());
        };

        let bootstrap_methods = get_class_bootstrap(&classfile).unwrap_or_default();
        let Some(code) = get_method_code(method) else {
            error!(
                "Packet class \"{}\" method \"{}\" has no code",
                classfile.this_class,
                Self::CODEC_METHOD
            );
            return Ok(Vec::new());
        };

        let mut fields = Vec::new();
        let mut skip = false;
        for (_, op) in code.opcodes.iter().rev().skip(index) {
            match op {
                Opcode::Invokedynamic(InvokeDynamic { attr_index, .. }) => {
                    // Skip the second dynamic call, which is the write method
                    if skip {
                        continue;
                    }
                    skip = true;

                    // Iterate over the bootstrap methods to find method handles
                    if let Some(attr) = bootstrap_methods.get(*attr_index as usize) {
                        for arg in &attr.arguments {
                            // Parse the function behind the method handle
                            if let BootstrapArgument::MethodHandle(handle) = arg {
                                fields.extend(Self::parse_method_handle(handle, data)?);
                            }
                        }
                    }
                }
                Opcode::Putstatic(_) => break,
                _ => {}
            }
        }

        Ok(fields)
    }

    fn parse_method_handle(
        method: &MethodHandle<'_>,
        data: &ExtractBundle<'_>,
    ) -> anyhow::Result<Vec<String>> {
        if !method.class_name.starts_with("net/minecraft") {
            return Ok(Vec::new());
        }

        let Some(classfile) =
            data.jar_container.get(method.class_name.as_ref()).map(ClassContainer::parse)
        else {
            bail!("Packet class \"{}\" not found in jar", method.class_name);
        };

        Self::parse_method(&classfile, &method.member_ref, data)
    }

    pub(super) fn parse_method(
        classfile: &ClassFile<'_>,
        name_and_type: &NameAndType,
        data: &ExtractBundle<'_>,
    ) -> anyhow::Result<Vec<String>> {
        let Some(method) = classfile.methods.iter().find(|&m| {
            m.name == name_and_type.name && m.descriptor.to_string() == name_and_type.descriptor
        }) else {
            bail!(
                "Packet class \"{}\" has no method \"{}\" with descriptor \"{}\"",
                classfile.this_class,
                name_and_type.name,
                name_and_type.descriptor
            );
        };

        let bootstrap_methods = get_class_bootstrap(classfile).unwrap_or_default();
        let Some(code) = get_method_code(method) else {
            error!(
                "Packet class \"{}\" method \"{}\" has no code",
                classfile.this_class, method.name
            );
            return Ok(Vec::new());
        };

        let mut fields = Vec::new();
        let mut skip = false;
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
                Opcode::Invokedynamic(InvokeDynamic { attr_index, .. }) => {
                    // Skip the second dynamic call, which is the write method
                    if skip {
                        continue;
                    }
                    skip = true;

                    // Iterate over the bootstrap methods to find method handles
                    if let Some(attr) = bootstrap_methods.get(*attr_index as usize) {
                        for arg in &attr.arguments {
                            // Parse the function behind the method handle, recursively
                            if let BootstrapArgument::MethodHandle(handle) = arg {
                                fields.extend(Self::parse_method_handle(handle, data)?);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(fields)
    }

    const BYTEBUF_TYPE: &'static str = "io/netty/buffer/ByteBuf";
    const PACKETBYTEBUF_TYPE: &'static str = "net/minecraft/network/PacketByteBuf";
    const REGISTRYBYTEBUF_TYPE: &'static str = "net/minecraft/network/RegistryByteBuf";

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
}
