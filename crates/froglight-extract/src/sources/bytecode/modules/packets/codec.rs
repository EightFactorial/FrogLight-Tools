use std::borrow::Cow;

use cafebabe::{bytecode::Opcode, constant_pool::MemberRef, ClassFile};
use tracing::error;

use super::Packets;
use crate::sources::get_class_method_code;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum CodecConstructor<'a> {
    CreateCodec(Cow<'a, str>),
    Tuple(Cow<'a, str>),
    XMap(Cow<'a, str>),
    Unit,
}

impl Packets {
    const CODEC_METHOD: &'static str = "<clinit>";

    pub(super) const CODEC_FIELD_NAME: &'static str = "CODEC";
    pub(super) const CODEC_TYPE: &'static str = "Lnet/minecraft/network/codec/PacketCodec;";

    /// Get the [`CodecConstructor`] for the given [`ClassFile`].
    pub(super) fn get_codec_type<'a>(classfile: &'a ClassFile<'_>) -> Option<CodecConstructor<'a>> {
        let code = get_class_method_code(classfile, Self::CODEC_METHOD)?;

        let mut return_next = false;
        for (_, op) in code.opcodes.iter().rev() {
            // Look for when the `Self::CODEC_FIELD_NAME` field is initialized
            if let Opcode::Putstatic(MemberRef { name_and_type, .. }) = op {
                if name_and_type.name == Self::CODEC_FIELD_NAME
                    && name_and_type.descriptor == Self::CODEC_TYPE
                {
                    return_next = true;
                    continue;
                }
            }

            // Get the function called to create the codec,
            // just before the field is initialized
            if return_next {
                if let Opcode::Invokeinterface(MemberRef { name_and_type, .. }, _)
                | Opcode::Invokestatic(MemberRef { name_and_type, .. }) = op
                {
                    match &*name_and_type.name {
                        "createCodec" => {
                            return Some(CodecConstructor::CreateCodec(
                                name_and_type.descriptor.clone(),
                            ));
                        }
                        "tuple" => {
                            return Some(CodecConstructor::Tuple(name_and_type.descriptor.clone()));
                        }
                        "unit" => {
                            return Some(CodecConstructor::Unit);
                        }
                        "xmap" => {
                            return Some(CodecConstructor::XMap(name_and_type.descriptor.clone()));
                        }
                        _ => {
                            error!("Unknown Codec constructor: {name_and_type:?}");
                        }
                    }
                } else {
                    error!("Expected Codec constuctor, got: {op:?}");
                }
                break;
            }
        }
        None
    }
}
