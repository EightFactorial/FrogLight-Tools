use std::{collections::HashMap, sync::Arc};

use cafebabe::{
    ClassFile,
    bytecode::Opcode,
    constant_pool::{InvokeDynamic, MemberRef},
};
use derive_more::Deref;
use froglight_dependency::{
    container::{Dependency, DependencyContainer},
    dependency::minecraft::{MinecraftCode, minecraft_code::CodeBundle},
    version::Version,
};
use indexmap::IndexMap;

use super::Packets;
use crate::{ToolConfig, class_helper::ClassHelper, module::packet::classes::NetworkState};

#[derive(Clone, PartialEq, Dependency)]
#[dep(retrieve = VersionCodecs::generate)]
pub(crate) struct VersionCodecs(Arc<HashMap<Version, NetworkCodecs>>);

#[derive(Clone, PartialEq, Deref)]
pub(crate) struct NetworkCodecs(IndexMap<String, NetworkPackets>);

impl VersionCodecs {
    /// Iterate over all versions and add all version's codecs to the set.
    async fn generate(deps: &mut DependencyContainer) -> anyhow::Result<Self> {
        let mut codecs = HashMap::new();

        for version in deps.get::<ToolConfig>().unwrap().versions.clone() {
            let network = Packets::extract_packet_codecs(&version, deps).await?;
            codecs.insert(version, NetworkCodecs(network));
        }

        Ok(Self(Arc::new(codecs)))
    }

    /// Get the [`NetworkCodecs`] for the given version.
    ///
    /// Returns `None` if no codecs are available for the version.
    #[inline]
    #[must_use]
    pub(crate) fn version(&self, version: &Version) -> Option<&NetworkCodecs> {
        self.0.get(version)
    }
}
// -------------------------------------------------------------------------------------------------

impl Packets {
    pub(super) async fn extract_packet_codecs(
        version: &Version,
        deps: &mut DependencyContainer,
    ) -> anyhow::Result<IndexMap<String, NetworkPackets>> {
        let classes = Self::extract_packet_classes(version, deps).await?;
        let mut codecs = IndexMap::with_capacity(classes.len());

        for (name, state) in classes {
            codecs.insert(name, Self::parse_state_classes(state, version, deps).await?);
        }

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
                            Self::parse_class(class, packet.codec.as_deref(), bundle),
                        );
                    } else {
                        panic!("PacketCodecBuilder: Missing class \"{}\"", packet.class);
                    }
                }

                for (ident, packet) in state.s2c {
                    if let Some(class) = bundle.get(&packet.class) {
                        packets.s2c.insert(
                            ident,
                            Self::parse_class(class, packet.codec.as_deref(), bundle),
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

    fn parse_class(class: ClassFile<'_>, codec: Option<&str>, classes: &CodeBundle) -> PacketInfo {
        let mut fields = IndexMap::new();

        if let Some(codec) = codec {
            let mut codec_type = CodecType::None;
            let mut direction = CodecDirection::None;

            let mut last_fields: IndexMap<String, (String, String)> = IndexMap::new();

            if let Some(initial) = class.class_optional_method_code("<clinit>") {
                let initial = initial.bytecode.as_ref().unwrap();
                let initial = initial.opcodes.iter().map(|(_, opcode)| opcode).collect::<Vec<_>>();
                class.iter_code_recursive(&initial, classes, |op| {
                    Self::handle_opcode(
                        &class,
                        op,
                        codec,
                        &mut codec_type,
                        &mut direction,
                        &mut fields,
                        &mut last_fields,
                        classes,
                    )
                });

                if codec_type != CodecType::Done {
                    panic!(
                        "PacketCodecBuilder: No Codec found for class \"{}\": {codec_type:?}, {direction:?}",
                        class.this_class
                    );
                }
            }
        }

        PacketInfo { class: class.this_class.to_string(), fields }
    }

    fn parse_class_method(class_name: &str, method_name: &str, classes: &CodeBundle) -> PacketInfo {
        let mut fields = IndexMap::new();

        if let Some(class) = classes.get(class_name) {
            if let Some(code) = class.class_optional_method_code(method_name) {
                let initial = code.bytecode.as_ref().unwrap();
                let initial = initial.opcodes.iter().map(|(_, opcode)| opcode).collect::<Vec<_>>();
                class.iter_code_recursive(&initial, classes, |op| {
                    Self::handle_opcode(
                        &class,
                        op,
                        method_name,
                        &mut CodecType::None,
                        &mut CodecDirection::None,
                        &mut fields,
                        &mut IndexMap::new(),
                        classes,
                    )
                });
            }
        }

        PacketInfo { class: class_name.to_string(), fields }
    }

    fn handle_opcode(
        class: &ClassFile<'_>,
        op: &Opcode<'_>,
        codec: &str,
        codec_type: &mut CodecType,
        direction: &mut CodecDirection,
        fields: &mut IndexMap<String, PacketField>,
        last_fields: &mut IndexMap<String, (String, String)>,
        classes: &CodeBundle,
    ) {
        const PACKET_BYTE_BUF_TYPE: &str = "net/minecraft/network/PacketByteBuf";
        const REGISTRY_BYTE_BUF_TYPE: &str = "net/minecraft/network/RegistryByteBuf";

        const PACKET_CODEC_TYPE: &str = "net/minecraft/network/codec/PacketCodec";
        const PACKET_CODEC_DESCRIPTOR: &str = "Lnet/minecraft/network/codec/PacketCodec;";
        const PACKET_CODEC_RESULT_FUNCTION_DESCRIPTOR: &str =
            "()Lnet/minecraft/network/codec/PacketCodec$ResultFunction;";

        const CODEC_DESCRIPTOR_PREFIX: &str = "()Lnet/minecraft/network/codec/";
        const DECODER_DESCRIPTOR: &str = "()Lnet/minecraft/network/codec/PacketDecoder;";
        const ENCODER_DESCRIPTOR: &str = "()Lnet/minecraft/network/codec/PacketEncoder;";

        const VALUE_FIRST_ENCODER_DESCRIPTOR: &str =
            "()Lnet/minecraft/network/codec/ValueFirstEncoder;";

        // println!("{}: {op:?}", class.this_class);
        match op {
            Opcode::Getfield(MemberRef { name_and_type, .. })
            | Opcode::Putfield(MemberRef { name_and_type, .. })
                if *direction != CodecDirection::None =>
            {
                let field_type =
                    name_and_type.descriptor.trim_start_matches('L').trim_end_matches(';');
                last_fields.insert(
                    class.this_class.to_string(),
                    (name_and_type.name.to_string(), field_type.to_string()),
                );
            }
            Opcode::Invokestatic(MemberRef { class_name, name_and_type })
                if class_name == PACKET_CODEC_TYPE && name_and_type.name == "unit" =>
            {
                *codec_type = CodecType::Unit;
            }
            Opcode::Invokedynamic(InvokeDynamic { name_and_type, .. })
                if name_and_type.descriptor.starts_with(CODEC_DESCRIPTOR_PREFIX) =>
            {
                match name_and_type.descriptor.as_ref() {
                    VALUE_FIRST_ENCODER_DESCRIPTOR => {
                        *codec_type = CodecType::ValueFirst;
                        match name_and_type.name.as_ref() {
                            "encode" => *direction = CodecDirection::Encode,
                            "decode" => *direction = CodecDirection::Decode,
                            unk => {
                                panic!("PacketCodec: Unknown ValueFirst method \"{unk}\"")
                            }
                        }
                    }
                    ENCODER_DESCRIPTOR if name_and_type.name == "encode" => {
                        last_fields.clear();
                        *direction = CodecDirection::Encode;
                    }
                    DECODER_DESCRIPTOR if name_and_type.name == "decode" => {
                        last_fields.clear();
                        *direction = CodecDirection::Decode;
                    }
                    PACKET_CODEC_RESULT_FUNCTION_DESCRIPTOR if name_and_type.name == "apply" => {}
                    unk => panic!("PacketCodec: Unknown encoder type \"{unk}\""),
                }
            }
            Opcode::Invokevirtual(MemberRef { class_name, name_and_type })
                if (class_name == PACKET_BYTE_BUF_TYPE || class_name == REGISTRY_BYTE_BUF_TYPE)
                    && *direction != CodecDirection::Decode =>
            {
                let (field_name, field_type) = last_fields
                    .shift_remove(&*class.this_class)
                    .unwrap_or_else(|| (String::from("unknown"), String::from("unknown")));

                match name_and_type.name.as_ref() {
                    "writeBoolean" => {
                        fields.insert(field_name, PacketField::Boolean);
                    }
                    "writeUnsignedByte" | "writeByte" => {
                        fields.insert(field_name, PacketField::Byte);
                    }
                    "writeBytes" | "writeByteArray" => {
                        fields.insert(field_name, PacketField::Vec(Box::new(PacketField::Byte)));
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
                    "writeIntArray" | "writeIntList" => {
                        fields.insert(field_name, PacketField::Vec(Box::new(PacketField::Int)));
                    }
                    "writeSyncId" | "writeVarInt" => {
                        fields.insert(field_name, PacketField::VarInt);
                    }
                    "writeInstant" | "writeLong" => {
                        fields.insert(field_name, PacketField::Long);
                    }
                    "writeLongArray" => {
                        fields.insert(field_name, PacketField::Vec(Box::new(PacketField::Long)));
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
                    "writeEnumSet" => {
                        fields.insert(
                            field_name,
                            PacketField::Vec(Box::new(PacketField::Enum(field_type))),
                        );
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
                        fields
                            .insert(field_name, PacketField::Option(Box::new(PacketField::String)));
                    }
                    ty @ ("writeBlockPos"
                    | "writeBlockHitResult"
                    | "writeBitSet"
                    | "writeChunkPos"
                    | "writeUuid") => {
                        fields.insert(
                            field_name,
                            PacketField::Other(ty.trim_start_matches("write").to_string()),
                        );
                    }
                    "encode"
                        if name_and_type.descriptor
                            == "(Ljava/util/function/ToIntFunction;Ljava/lang/Object;)Lnet/minecraft/network/PacketByteBuf;" =>
                        {}
                    unk => {
                        panic!("PacketCodec: Unknown PacketByteBuf encode method \"{unk}\"")
                    }
                }
            }
            Opcode::Invokevirtual(MemberRef { class_name, name_and_type })
                if (class_name == PACKET_BYTE_BUF_TYPE || class_name == REGISTRY_BYTE_BUF_TYPE)
                    && *direction != CodecDirection::Encode =>
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
                    _ty @ ("readBlockPos" | "readBlockHitResult" | "readBitSet"
                    | "readChunkPos" | "readUuid") => {}
                    unk => {
                        panic!("PacketCodec: Unknown PacketByteBuf decode method \"{unk}\"")
                    }
                }
            }
            Opcode::Invokevirtual(MemberRef { class_name, name_and_type })
                if name_and_type.descriptor.contains(PACKET_BYTE_BUF_TYPE)
                    && *direction == CodecDirection::Encode =>
            {
                let field = Self::parse_class_method(class_name, &name_and_type.name, classes);

                if let Some((name, field_type)) = last_fields.shift_remove(&*class.this_class)
                    && name_and_type.descriptor.contains(&field_type)
                {
                    fields.insert(name, PacketField::Struct(field));
                } else {
                    fields.insert(String::from("unknown"), PacketField::Struct(field));
                }
            }
            Opcode::Getstatic(MemberRef { class_name, name_and_type })
            | Opcode::Invokestatic(MemberRef { class_name, name_and_type })
                if name_and_type.descriptor == PACKET_CODEC_DESCRIPTOR
                    && name_and_type.name != codec
                    && *direction == CodecDirection::Encode =>
            {
                // println!("{}: Parsing \"{}.{}\"", class.this_class, class_name,
                // name_and_type.name);
                if let Some(class) = classes.get(&*class_name) {
                    fields.extend(
                        Self::parse_class(class, Some(&name_and_type.name), classes).fields,
                    );
                }
            }
            Opcode::Putstatic(MemberRef { class_name, name_and_type })
                if *class_name == *class.this_class && name_and_type.name == codec =>
            {
                *codec_type = CodecType::Done;
            }
            Opcode::Putstatic(MemberRef { class_name, name_and_type })
                if *class_name == *class.this_class
                    && name_and_type.descriptor == PACKET_CODEC_DESCRIPTOR
                    && *codec_type != CodecType::Done =>
            {
                *codec_type = CodecType::None;
                last_fields.clear();
                fields.clear();
            }
            _ => {}
        }
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

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct NetworkPackets {
    pub(crate) c2s: IndexMap<String, PacketInfo>,
    pub(crate) s2c: IndexMap<String, PacketInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PacketInfo {
    pub(crate) class: String,
    pub(crate) fields: IndexMap<String, PacketField>,
}

impl PacketInfo {
    /// Create an iterator over all fields in the packet.
    pub(crate) fn fields(&self) -> impl Iterator<Item = (&String, &PacketField)> {
        self.fields.iter()
    }

    /// Create an iterator over all fields in the packet, recursively.
    pub(crate) fn fields_recursive(&self) -> impl Iterator<Item = (&String, &PacketField)> {
        self.fields.iter().flat_map(|(n, f)| -> Box<dyn Iterator<Item = (&String, &PacketField)>> {
            match f {
                // Recurse into structs to get their fields
                PacketField::Struct(info) => Box::new(info.fields_recursive()),
                // Get the inner fields of `Option`s
                PacketField::Option(inner) => Box::new(std::iter::once((n, inner.as_ref()))),
                // Get the `Vec` itself and its inner type
                f @ PacketField::Vec(inner) => Box::new([(n, f), (n, inner.as_ref())].into_iter()),
                // Use all other fields as-is
                other => Box::new(std::iter::once((n, other))),
            }
        })
    }
}

// -------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PacketField {
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

impl PacketField {
    /// Print the field as a tuple of `(imports, attributes, type)`.
    #[must_use]
    pub(crate) fn print_field(&self) -> (String, String, String) {
        match self {
            PacketField::Boolean => (String::new(), String::new(), String::from("bool")),
            PacketField::Byte => (String::new(), String::new(), String::from("i8")),
            PacketField::Short => (String::new(), String::new(), String::from("u16")),
            PacketField::VarShort => (String::new(), String::from("var"), String::from("u16")),
            PacketField::Int => (String::new(), String::new(), String::from("u32")),
            PacketField::VarInt => (String::new(), String::from("var"), String::from("u32")),
            PacketField::Long => (String::new(), String::new(), String::from("u64")),
            PacketField::VarLong => (String::new(), String::from("var"), String::from("u64")),
            PacketField::Float => (String::new(), String::new(), String::from("f32")),
            PacketField::Double => (String::new(), String::new(), String::from("f64")),
            PacketField::String => (String::new(), String::new(), String::from("String")),
            PacketField::Identifier => (
                String::from("froglight_common::prelude::Identifier"),
                String::new(),
                String::from("Identifier"),
            ),
            PacketField::Nbt => (
                String::from("froglight_nbt::prelude::NbtTag"),
                String::new(),
                String::from("NbtTag"),
            ),
            PacketField::Struct(..) => (String::new(), String::new(), String::from("()")),
            PacketField::Enum(..) => (String::new(), String::new(), String::from("()")),
            PacketField::Map() => (
                String::from("bevy_platform::collections::HashMap"),
                String::new(),
                String::from("HashMap<(), ()>"),
            ),
            PacketField::Option(field) => {
                let (import, attr, field) = field.print_field();
                (import, attr, format!("Option<{field}>"))
            }
            PacketField::Vec(field) => {
                let (import, attr, field) = field.print_field();
                (import, attr, format!("Vec<{field}>"))
            }
            PacketField::Other(..) => (String::new(), String::new(), String::from("()")),
        }
    }
}
