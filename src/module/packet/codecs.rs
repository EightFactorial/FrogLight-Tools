use std::collections::HashMap;

use cafebabe::{
    bytecode::Opcode,
    constant_pool::{InvokeDynamic, LiteralConstant, Loadable, MemberRef},
    ClassFile,
};
use froglight_dependency::{
    container::DependencyContainer,
    dependency::minecraft::{minecraft_code::CodeBundle, MinecraftCode},
    version::Version,
};
use indexmap::IndexMap;

use super::Packets;
use crate::{class_helper::ClassHelper, module::packet::classes::NetworkState};

impl Packets {
    pub(super) async fn extract_packet_codecs(
        version: &Version,
        deps: &mut DependencyContainer,
    ) -> anyhow::Result<HashMap<String, NetworkPackets>> {
        let classes = Self::extract_packet_classes(version, deps).await?;
        let mut codecs = HashMap::with_capacity(classes.len());

        for (name, state) in classes {
            codecs.insert(name, Self::parse_state_classes(state, version, deps).await?);
        }

        // println!("{codecs:#?}");

        Ok(codecs)
    }

    async fn parse_state_classes(
        state: NetworkState,
        version: &Version,
        deps: &mut DependencyContainer,
    ) -> anyhow::Result<NetworkPackets> {
        let mut packets = NetworkPackets::default();

        deps.get_or_retrieve::<MinecraftCode>().await?;
        deps.scoped_fut::<MinecraftCode, anyhow::Result<()>>(
            async |code: &mut MinecraftCode, deps| {
                let bundle = code.get_version(version, deps).await?;

                for (ident, packet) in state.c2s {
                    if let Some(class) = bundle.get(&packet.class) {
                        packets.c2s.insert(
                            ident,
                            Self::parse_class(class, packet.codec.as_deref(), bundle).await,
                        );
                    } else {
                        panic!("PacketCodecBuilder: Missing class \"{}\"", packet.class);
                    }
                }

                for (ident, packet) in state.s2c {
                    if let Some(class) = bundle.get(&packet.class) {
                        packets.s2c.insert(
                            ident,
                            Self::parse_class(class, packet.codec.as_deref(), bundle).await,
                        );
                    } else {
                        panic!("PacketCodecBuilder: Missing class \"{}\"", packet.class);
                    }
                }

                Ok(())
            },
        )
        .await?;

        Ok(packets)
    }

    async fn parse_class(
        class: ClassFile<'_>,
        codec: Option<&str>,
        classes: &CodeBundle,
    ) -> PacketInfo {
        const PACKET_BYTE_BUF_TYPE: &str = "net/minecraft/network/PacketByteBuf";
        const PACKET_DECODER_TYPE: &str = "net/minecraft/network/codec/PacketDecoder";

        const PACKET_CODEC_TYPE: &str = "net/minecraft/network/codec/PacketCodec";
        const PACKET_CODEC_DESCRIPTOR: &str = "Lnet/minecraft/network/codec/PacketCodec;";
        const PACKET_CODEC_RESULT_FUNCTION_DESCRIPTOR: &str =
            "()Lnet/minecraft/network/codec/PacketCodec$ResultFunction;";

        const CODEC_DESCRIPTOR_PREFIX: &str = "()Lnet/minecraft/network/codec/";
        const DECODER_DESCRIPTOR: &str = "()Lnet/minecraft/network/codec/PacketDecoder;";
        const ENCODER_DESCRIPTOR: &str = "()Lnet/minecraft/network/codec/PacketEncoder;";

        const VALUE_FIRST_ENCODER_DESCRIPTOR: &str =
            "()Lnet/minecraft/network/codec/ValueFirstEncoder;";
        const UNIT_ENCODER_DESCRIPTOR: &str =
            "(Ljava/lang/Object;)Lnet/minecraft/network/codec/PacketCodec;";

        let mut fields = IndexMap::new();

        if let Some(codec) = codec {
            let mut codec_type = CodecType::None;
            let mut direction = CodecDirection::None;

            let mut last_fields: HashMap<String, (String, String)> = HashMap::new();

            let initial = class.class_code().bytecode.as_ref().unwrap();
            let initial = initial.opcodes.iter().map(|(_, opcode)| opcode).collect::<Vec<_>>();
            class.iter_code_recursive(&initial, classes, |op| {
                // println!("{op:?}");
                match op {
                    (Opcode::Getfield(MemberRef { class_name, name_and_type })
                        | Opcode::Putfield(MemberRef { class_name, name_and_type }))
                        if direction != CodecDirection::None
                    => {
                        let field_type = name_and_type.descriptor.trim_start_matches('L').trim_end_matches(';');
                        last_fields.insert(class_name.to_string(), (name_and_type.name.to_string(), field_type.to_string()));
                    }
                    Opcode::Invokestatic(MemberRef { class_name, name_and_type })
                        if class_name == PACKET_CODEC_TYPE && name_and_type.name == "unit" =>
                    {
                        codec_type = CodecType::Unit;
                    }
                    Opcode::Invokedynamic(InvokeDynamic { name_and_type, .. })
                        if name_and_type.descriptor.starts_with(CODEC_DESCRIPTOR_PREFIX) =>
                    {
                        match name_and_type.descriptor.as_ref() {
                            VALUE_FIRST_ENCODER_DESCRIPTOR => {
                                codec_type = CodecType::ValueFirst;
                                match name_and_type.name.as_ref() {
                                    "encode" => direction = CodecDirection::Encode,
                                    "decode" => direction = CodecDirection::Decode,
                                    unk => {
                                        panic!("PacketCodec: Unknown ValueFirst method \"{unk}\"")
                                    }
                                }
                            }
                            ENCODER_DESCRIPTOR if name_and_type.name == "encode" => {
                                direction = CodecDirection::Encode;
                            }
                            DECODER_DESCRIPTOR if name_and_type.name == "decode" => {
                                direction = CodecDirection::Decode;
                            }
                            PACKET_CODEC_RESULT_FUNCTION_DESCRIPTOR
                                if name_and_type.name == "apply" => {}
                            unk => panic!("PacketCodec: Unknown encoder type \"{unk}\""),
                        }
                    }
                    Opcode::Invokevirtual(MemberRef { class_name, name_and_type })
                        if class_name == PACKET_BYTE_BUF_TYPE
                            && direction == CodecDirection::Encode =>
                    {
                        let (field_name, field_type) = last_fields
                            .remove(&*class.this_class)
                            .unwrap_or_else(|| (String::from("unknown"), String::from("unknown")));

                        match name_and_type.name.as_ref() {
                            "writeBoolean" => {
                                fields.insert(field_name, PacketField::Boolean);
                            }
                            "writeUnsignedByte" | "writeByte" => {
                                fields.insert(field_name, PacketField::Byte);
                            }
                            "writeBytes" | "writeByteArray" => {
                                fields.insert(
                                    field_name,
                                    PacketField::Vec(Box::new(PacketField::Byte)),
                                );
                            }
                            "writeShort" => {
                                fields.insert(field_name, PacketField::Short);
                            }
                            "writeVarShort" => {
                                fields.insert(field_name, PacketField::VarShort);
                            }
                            "writeInt" => {
                                fields.insert(field_name, PacketField::Int);
                            }
                            "writeIntArray" | "writeIntList"  => {
                                fields.insert(
                                    field_name,
                                    PacketField::Vec(Box::new(PacketField::Int)),
                                );
                            }
                            "writeSyncId" | "writeVarInt" => {
                                fields.insert(field_name, PacketField::VarInt);
                            }
                            "writeInstant" | "writeLong" => {
                                fields.insert(field_name, PacketField::Long);
                            }
                            "writeLongArray" => {
                                fields.insert(
                                    field_name,
                                    PacketField::Vec(Box::new(PacketField::Long)),
                                );
                            }
                            "writeVarLong" => {
                                fields.insert(field_name, PacketField::VarLong);
                            }
                            "writeFloat" => {
                                fields.insert(field_name, PacketField::Float);
                            }
                            "writeDouble" => {
                                fields.insert(field_name, PacketField::Double);
                            }
                            "encodeAsJson" | "writeString" => {
                                fields.insert(field_name, PacketField::String);
                            }
                            "writeRegistryKey" | "writeIdentifier" => {
                                fields.insert(field_name, PacketField::Identifier);
                            }
                            "writeNbt" => {
                                fields.insert(field_name, PacketField::Nbt);
                            }
                            "writeEnumConstant" => {
                                fields.insert(field_name, PacketField::Enum(field_type));
                            }
                            "writeCollection" | "writeList" => {
                                if let Some((_, last)) = fields.last_mut() {
                                    let stolen = core::mem::replace(last, PacketField::Boolean);
                                    *last = PacketField::Vec(Box::new(stolen));
                                }
                            }
                            "writeMap" => {
                                fields.insert(
                                    field_name,
                                    PacketField::Map(), // TODO: Specify key and value types
                                );
                            }
                            "writeOptional" | "writeNullable" => {
                                fields.insert(
                                    field_name,
                                    PacketField::Option(Box::new(PacketField::String)),
                                );
                            }
                            ty @ ("writeBlockPos"
                            | "writeBlockHitResult"
                            | "writeChunkPos"
                            | "writeUuid") => {
                                fields.insert(
                                    field_name,
                                    PacketField::Other(ty.trim_start_matches("write").to_string()),
                                );
                            }
                            "encode" if name_and_type.descriptor == "(Ljava/util/function/ToIntFunction;Ljava/lang/Object;)Lnet/minecraft/network/PacketByteBuf;" => {}
                            unk => {
                                panic!("PacketCodec: Unknown PacketByteBuf encode method \"{unk}\"")
                            }
                        }
                    }
                    Opcode::Invokevirtual(MemberRef { class_name, name_and_type })
                        if class_name == PACKET_BYTE_BUF_TYPE
                            && direction == CodecDirection::Decode =>
                    {
                        match name_and_type.name.as_ref() {
                            "readBoolean" => {}
                            "readUnsignedByte" | "readByte" => {}
                            "readBytes" | "readByteArray" => {}
                            "readShort" => {}
                            "readVarShort" => {}
                            "readInt" => {}
                            "readIntArray" | "readIntList" => {}
                            "readSyncId" | "readVarInt" => {}
                            "readInstant" | "readLong" => {}
                            "readLongArray" => {}
                            "readVarLong" => {}
                            "readFloat" => {}
                            "readDouble" => {}
                            "decodeAsJson" | "readString" => {}
                            "readRegistryKey" | "readIdentifier" => {}
                            "readNbt" => {}
                            "readEnumConstant" => {}
                            "readCollection" | "readList" => {}
                            "readOptional" | "readNullable" => {}
                            ty @ ("readBlockPos" | "readBlockHitResult" | "readChunkPos"
                            | "readUuid") => {}
                            unk => {
                                panic!("PacketCodec: Unknown PacketByteBuf decode method \"{unk}\"")
                            }
                        }
                    }
                    Opcode::Invokevirtual(MemberRef { class_name, name_and_type })
                        if name_and_type.descriptor.contains(PACKET_BYTE_BUF_TYPE) && direction == CodecDirection::Decode =>
                    {
                        let field =
                            Self::parse_class_method(class_name, &name_and_type.name, classes);

                        if let Some((name, field_type)) = last_fields.remove(&*class.this_class)
                            && name_and_type.descriptor.contains(&field_type)
                        {
                            fields.insert(name, PacketField::Struct(field));
                        } else {
                            fields.insert(String::from("unknown"), PacketField::Struct(field));
                        }
                    }
                    Opcode::Putstatic(MemberRef { class_name, name_and_type })
                        if *class_name == *class.this_class && name_and_type.name == codec =>
                    {
                        codec_type = CodecType::Done;
                    }
                    Opcode::Putstatic(MemberRef { class_name, name_and_type })
                        if *class_name == *class.this_class
                            && name_and_type.descriptor == PACKET_CODEC_DESCRIPTOR
                            && codec_type != CodecType::Done =>
                    {
                        codec_type = CodecType::None;
                        last_fields.clear();
                        fields.clear();
                    }
                    _ => {}
                }
            });

            if codec_type != CodecType::Done {
                panic!(
                    "PacketCodecBuilder: No Codec found for class \"{}\": {codec_type:?}, {direction:?}",
                    class.this_class
                );
            }
        }

        PacketInfo { class: class.this_class.to_string(), fields }
    }

    fn parse_class_method(class: &str, method: &str, classes: &CodeBundle) -> PacketInfo {
        let mut fields = IndexMap::new();

        PacketInfo { class: class.to_string(), fields }
    }
}

// -------------------------------------------------------------------------------------------------

#[derive(Debug, Default, PartialEq, Eq)]
enum CodecType {
    #[default]
    None,
    Done,
    Unit,
    ValueFirst,
}

#[derive(Debug, Default, PartialEq, Eq)]
enum CodecDirection {
    #[default]
    None,
    Encode,
    Decode,
}

// -------------------------------------------------------------------------------------------------

#[derive(Debug, Default, PartialEq)]
pub(super) struct NetworkPackets {
    pub(super) c2s: IndexMap<String, PacketInfo>,
    pub(super) s2c: IndexMap<String, PacketInfo>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct PacketInfo {
    pub(super) class: String,
    pub(super) fields: IndexMap<String, PacketField>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum PacketField {
    Boolean,
    Byte,
    Short,
    VarShort,
    Int,
    VarInt,
    Long,
    VarLong,
    Float,
    Double,
    String,
    Identifier,
    Nbt,
    Struct(PacketInfo),
    Enum(String),
    Map(),
    Option(Box<PacketField>),
    Vec(Box<PacketField>),
    Other(String),
}
