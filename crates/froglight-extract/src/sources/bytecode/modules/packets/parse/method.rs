use cafebabe::{
    bytecode::Opcode,
    constant_pool::{BootstrapArgument, InvokeDynamic, MemberRef, MethodHandle, NameAndType},
    ClassFile,
};
use tracing::trace;

use crate::{
    bundle::ExtractBundle,
    sources::helpers::{get_bootstrap, get_class_method, get_code, handle_to_ref},
};

/// Parsse the types read by a method
///
/// Uses a handle for convenience
pub(super) fn parse_method_handle(
    method: &MethodHandle<'_>,
    data: &ExtractBundle<'_>,
) -> anyhow::Result<Vec<String>> {
    // Skip non-Minecraft classes
    if !method.class_name.starts_with("net/minecraft") {
        return Ok(Vec::new());
    }

    let method_classfile = data.jar_container.get_class_err(&method.class_name)?;
    let method_ref = handle_to_ref(method);
    parse_method(&method_classfile, &method_ref, data)
}

/// Parse the types read by a method
pub(super) fn parse_method(
    classfile: &ClassFile<'_>,
    method: &MemberRef<'_>,
    data: &ExtractBundle<'_>,
) -> anyhow::Result<Vec<String>> {
    let method = get_class_method(
        classfile,
        &method.name_and_type.name,
        Some(&method.name_and_type.descriptor),
    )?;
    let code = get_code(&method.attributes)?;

    // Iterator through opcodes, looking for relevant method calls
    let mut fields = Vec::new();
    for (_, op) in &code.opcodes {
        match op {
            // Find the type read from the method call, if any
            Opcode::Invokeinterface(MemberRef { class_name, name_and_type }, ..)
            | Opcode::Invokespecial(MemberRef { class_name, name_and_type })
            | Opcode::Invokestatic(MemberRef { class_name, name_and_type })
            | Opcode::Invokevirtual(MemberRef { class_name, name_and_type }) => {
                if let Some(field) = function_to_type(class_name, name_and_type) {
                    fields.push(field);
                }
            }
            // Resolve the dynamic method call and find the type, if any
            Opcode::Invokedynamic(InvokeDynamic { attr_index, .. }) => {
                // Iterate over the bootstrap methods to find method handles
                let bootstrap = get_bootstrap(&classfile.attributes)?;
                if let Some(attr) = bootstrap.get(*attr_index as usize) {
                    for arg in &attr.arguments {
                        // Parse the function behind the method handle, recursively
                        if let BootstrapArgument::MethodHandle(handle) = arg {
                            fields.extend(parse_method_handle(handle, data)?);
                        }
                    }
                }
            }
            _ => {}
        }
    }
    Ok(fields)
}

const BYTEBUF_TYPE: &str = "io/netty/buffer/ByteBuf";
const PACKETBYTEBUF_TYPE: &str = "net/minecraft/network/PacketByteBuf";
const REGISTRYBYTEBUF_TYPE: &str = "net/minecraft/network/RegistryByteBuf";

/// Return the type read by a method call
#[allow(clippy::match_same_arms)]
fn function_to_type(class_name: &str, name_and_type: &NameAndType<'_>) -> Option<String> {
    trace!("    {class_name} :: {}", name_and_type.name);
    match class_name {
        BYTEBUF_TYPE | PACKETBYTEBUF_TYPE | REGISTRYBYTEBUF_TYPE => match &*name_and_type.name {
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
            _ => None,
        },
        "net/minecraft/entity/EquipmentSlot" => {
            if name_and_type.name == "values" {
                Some(String::from("Equipment"))
            } else {
                None
            }
        }
        "net/minecraft/network/packet/s2c/login/LoginQueryRequestS2CPacket"
        | "net/minecraft/network/packet/c2s/login/LoginQueryResponseC2SPacket" => {
            if name_and_type.name == "readPayload" {
                Some(String::from("UnsizedByteBuffer"))
            } else {
                None
            }
        }
        "net/minecraft/network/codec/PacketCodecs" => {
            if name_and_type.name == "registryValue" {
                Some(String::from("VarInt"))
            } else {
                None
            }
        }
        "net/minecraft/text/Text$Serialization" => {
            if name_and_type.name == "fromLenientJson" {
                Some(String::from("Text"))
            } else {
                None
            }
        }
        "net/minecraft/network/packet/c2s/handshake/ConnectionIntent"
        | "net/minecraft/util/math/Direction"
        | "net/minecraft/world/Difficulty" => None,
        "com/google/common/collect/Lists"
        | "net/minecraft/network/codec/PacketCodec"
        | "net/minecraft/util/math/MathHelper"
        | "com/mojang/datafixers/util/Pair"
        | "java/lang/Object"
        | "java/util/function/Function"
        | "java/util/List"
        | "java/util/Optional" => None,
        _ => None,
    }
}
