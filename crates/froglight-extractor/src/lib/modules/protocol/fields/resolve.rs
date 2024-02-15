use std::borrow::Cow;

use cafebabe::{
    attributes::AttributeData,
    bytecode::{ByteCode, Opcode},
    constant_pool::MemberRef,
    descriptor::{FieldType, Ty},
    ClassFile, MethodInfo,
};
use tracing::{debug, error, warn};

use super::{
    find,
    output::{Output, OutputType, Resolvable},
};
use crate::classmap::ClassMap;

/// Compare the read and write methods of a packet to determine the fields.
///
/// If both methods agree on what the fields are,
/// then the fields are returned with their names.
///
/// If the methods disagree, then use whichever method has the most fields
pub(super) fn compare_methods<'a>(
    class: &ClassFile<'a>,
    reader: &'a MethodInfo<'_>,
    writer: &'a MethodInfo<'_>,
    classmap: &'a ClassMap,
) -> Output<'a> {
    let reader_fields = resolve_reader(class, reader);
    let writer_fields = resolve_writer(class, writer);

    // If no fields were found or if any of the fields are unnamed,
    if (reader_fields.is_empty() && writer_fields.is_empty())
        || (reader_fields.iter().any(|(name, _)| name.is_none())
            || writer_fields.iter().any(|(name, _)| name.is_none()))
    {
        // If the superclass has the `Packet` interface,
        // then use the primitive constructor
        if let Some(superclass) = &class.super_class {
            if superclass != "java/lang/Object" {
                if let Some(superclass) = classmap.get(superclass) {
                    if superclass
                        .interfaces
                        .iter()
                        .any(|i| i == "net/minecraft/network/packet/Packet")
                    {
                        if let Some(method) = find::primitive_constructor(class) {
                            debug!("Using primitive constructor for packet `{}`", class.this_class);
                            return Output::Unnamed(resolve_primitive(method));
                        }
                    }
                }
            }
        }

        warn!("Could not resolve fields for packet `{}`", class.this_class);
        return Output::Unnamed(Vec::new());
    }

    // Compare the fields that were found
    match (reader_fields, writer_fields) {
        // If the reader and writer are the same, then keep the names
        // These are *probably* the correct fields, but there's no guarantee
        (reader, writer) if reader == writer => {
            Output::Named(reader.into_iter().map(|(name, ty)| (name.unwrap(), ty)).collect())
        }
        // If the reader and writer are different, then keep the one with the most fields
        // These might be all the fields, but it's unlikely and should be checked manually
        (reader, writer) => {
            if reader.len() > writer.len() {
                Output::Unnamed(reader.into_iter().map(|(_, f)| f).collect())
            } else {
                Output::Unnamed(writer.into_iter().map(|(_, f)| f).collect())
            }
        }
    }
}

/// Resolve the fields from the read method of a packet.
pub(super) fn resolve_reader<'a>(
    class: &ClassFile<'a>,
    reader: &'a MethodInfo<'_>,
) -> Vec<(Option<Cow<'a, str>>, OutputType)> {
    let mut vec = Vec::new();

    let Some(code) = find_code(reader) else {
        error!("Could not find ByteCode for method `{}`", class.this_class);
        return vec;
    };

    for (index, (_, op)) in code.opcodes.iter().enumerate() {
        match op {
            Opcode::Invokevirtual(MemberRef { class_name, name_and_type }) => {
                if class_name == "net/minecraft/network/PacketByteBuf" {
                    if let Some(out) = OutputType::try_from_read_method(&name_and_type.name) {
                        vec.push((None, out));
                    } else if let Some(out) = OutputType::try_from_generic(
                        class,
                        &code.opcodes[index..std::cmp::min(index + 8, code.opcodes.len())],
                    ) {
                        vec.push((None, out));
                    } else if matches!(name_and_type.name.as_ref(), "readOptional" | "readNullable")
                    {
                        vec.push((None, OutputType::Option(Box::new(Resolvable::Unknown))));
                    } else if matches!(
                        name_and_type.name.as_ref(),
                        "readList" | "readSet" | "readCollection"
                    ) {
                        vec.push((None, OutputType::Vec(Box::new(Resolvable::Unknown))));
                    } else if matches!(name_and_type.name.as_ref(), "readMap") {
                        vec.push((
                            None,
                            OutputType::Map(
                                Box::new(Resolvable::Unknown),
                                Box::new(Resolvable::Unknown),
                            ),
                        ));
                    } else {
                        error!("Could not resolve read method `{}`", name_and_type.name);
                    }
                }
            }
            Opcode::Putfield(MemberRef { class_name, name_and_type }) => {
                if class.this_class == *class_name {
                    if let Some((last_name, _)) = vec.iter_mut().last() {
                        if last_name.is_none() {
                            *last_name = Some(name_and_type.name.clone());
                        }
                    }
                }
            }

            _ => {}
        }
    }

    vec
}

/// Resolve the fields from the write method of a packet.
pub(super) fn resolve_writer<'a>(
    class: &ClassFile<'a>,
    writer: &'a MethodInfo<'_>,
) -> Vec<(Option<Cow<'a, str>>, OutputType)> {
    let mut vec = Vec::new();

    let Some(code) = find_code(writer) else {
        error!("Could not find ByteCode for method `{}`", class.this_class);
        return vec;
    };

    let mut field_name = None;
    for (index, (_, op)) in code.opcodes.iter().enumerate() {
        match op {
            Opcode::Getfield(MemberRef { class_name, name_and_type }) => {
                if class.this_class == *class_name {
                    field_name = Some(name_and_type.name.clone());
                }
            }
            Opcode::Invokevirtual(MemberRef { class_name, name_and_type }) => {
                if class_name == "net/minecraft/network/PacketByteBuf" {
                    if let Some(out) = OutputType::try_from_write_method(&name_and_type.name) {
                        vec.push((field_name.take(), out));
                    } else if let Some(out) = OutputType::try_from_generic(
                        class,
                        &code.opcodes[index.saturating_sub(8)..index],
                    ) {
                        vec.push((field_name.take(), out));
                    } else if matches!(
                        name_and_type.name.as_ref(),
                        "writeOptional" | "writeNullable"
                    ) {
                        vec.push((
                            field_name.take(),
                            OutputType::Option(Box::new(Resolvable::Unknown)),
                        ));
                    } else if matches!(
                        name_and_type.name.as_ref(),
                        "writeList" | "writeSet" | "writeCollection"
                    ) {
                        vec.push((
                            field_name.take(),
                            OutputType::Vec(Box::new(Resolvable::Unknown)),
                        ));
                    } else if matches!(name_and_type.name.as_ref(), "writeMap") {
                        vec.push((
                            field_name.take(),
                            OutputType::Map(
                                Box::new(Resolvable::Unknown),
                                Box::new(Resolvable::Unknown),
                            ),
                        ));
                    } else {
                        error!("Could not resolve write method `{}`", name_and_type.name);
                    }
                }
            }
            _ => {}
        }
    }

    vec
}

/// Resolve the fields from a primitive constructor.
fn resolve_primitive(method: &MethodInfo<'_>) -> Vec<OutputType> {
    method
        .descriptor
        .parameters
        .iter()
        .filter_map(|p| match p {
            FieldType::Ty(Ty::Base(base)) => Some(OutputType::from(base.clone())),
            _ => None,
        })
        .collect()
}

/// Find the [`ByteCode`] of a method.
fn find_code<'a>(method: &'a MethodInfo<'_>) -> Option<&'a ByteCode<'a>> {
    method.attributes.iter().find_map(|attr| {
        if let AttributeData::Code(code) = &attr.data {
            code.bytecode.as_ref()
        } else {
            None
        }
    })
}
