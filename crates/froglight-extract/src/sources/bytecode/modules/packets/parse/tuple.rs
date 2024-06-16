use anyhow::bail;
use cafebabe::{
    bytecode::{ByteCode, Opcode},
    constant_pool::InvokeDynamic,
    ClassFile,
};

use super::CodecType;
use crate::{
    bundle::ExtractBundle,
    sources::{
        helpers::{get_class_field, is_function},
        Packets,
    },
};

pub(super) fn create_tuple<'a>(
    classfile: &ClassFile<'_>,
    code: &'a ByteCode<'_>,
    index: usize,
) -> anyhow::Result<CodecType<'a>> {
    let bundle = bundle_opcodes(code, index);
    if let Opcode::Invokestatic(member) = bundle.last().expect("Tuple Codec has no Opcodes") {
        Ok(CodecType::Tuple { bundle, descriptor: &member.name_and_type.descriptor })
    } else {
        bail!("Failed to find `Invokestatic` for Tuple Codec in \"{}\"", classfile.this_class);
    }
}

/// Bundle together the opcodes that make up a codec
pub(super) fn bundle_opcodes<'a>(code: &'a ByteCode<'_>, index: usize) -> Vec<&'a Opcode<'a>> {
    let mut bundle = Vec::new();
    let invert_index = code.opcodes.len() - index;
    for (_, op) in code.opcodes.iter().rev().skip(invert_index) {
        if let Opcode::Putstatic(_) = op {
            break;
        }
        bundle.push(op);
    }
    bundle.reverse();
    bundle
}

const TUPLE_PREFIX: &str = "(";
const TUPLE_SUFFIX: &str = ")Lnet/minecraft/network/codec/PacketCodec;";

/// Parse a [`CodecType::Tuple`] codec
pub(super) fn parse_tuple(
    _classfile: &ClassFile<'_>,
    bundle: &[&Opcode<'_>],
    mut descriptor: &str,
    data: &ExtractBundle<'_>,
) -> anyhow::Result<Vec<String>> {
    descriptor = descriptor.trim_start_matches(TUPLE_PREFIX).trim_end_matches(TUPLE_SUFFIX);

    // Collect all of the members that are part of the tuple
    let mut members = Vec::new();
    for &op in bundle {
        match op {
            Opcode::Getstatic(member)
            | Opcode::Invokestatic(member)
            | Opcode::Invokeinterface(member, ..) => {
                let trimmed_desc = member.name_and_type.descriptor.split(')').last().unwrap();
                if descriptor.starts_with(trimmed_desc) {
                    descriptor = &descriptor[trimmed_desc.len()..];
                    members.push(member);
                }
            }
            Opcode::Invokedynamic(InvokeDynamic { name_and_type, .. }) => {
                let trimmed_desc = name_and_type.descriptor.split(')').last().unwrap();
                if descriptor.starts_with(trimmed_desc) {
                    descriptor = &descriptor[trimmed_desc.len()..];
                }
            }
            _ => {}
        }
    }

    // Parse the members of the tuple and collect the fields
    let mut fields = Vec::new();
    for member in members {
        let member_classfile = data.jar_container.get_class_err(&member.class_name)?;
        if is_function(&member.name_and_type.descriptor) {
            fields.extend(super::method::parse_method(&member_classfile, member, data)?);
        } else {
            let member_field = get_class_field(&member_classfile, &member.name_and_type.name)?;
            fields.extend(Packets::parse_codec_field(&member_classfile, member_field, data)?);
        }
    }
    Ok(fields)
}
