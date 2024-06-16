use std::borrow::Cow;

use anyhow::bail;
use cafebabe::{
    bytecode::{ByteCode, Opcode},
    constant_pool::{MemberRef, MethodHandle},
    ClassFile, FieldInfo,
};
use hashbrown::HashMap;
use tracing::trace;

mod create;
mod map;
mod method;
mod tuple;
mod unlimited;
mod xmap;

use super::Packets;
use crate::{
    bundle::ExtractBundle,
    sources::helpers::{get_class_field, get_class_method, get_code},
};

impl Packets {
    /// Parse the packets in the given class map.
    pub(super) fn parse(
        classes: HashMap<String, String>,
        data: &ExtractBundle<'_>,
    ) -> anyhow::Result<HashMap<String, (String, Vec<String>)>> {
        let mut packet_data = HashMap::with_capacity(classes.len());

        for (packet, class) in classes {
            trace!("Packet: {packet}");
            let fields = Self::parse_packet(&class, data)?;
            packet_data.insert(packet, (class, fields));
        }

        Ok(packet_data)
    }

    const PACKET_CODEC_FIELD: &'static str = "CODEC";

    /// Parse a packet class to extract its fields.
    fn parse_packet(class: &str, data: &ExtractBundle<'_>) -> anyhow::Result<Vec<String>> {
        // Skip "Bundle" packets, which have no codec
        if class.contains("Bundle") {
            return Ok(Vec::new());
        }

        let classfile = data.jar_container.get_class_err(class)?;
        let codec = get_class_field(&classfile, Self::PACKET_CODEC_FIELD)?;
        Self::parse_codec_field(&classfile, codec, data)
    }

    /// Parse the codec field of a packet class.
    fn parse_codec_field(
        classfile: &ClassFile<'_>,
        field: &FieldInfo<'_>,
        data: &ExtractBundle<'_>,
    ) -> anyhow::Result<Vec<String>> {
        match CodecType::parse_codec(classfile, field)? {
            CodecType::Create { encode: _, decode } => {
                create::parse_create(classfile, decode, data)
            }
            CodecType::Tuple { bundle, descriptor } => {
                tuple::parse_tuple(classfile, &bundle, descriptor, data)
            }
            CodecType::Xmap { parent, encode: _, decode } => {
                xmap::parse_xmap(classfile, parent, decode, data)
            }
            CodecType::Special { member } => {
                let member_classfile = data.jar_container.get_class_err(&member.class_name)?;
                method::parse_method(&member_classfile, member, data)
            }
            CodecType::Unlimited { parent } => unlimited::parse_unlimited(classfile, parent, data),
            CodecType::Map { key, value } => map::parse_map(classfile, key, value, data),
            CodecType::Unit => Ok(Vec::new()),
        }
    }
}

/// A type of codec used to encode and decode packets.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(super) enum CodecType<'a> {
    /// Codecs created using `createCodec`
    Create { encode: &'a MethodHandle<'a>, decode: &'a MethodHandle<'a> },
    /// Codecs created using `map`
    Map { key: &'a MemberRef<'a>, value: &'a MemberRef<'a> },
    /// Codecs created using an `<init>` method
    Special { member: &'a MemberRef<'a> },
    /// Codecs created using `tuple`
    Tuple { bundle: Vec<&'a Opcode<'a>>, descriptor: &'a Cow<'a, str> },
    /// Codecs created using `unit`
    Unit,
    /// Codecs created using `unlimitedCodec`
    Unlimited { parent: &'a MemberRef<'a> },
    /// Codecs created using `xmap`
    Xmap { parent: &'a MemberRef<'a>, encode: &'a MethodHandle<'a>, decode: &'a MethodHandle<'a> },
}

impl<'a> CodecType<'a> {
    const CODEC_METHOD: &'static str = "<clinit>";

    pub(super) const PACKET_CODECS_TYPE: &'static str = "net/minecraft/network/codec/PacketCodecs";
    pub(super) const PACKET_CODEC_TYPE: &'static str = "net/minecraft/network/codec/PacketCodec";
    pub(super) const PACKET_TYPE: &'static str = "net/minecraft/network/packet/Packet";

    /// Parse the [`CodecType`] from a codec field.
    fn parse_codec(classfile: &'a ClassFile<'_>, field: &FieldInfo<'_>) -> anyhow::Result<Self> {
        let method = get_class_method(classfile, Self::CODEC_METHOD, None)?;
        let code = get_code(&method.attributes)?;

        // Find the position of the `PutStatic` opcode for the codec field.
        let Some(index) = code.opcodes.iter().position(|(_, op)| {
            if let Opcode::Putstatic(MemberRef { name_and_type, .. }) = op {
                name_and_type.name == field.name
            } else {
                false
            }
        }) else {
            bail!(
                "Failed to find `PutStatic` for \"{}\" in \"{}\"",
                field.name,
                classfile.this_class
            );
        };

        Self::parse_codec_function(classfile, code, index).inspect_err(|_| {
            trace!("Failed to parse codec: {:?}", &code.opcodes[index].1);
        })
    }

    // Match on the opcode before the `PutStatic` to determine the codec type.
    fn parse_codec_function(
        classfile: &'a ClassFile<'_>,
        code: &'a ByteCode<'_>,
        opcode_index: usize,
    ) -> anyhow::Result<Self> {
        match &code.opcodes[opcode_index - 1].1 {
            Opcode::Invokeinterface(member, ..) | Opcode::Invokestatic(member) => {
                match &*member.class_name {
                    Self::PACKET_CODECS_TYPE => match &*member.name_and_type.name {
                        "map" => map::create_map(classfile, code, opcode_index),
                        "unlimitedCodec" | "unlimitedRegistryCodec" => {
                            unlimited::create_unlimited(classfile, code, opcode_index)
                        }
                        _ => Ok(CodecType::Special { member }),
                    },
                    Self::PACKET_CODEC_TYPE => match &*member.name_and_type.name {
                        "unit" => Ok(CodecType::Unit),
                        "tuple" => tuple::create_tuple(classfile, code, opcode_index),
                        "xmap" => xmap::create_xmap(classfile, code, opcode_index),
                        _ => Ok(CodecType::Special { member }),
                    },
                    Self::PACKET_TYPE => match &*member.name_and_type.name {
                        "createCodec" => create::create_create(classfile, code, opcode_index - 1),

                        _ => bail!("Unexpected method before `PutStatic`: {member:?}"),
                    },
                    "net/minecraft/network/packet/CustomPayload" => {
                        match &*member.name_and_type.name {
                            "codecOf" => create::create_create(classfile, code, opcode_index - 1),
                            _ => bail!("Unexpected method before `PutStatic`: {member:?}"),
                        }
                    }
                    _ => bail!("Unexpected class before `PutStatic`: {member:?}"),
                }
            }
            Opcode::Invokespecial(member) => Ok(CodecType::Special { member }),
            unk => {
                bail!("Unexpected opcode before `PutStatic`: {unk:?}");
            }
        }
    }
}
