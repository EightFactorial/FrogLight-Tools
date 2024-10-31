use anyhow::{anyhow, bail};
use cafebabe::{
    bytecode::Opcode,
    constant_pool::{BootstrapArgument, InvokeDynamic, MemberRef},
    ClassFile, FieldInfo,
};
use tracing::trace;

use crate::{
    bundle::ExtractBundle,
    sources::{
        helpers::{get_bootstrap, get_class_field, get_class_method, get_code},
        Packets,
    },
};

const STATIC_METHOD: &str = "<clinit>";

pub(super) fn parse_codec(
    classfile: &ClassFile<'_>,
    field: &FieldInfo<'_>,
    data: &ExtractBundle,
) -> anyhow::Result<Vec<String>> {
    trace!("  Reading: {}.{}", classfile.this_class, field.name);
    let bundle = opcode_bundle(classfile, field)?;

    let &last = bundle.last().unwrap();
    if let Opcode::Invokeinterface(member, ..)
    | Opcode::Invokespecial(member)
    | Opcode::Invokestatic(member)
    | Opcode::Invokevirtual(member) = last
    {
        let result = match member.class_name.as_ref() {
            Packets::PACKET_TYPE | Packets::PACKETCODEC_TYPE => {
                #[allow(clippy::match_same_arms)]
                match member.name_and_type.name.as_ref() {
                    // Made from "encode"/"decode" functions
                    "createCodec" | "of" | "ofStatic" => {
                        parse_encode_decode(classfile, &bundle, data)
                    }
                    // Made from a tuple of functions/codecs
                    "tuple" => parse_tuple(&bundle, data),
                    // Units have no fields
                    "unit" => Ok(Vec::new()),
                    // A "createCodec" on top of another codec
                    "xmap" => parse_xmap(classfile, &bundle, data),
                    // Ignore these methods
                    "collect" | "dispatch" => Ok(Vec::new()),
                    _ => Err(anyhow!("Unexpected `Invoke*` before Codec initialization: {last:?}")),
                }
            }
            Packets::PACKETCODECS_TYPE => match member.name_and_type.name.as_ref() {
                "indexed" | "registryEntry" => Ok(vec![String::from("VarInt")]),
                "map" => Ok(vec![String::from("HashMap")]),
                _ => Err(anyhow!("Unexpected `Invoke*` before Codec initialization: {last:?}")),
            },
            _ => parse_method(member, data),
        }?;
        trace!("  Done: {}.{}", classfile.this_class, field.name);
        Ok(result)
    } else {
        Err(anyhow!("Unexpected Opcode before Codec initialization: {last:?}"))
    }
}

/// Get a list of opcodes that construct a field.
fn opcode_bundle<'a>(
    classfile: &'a ClassFile<'_>,
    field: &'a FieldInfo<'_>,
) -> anyhow::Result<Vec<&'a Opcode<'a>>> {
    let method = get_class_method(classfile, STATIC_METHOD, None)?;
    let code = get_code(&method.attributes)?;

    if let Some(index) = code.opcodes.iter().position(|(_, op)| {
        if let Opcode::Putstatic(op_field) = op {
            op_field.name_and_type.name == field.name
        } else {
            false
        }
    }) {
        let mut bundle = Vec::new();

        let invert_index = code.opcodes.len() - index;
        for (_, op) in code.opcodes.iter().rev().skip(invert_index) {
            if let Opcode::Putstatic(_) = op {
                break;
            }
            bundle.push(op);
        }
        bundle.reverse();

        Ok(bundle)
    } else {
        Err(anyhow!("Unable to find `PutStatic` for \"{}.{}\"", classfile.this_class, field.name))
    }
}

/// Parse a method called by a codec.
fn parse_method(member: &MemberRef<'_>, data: &ExtractBundle) -> anyhow::Result<Vec<String>> {
    let classfile = data.jar_container.get_class_err(&member.class_name)?;
    let method = get_class_method(
        &classfile,
        &member.name_and_type.name,
        Some(&member.name_and_type.descriptor),
    )?;
    super::method::parse_method(&classfile, method, data)
}

/// Parse a codec created from a pair of `encode` and `decode` methods.
///
/// Only parses the decode method
fn parse_encode_decode(
    classfile: &ClassFile<'_>,
    bundle: &[&Opcode],
    data: &ExtractBundle,
) -> anyhow::Result<Vec<String>> {
    let mut iter = bundle.iter();
    iter.next_back();

    // Get the "decode" method used in the codec
    let &decode = iter.next_back().unwrap();
    if let Opcode::Invokedynamic(InvokeDynamic { attr_index, .. }) = decode {
        // Get the bootstrap method called by the `InvokeDynamic`
        let bootstrap = get_bootstrap(&classfile.attributes)?;
        if let Some(bootstrap_entry) = bootstrap.get(*attr_index as usize) {
            let mut fields = Vec::new();

            // Parse all methods called by the bootstrap method
            for arg in &bootstrap_entry.arguments {
                if let BootstrapArgument::MethodHandle(handle) = arg {
                    trace!("  Reading: {}.{}", handle.class_name, handle.member_ref.name);
                    fields.extend(super::method::parse_method_handle(handle, data)?);
                    trace!("  Done: {}.{}", handle.class_name, handle.member_ref.name);
                }
            }

            Ok(fields)
        } else {
            Err(anyhow!("No `BootstrapMethod` {attr_index} in \"{}\"", classfile.this_class))
        }
    } else {
        Err(anyhow!("Expected `InvokeDynamic` Opcode when parsing `createCodec` Codec: {decode:?}"))
    }
}

const PACKETCODEC_OBJECT: &str = "Lnet/minecraft/network/codec/PacketCodec;";

/// Parse a codec created by a `tuple` method.
fn parse_tuple(bundle: &[&Opcode], data: &ExtractBundle) -> anyhow::Result<Vec<String>> {
    let mut fields = Vec::new();

    // Parse all codecs in the tuple
    for &op in bundle {
        if let Opcode::Getstatic(member) = op {
            if member.name_and_type.descriptor == PACKETCODEC_OBJECT {
                if let Some(field) =
                    match (member.class_name.as_ref(), member.name_and_type.name.as_ref()) {
                        (Packets::PACKETCODECS_TYPE, "BOOL") => Some(String::from("bool")),
                        (Packets::PACKETCODECS_TYPE, "BYTE" | "DEGREES") => {
                            Some(String::from("u8"))
                        }
                        (Packets::PACKETCODECS_TYPE, "BYTE_ARRAY") => Some(String::from("Vec<u8>")),
                        (Packets::PACKETCODECS_TYPE, "DOUBLE") => Some(String::from("f64")),
                        (Packets::PACKETCODECS_TYPE, "FLOAT") => Some(String::from("f32")),
                        (Packets::PACKETCODECS_TYPE, "LONG") => Some(String::from("i64")),
                        // TODO: Remove "field" name when assigned a proper name
                        (Packets::PACKETCODECS_TYPE, "INTEGER" | "field_53740") => {
                            Some(String::from("i32"))
                        }
                        (Packets::PACKETCODECS_TYPE, "GAME_PROFILE") => {
                            Some(String::from("GameProfile"))
                        }
                        (Packets::PACKETCODECS_TYPE, "OPTIONAL_INT") => {
                            Some(String::from("Option<VarInt>"))
                        }
                        (Packets::PACKETCODECS_TYPE, "SHORT") => Some(String::from("i16")),
                        (Packets::PACKETCODECS_TYPE, "STRING") => Some(String::from("String")),
                        (Packets::PACKETCODECS_TYPE, "VAR_INT" | "SYNC_ID") => {
                            Some(String::from("VarInt"))
                        }
                        (Packets::PACKETCODECS_TYPE, "VAR_LONG") => Some(String::from("VarLong")),
                        (
                            Packets::PACKETCODECS_TYPE,
                            "NBT_COMPOUND"
                            | "UNLIMITED_NBT_COMPOUND"
                            | "NBT_ELEMENT"
                            | "UNLIMITED_NBT_ELEMENT",
                        ) => Some(String::from("Nbt")),
                        (Packets::PACKETCODECS_TYPE, _) => {
                            bail!(
                                "Unknown `PacketCodec`: \"{}.{}\"",
                                member.class_name,
                                member.name_and_type.name
                            )
                        }
                        (
                            "net/minecraft/text/TextCodecs",
                            "PACKET_CODEC" | "UNLIMITED_REGISTRY_PACKET_CODEC",
                        ) => Some(String::from("Text")),
                        (
                            "net/minecraft/text/TextCodecs",
                            "OPTIONAL_PACKET_CODEC" | "OPTIONAL_UNLIMITED_REGISTRY_PACKET_CODEC",
                        ) => Some(String::from("Option<Text>")),
                        ("net/minecraft/util/math/BlockPos", "PACKET_CODEC") => {
                            Some(String::from("BlockPos"))
                        }
                        _ => None,
                    }
                {
                    trace!("    Field: {field}");
                    fields.push(field);
                } else if member.name_and_type.name.contains("OPTION") {
                    // If the codec is an `Option`, mark it as such
                    trace!("    Field: Option");
                    fields.push(String::from("Option"));
                } else if member.name_and_type.name.contains("LIST") {
                    // If the codec is an `LIST`, mark it as such
                    trace!("    Field: Vec");
                    fields.push(String::from("Vec"));
                } else {
                    let member_classfile = data.jar_container.get_class_err(&member.class_name)?;
                    let member_field =
                        get_class_field(&member_classfile, &member.name_and_type.name)?;

                    fields.extend(parse_codec(&member_classfile, member_field, data)?);
                }
            }
        }
    }
    Ok(fields)
}

/// Parse a codec created by an `xmap` method.
///
/// Uses a combination of `tuple` and `encode`/`decode` methods.
fn parse_xmap(
    classfile: &ClassFile<'_>,
    bundle: &[&Opcode],
    data: &ExtractBundle,
) -> anyhow::Result<Vec<String>> {
    let mut fields = Vec::new();
    fields.extend(parse_tuple(bundle, data)?);
    fields.extend(parse_encode_decode(classfile, bundle, data)?);
    Ok(fields)
}
