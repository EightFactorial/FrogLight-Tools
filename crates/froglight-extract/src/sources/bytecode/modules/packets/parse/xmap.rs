use anyhow::bail;
use cafebabe::{
    bytecode::{ByteCode, Opcode},
    constant_pool::{MemberKind, MemberRef, MethodHandle},
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

const CODEC_OBJ: &str = "Lnet/minecraft/network/codec/PacketCodec;";

pub(super) fn create_xmap<'a>(
    classfile: &'a ClassFile<'_>,
    code: &'a ByteCode<'_>,
    index: usize,
) -> anyhow::Result<CodecType<'a>> {
    let mut ignore = false;
    let invert_index = code.opcodes.len() - index;
    if let Some((_, parent)) = code.opcodes.iter().rev().skip(invert_index).find(|(_, op)| {
        if ignore {
            false
        } else if let Opcode::Putstatic(_) = op {
            ignore = true;
            false
        } else if let Opcode::Getstatic(member) = op {
            member.name_and_type.descriptor == CODEC_OBJ
        } else {
            false
        }
    }) {
        let Opcode::Getstatic(parent) = parent else { unreachable!("Already checked opcode type") };
        let encode = super::create::get_method_handle(classfile, code, index - 2)?;
        let decode = super::create::get_method_handle(classfile, code, index - 3)?;
        Ok(CodecType::Xmap { parent, encode, decode })
    } else {
        bail!("Failed to find `Parent` for Xmap Codec in \"{}\"", classfile.this_class);
    }
}

/// Parse a [`CodecType::Xmap`] codec
pub(super) fn parse_xmap(
    _classfile: &ClassFile<'_>,
    parent: &MemberRef<'_>,
    decode: &MethodHandle<'_>,
    data: &ExtractBundle<'_>,
) -> anyhow::Result<Vec<String>> {
    let mut fields = Vec::new();

    // Parse the codec decode function
    fields.extend(match decode.member_kind {
        MemberKind::Field => {
            let decode_classfile = data.jar_container.get_class_err(&decode.class_name)?;
            let decode_field = get_class_field(&decode_classfile, &decode.member_ref.name)?;
            Packets::parse_codec_field(&decode_classfile, decode_field, data)?
        }
        MemberKind::Method | MemberKind::InterfaceMethod => {
            super::method::parse_method_handle(decode, data)?
        }
    });

    // Parse the parent codec
    fields.extend(if is_function(&parent.name_and_type.descriptor) {
        let parent_classfile = data.jar_container.get_class_err(&parent.class_name)?;
        super::method::parse_method(&parent_classfile, parent, data)?
    } else {
        let parent_classfile = data.jar_container.get_class_err(&parent.class_name)?;
        let parent_field = get_class_field(&parent_classfile, &parent.name_and_type.name)?;
        Packets::parse_codec_field(&parent_classfile, parent_field, data)?
    });

    Ok(fields)
}
