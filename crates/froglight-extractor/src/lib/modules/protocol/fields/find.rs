use cafebabe::{
    descriptor::{FieldType, Ty},
    ClassFile, MethodInfo,
};

/// Find a method that reads from a `PacketByteBuf`.
pub(super) fn bytebuf_reader<'a>(class: &'a ClassFile) -> Option<&'a MethodInfo<'a>> {
    class.methods.iter().find(|&method| {
        method.name == "<init>"
            && method.descriptor.parameters.len() == 1
            && matches!(
                &method.descriptor.parameters[0],
                FieldType::Ty(Ty::Object(obj)) if obj == "net/minecraft/network/PacketByteBuf"
            )
    })
}

/// Find a method that writes to a `PacketByteBuf`.
pub(super) fn bytebuf_writer<'a>(class: &'a ClassFile) -> Option<&'a MethodInfo<'a>> {
    class.methods.iter().find(|&method| {
        method.name == "write"
            && method.descriptor.parameters.len() == 1
            && matches!(
                &method.descriptor.parameters[0],
                FieldType::Ty(Ty::Object(obj)) if obj == "net/minecraft/network/PacketByteBuf"
            )
    })
}

/// Find a non-empty constructor that only uses primitive types.
pub(super) fn primitive_constructor<'a>(class: &'a ClassFile) -> Option<&'a MethodInfo<'a>> {
    class.methods.iter().find(|&method| {
        method.name == "<init>"
            && !method.descriptor.parameters.is_empty()
            && method
                .descriptor
                .parameters
                .iter()
                .all(|param| matches!(param, FieldType::Ty(Ty::Base(_))))
    })
}
