use std::borrow::Cow;

use anyhow::bail;
use cafebabe::{bytecode::Opcode, constant_pool::MemberRef, ClassFile, FieldInfo};

use super::Packets;
use crate::sources::{get_class_method_code, is_descriptor_function};

#[derive(Debug)]
#[allow(dead_code)]
pub(super) enum CodecConstructor<'a> {
    Create(&'a Cow<'a, str>),
    Special(&'a MemberRef<'a>),
    Tuple(&'a Cow<'a, str>),
    Unit,
    XMap(&'a Cow<'a, str>),
}

impl Packets {
    pub(super) const CODEC_METHOD: &'static str = "<clinit>";

    /// Get the [`CodecConstructor`] for the given codec [`FieldInfo`].
    pub(super) fn codec_type<'a>(
        classfile: &'a ClassFile<'_>,
        codec: &FieldInfo<'_>,
    ) -> anyhow::Result<(CodecConstructor<'a>, usize)> {
        let Some(code) = get_class_method_code(classfile, Self::CODEC_METHOD) else {
            bail!(
                "Packet class \"{}\" has no \"{}\" method",
                classfile.this_class,
                Self::CODEC_METHOD
            );
        };

        let mut return_next = false;
        for (index, (_, op)) in code.opcodes.iter().rev().enumerate() {
            // Look for when the codec field is initialized
            if let Opcode::Putstatic(MemberRef { name_and_type, .. }) = op {
                if name_and_type.name == codec.name {
                    return_next = true;
                    continue;
                }
            }

            // Get the function called to create the codec,
            // just before the field is initialized
            if return_next {
                match op {
                    Opcode::Invokeinterface(
                        member @ MemberRef { class_name, name_and_type },
                        _,
                    )
                    | Opcode::Invokestatic(member @ MemberRef { class_name, name_and_type }) => {
                        if matches!(
                            &**class_name,
                            Self::PACKET_TYPE | Self::PACKET_CODEC_TYPE | Self::PACKET_CODECS_TYPE
                        ) {
                            match &*name_and_type.name {
                                "unit" => {
                                    return Ok((CodecConstructor::Unit, index));
                                }
                                "createCodec" => {
                                    return Ok((
                                        CodecConstructor::Create(&classfile.this_class),
                                        index,
                                    ));
                                }
                                "tuple" => {
                                    return Ok((
                                        CodecConstructor::Tuple(&name_and_type.descriptor),
                                        index,
                                    ));
                                }
                                "xmap" => {
                                    return Ok((
                                        CodecConstructor::XMap(&name_and_type.descriptor),
                                        index,
                                    ));
                                }
                                _ => {}
                            }
                        }

                        if is_descriptor_function(&name_and_type.descriptor) {
                            return Ok((CodecConstructor::Special(member), index));
                        }

                        bail!("Unknown Codec constructor: {member:?}");
                    }
                    Opcode::Invokespecial(member) => {
                        return Ok((CodecConstructor::Special(member), index));
                    }
                    _ => bail!("Expected Codec constuctor, got: {op:?}"),
                }
            }
        }

        bail!(
            "Failed to find codec constructor for \"{}\" in \"{}\"",
            codec.name,
            classfile.this_class
        )
    }
}
