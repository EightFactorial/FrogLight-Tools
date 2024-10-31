use std::borrow::Cow;

use cafebabe::{
    bytecode::Opcode,
    constant_pool::{BootstrapArgument, InvokeDynamic, MemberRef, MethodHandle},
    descriptor::{FieldType, MethodDescriptor, Ty},
    ClassFile, MethodInfo,
};
use tracing::{trace, warn};

use super::Packets;
use crate::{
    bundle::ExtractBundle,
    sources::{
        helpers::{get_bootstrap, get_class_method, get_code_silent},
        traits::MethodDescriptorTrait,
    },
};

/// Parse a method handle to extract the fields it reads.
pub(super) fn parse_method_handle(
    handle: &MethodHandle<'_>,
    data: &ExtractBundle,
) -> anyhow::Result<Vec<String>> {
    // Skip non-Minecraft classes
    if !handle.class_name.starts_with("net/minecraft") {
        return Ok(Vec::new());
    }

    let classfile = data.jar_container.get_class_err(&handle.class_name)?;
    let method =
        get_class_method(&classfile, &handle.member_ref.name, Some(&handle.member_ref.descriptor))?;

    parse_method(&classfile, method, data)
}

/// Parse a method to extract the fields it reads.
pub(super) fn parse_method(
    classfile: &ClassFile<'_>,
    method: &MethodInfo<'_>,
    data: &ExtractBundle,
) -> anyhow::Result<Vec<String>> {
    let mut fields = Vec::new();

    // Silently return if the method has no code
    let Some(code) = get_code_silent(&method.attributes) else {
        return Ok(fields);
    };

    // Iterate over the opcodes to find method calls
    for (_, op) in &code.opcodes {
        match op {
            // Find the types read from the method call, if any
            Opcode::Invokeinterface(member, ..)
            | Opcode::Invokespecial(member)
            | Opcode::Invokestatic(member)
            | Opcode::Invokevirtual(member) => {
                fields.extend(parse_invoke_member(member, data)?);
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

/// Parse an invoked member/method.
fn parse_invoke_member(
    member: &MemberRef<'_>,
    data: &ExtractBundle,
) -> anyhow::Result<Vec<String>> {
    let mut fields = Vec::new();

    // If a `*ByteBuf` method is called, return the type it reads
    if matches!(
        member.class_name.as_ref(),
        Packets::BYTEBUF_TYPE | Packets::PACKETBYTEBUF_TYPE | Packets::REGISTRYBYTEBUF_TYPE
    ) {
        if let Some(field) = match member.name_and_type.name.as_ref() {
            // Recursively parse the method
            "decode" => {
                let member_classfile = data.jar_container.get_class_err(&member.class_name)?;
                let member_method = get_class_method(
                    &member_classfile,
                    &member.name_and_type.name,
                    Some(&member.name_and_type.descriptor),
                )?;

                fields.extend(parse_method(&member_classfile, member_method, data)?);
                None
            }
            "decodeAsJson" => Some(String::from("Json")),
            // Multiple methods, type depends on method signature
            "readBitSet" => match member.name_and_type.descriptor.as_ref() {
                "(I)Ljava/util/BitSet;" => Some(String::from("FixedBitSet")),
                "()Ljava/util/BitSet;" => Some(String::from("BitSet")),
                unk => {
                    warn!(
                        "   Unknown: \"{}.{} ({unk})\"",
                        member.class_name, member.name_and_type.name
                    );
                    None
                }
            },
            "readBlockPos" => Some(String::from("BlockPos")),
            "readBlockHitResult" => Some(String::from("BlockHitResult")),
            "readBoolean" => Some(String::from("bool")),
            "readByte" | "readUnsignedByte" => Some(String::from("u8")),
            "readBytes" => Some(String::from("[u8]")),
            "readByteArray" => Some(String::from("Vec<u8>")),
            "readChunkPos" => Some(String::from("ChunkPos")),
            "readCollection" | "readList" => Some(String::from("Vec")),
            "readDouble" => Some(String::from("f64")),
            "readEnumConstant" => Some(String::from("Enum")),
            "readEnumSet" => Some(String::from("BitSet")),
            "readFloat" => Some(String::from("f32")),
            "readIdentifier" | "readRegistryKey" => Some(String::from("ResourceLocation")),
            "readInstant" | "readLong" => Some(String::from("i64")),
            "readInt" => Some(String::from("i32")),
            "readIntArray" | "readIntList" => Some(String::from("Vec<i32>")),
            "readLongArray" => Some(String::from("Vec<i64>")),
            "readMap" => Some(String::from("HashMap")),
            "readNbt" => Some(String::from("Nbt")),
            // Takes a codec as input, could parse for more fields?
            "readNullable" | "readOptional" => Some(String::from("Option")),
            "readPublicKey" => Some(String::from("PublicKey")),
            "readShort" => Some(String::from("i16")),
            "readString" => Some(String::from("String")),
            "readSyncId" | "readVarInt" => Some(String::from("VarInt")),
            "readUnsignedShort" => Some(String::from("u16")),
            "readUuid" => Some(String::from("Uuid")),
            "readVarLong" => Some(String::from("VarLong")),
            // Ignore these methods
            "getMaxValidator" | "readableBytes" | "skipBytes" => None,
            // Warn about unknown methods
            unk => {
                warn!("   Unknown: \"{}.{unk}\"", member.class_name);
                None
            }
        } {
            trace!("   Field: {field}");
            fields.push(field);
        }
        return Ok(fields);
    }

    // Skip non-Minecraft classes
    if !member.class_name.starts_with("net/minecraft") {
        return Ok(fields);
    }

    // If the method takes a `PacketByteBuf`, parse any fields it reads
    if let Some(descriptor) = MethodDescriptor::parse(&member.name_and_type.descriptor) {
        if descriptor.parameters.iter().any(|p| {
            p == BYTEBUF_FIELD_DESCRIPTOR
                || p == PACKETBYTEBUF_FIELD_DESCRIPTOR
                || p == REGISTRYBYTEBUF_FIELD_DESCRIPTOR
        }) {
            trace!(" Reading: {}.{}", member.class_name, member.name_and_type.name);

            let member_classfile = data.jar_container.get_class_err(&member.class_name)?;
            let member_method = get_class_method(
                &member_classfile,
                &member.name_and_type.name,
                Some(&member.name_and_type.descriptor),
            )?;

            fields.extend(parse_method(&member_classfile, member_method, data)?);
            trace!(" Done: {}.{}", member.class_name, member.name_and_type.name);
        }
    }

    Ok(fields)
}

const BYTEBUF_FIELD_DESCRIPTOR: &FieldType<'static> =
    &FieldType::Ty(Ty::Object(Cow::Borrowed(Packets::BYTEBUF_TYPE)));
const PACKETBYTEBUF_FIELD_DESCRIPTOR: &FieldType<'static> =
    &FieldType::Ty(Ty::Object(Cow::Borrowed(Packets::PACKETBYTEBUF_TYPE)));
const REGISTRYBYTEBUF_FIELD_DESCRIPTOR: &FieldType<'static> =
    &FieldType::Ty(Ty::Object(Cow::Borrowed(Packets::REGISTRYBYTEBUF_TYPE)));
