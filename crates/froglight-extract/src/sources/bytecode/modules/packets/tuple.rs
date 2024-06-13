use anyhow::bail;
use cafebabe::{bytecode::Opcode, constant_pool::InvokeDynamic, ClassFile};

use super::Packets;
use crate::{
    bundle::ExtractBundle,
    bytecode::ClassContainer,
    sources::{
        bytecode::modules::packets::constructor::CodecConstructor, get_class_method_code,
        is_descriptor_function,
    },
};

impl Packets {
    const TUPLE_PREFIX: &'static str = "(";
    const TUPLE_SUFFIX: &'static str = ")Lnet/minecraft/network/codec/PacketCodec;";

    pub(super) fn parse_tuple(
        classfile: &ClassFile<'_>,
        codec: CodecConstructor<'_>,
        index: usize,
        data: &ExtractBundle<'_>,
    ) -> anyhow::Result<Vec<String>> {
        let CodecConstructor::Tuple(descriptor) = codec else { panic!("Expected tuple codec") };
        let Some(code) = get_class_method_code(classfile, Self::CODEC_METHOD) else {
            bail!(
                "Packet class \"{}\" has no \"{}\" method",
                classfile.this_class,
                Self::CODEC_METHOD
            );
        };

        // Seperate out the opcodes that are relevant to the tuple
        let mut opcode_bundle = Vec::new();
        for (_, op) in code.opcodes.iter().rev().skip(index) {
            if matches!(op, Opcode::Putstatic(_)) {
                break;
            }
            opcode_bundle.push(op);
        }
        opcode_bundle.reverse();

        // Iterate through the opcodes to find the fields used in the tuple
        let mut members = Vec::new();
        let mut descriptor = descriptor
            .as_ref()
            .trim_start_matches(Self::TUPLE_PREFIX)
            .trim_end_matches(Self::TUPLE_SUFFIX);

        // Use the codec descriptor to find relevant fields and method calls
        for op in opcode_bundle {
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

        // Extract the fields from the members
        let mut fields = Vec::new();
        for member in members {
            // If the descriptor is a function, parse it as a `CodecConstructor::Special`
            if is_descriptor_function(&member.name_and_type.descriptor) {
                if let Some(field) = Self::parse_special(CodecConstructor::Special(member)) {
                    fields.push(field);
                }
            } else if let Some(member_classfile) =
                data.jar_container.get(member.class_name.as_ref()).map(ClassContainer::parse)
            {
                // Otherwise parse it like a regular codec
                fields.extend(Self::codec_fields(
                    &member_classfile,
                    &member.name_and_type.name,
                    data,
                )?);
            } else {
                bail!("Error extracting packet fields, \"{}\" does not exist", member.class_name);
            }
        }

        Ok(fields)
    }
}
